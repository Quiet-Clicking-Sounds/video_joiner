use std::collections::VecDeque;
use std::convert::Into;
use std::fmt::{Debug, Formatter};
use std::io::Write;
use std::ops::{Div, Not, Rem, Sub};
use std::path::PathBuf;
use std::time::Instant;

use ffmpeg_sidecar;
use ffmpeg_sidecar::child::FfmpegChild;
use ffmpeg_sidecar::event::{LogLevel, OutputVideoFrame};
use ffmpeg_sidecar::iter::FfmpegIterator;
use ffmpeg_sidecar::{command::FfmpegCommand, event::FfmpegEvent};
use ffprobe;

use crate::audio::join_audio_video_streams;
use crate::frame_shape::FrameShape;
use crate::helper_functions;
use crate::helper_functions::{iter_ffmpeg_events, seconds_to_hhmmss, MultiPathBuf};
use crate::switches::SortOrder;


//noinspection SpellCheckingInspection
const DECODER: [&str; 2] = ["-hwaccel", "auto"];
// None     77, 85, 75, 80
// dxva2    83, 75, 83, 77
// d3d11va  ??, 77, 80, 86
// auto     71, 75, 80, 75
// cuda     Error
// qsv      depreciated
// opencl   Error
// vulkan   Error


#[derive(Clone)]
pub(crate) struct VideoEditData {
    pub(crate) output_height: u32,
    pub(crate) output_width: u32,
    shapes: Vec<(u32, u32)>,
    fps: f32,
}

impl VideoEditData {
    pub fn init() -> VideoEditData {
        let shapes = vec![(5, 5); 25];
        VideoEditData {
            output_height: 1440,
            output_width: 850 * 3,
            shapes,
            fps: 30f32,
        }
    }
    pub fn init_wxh(w: u32, h: u32, count: u32) -> VideoEditData {
        let shapes = (0..count)
            .into_iter()
            .map(|_| (w.clone() / count.clone(), h.clone()))
            .collect();

        VideoEditData {
            output_height: h,
            output_width: w,
            shapes,
            fps: 30f32,
        }
    }

    #[allow(unused)]
    pub fn init_abc_shape(widths: Vec<u32>, h: u32) -> VideoEditData {
        let shapes = widths.iter().map(|f| (f.clone(), h.clone())).collect();
        VideoEditData {
            output_height: h,
            output_width: widths.iter().sum(),
            shapes,
            fps: 30f32,
        }
    }
    pub fn set_fps(&mut self, fps: f32) {
        self.fps = fps
    }
    fn get_shapes(&self) -> &[(u32, u32)] {
        self.shapes.as_slice()
    }


    pub(crate) fn set_shape(&mut self, shaper: FrameShape) {
        match shaper {
            FrameShape::Mono => {
                self.shapes = vec![
                    (self.output_width, self.output_height),
                ]
            }
            FrameShape::Dual => {
                self.shapes = vec![
                    (self.output_width / 2, self.output_height),
                    (self.output_width - self.output_width / 2, self.output_height),
                ];
            }
            FrameShape::Triple => {
                let ow = self.output_width / 3;
                self.shapes = vec![
                    (ow, self.output_height),
                    (self.output_width - ow - ow, self.output_height),
                    (ow, self.output_height),
                ];
            }
            FrameShape::Quad => {
                let (w1, h1) = (self.output_width / 2, self.output_height / 2);
                let (w2, h2) = (self.output_width - w1, self.output_height - h1);
                self.shapes = vec![
                    (w1, h1),
                    (w2, h1),
                    (w1, h2),
                    (w2, h2),
                ];
            }
            FrameShape::VertEmph => {
                let w23 = self.output_width / 3;
                let h1 = self.output_height / 2;
                let h2 = self.output_height - h1;
                self.shapes = vec![
                    (self.output_width - w23 - w23, self.output_height),
                    (w23, h1),
                    (w23, h1),
                    (w23, h2),
                    (w23, h2),
                ];
            }
            FrameShape::HorizEmph => {
                let w23 = self.output_width / 3;
                let h1 = self.output_height / 2;
                self.shapes = vec![
                    (w23, self.output_height),
                    (self.output_width - w23 - w23, h1),
                    (self.output_width - w23 - w23, self.output_height - h1),
                    (w23, self.output_height),
                ];
            }
            FrameShape::VertEmph2 => {
                let owx = self.output_width.rem(8);
                let ow = self.output_width.sub(owx).div(8);

                self.shapes = vec![
                    (ow * 2 + owx, self.output_height),
                    (ow * 3, self.output_height / 2),
                    (ow * 3, self.output_height / 2),
                    (ow * 3, self.output_height - self.output_height / 2),
                    (ow * 3, self.output_height - self.output_height / 2),
                ];
            }
            FrameShape::HorizEmph2 => {
                let owx = self.output_width.rem(4);
                let ow = self.output_width.sub(owx).div(4);
                self.shapes = vec![
                    (ow, self.output_height),
                    (ow * 2 + owx, self.output_height / 2),
                    (ow * 2 + owx, self.output_height - self.output_height / 2),
                    (ow, self.output_height),
                ];
            }
            FrameShape::SideVert | FrameShape::SideVert2 => {
                let owx = self.output_width.rem(3);
                let ow = self.output_width.sub(owx).div(3);
                self.shapes = vec![
                    (ow, self.output_height),
                    (ow * 2 + owx, self.output_height / 2),
                    (ow * 2 + owx, self.output_height - self.output_height / 2),
                ];
            }
            FrameShape::CentreEmphVert | FrameShape::CentreEmphVert2 => {
                let wmid = self.output_width / 5 * 3;
                let w_l = (self.output_width - wmid) / 2;
                let w_r = self.output_width - wmid - w_l;

                let wml = wmid / 2;
                let wmr = wmid - wml;

                let htop = self.output_height / 5 * 3;
                let hbot = self.output_height - htop;

                assert_eq!(wmid + w_l + w_r, self.output_width);
                assert_eq!(wml + wmr + w_l + w_r, self.output_width);
                assert_eq!(htop + hbot, self.output_height);

                self.shapes = vec![
                    (wmid, htop),
                    (w_l, self.output_height),
                    (w_r, self.output_height),
                    (wml, hbot),
                    (wmr, hbot),
                ]
            }
            FrameShape::MoreHoriz | FrameShape::MoreHoriz2 => {
                let wmid = self.output_width / 5 * 3;
                let w_l = (self.output_width - wmid) / 2;
                let w_r = self.output_width - wmid - w_l;

                let wml = wmid / 2;
                let wmr = wmid - wml;

                let h_mid_big = self.output_height / 5 * 3;
                let h_mid_small = self.output_height - h_mid_big;

                let h_side_big = self.output_height / 3 * 2;
                let h_side_small = self.output_height - h_side_big;

                assert_eq!(wmid + w_l + w_r, self.output_width);
                assert_eq!(wml + wmr + w_l + w_r, self.output_width);
                assert_eq!(h_mid_big + h_mid_small, self.output_height);

                self.shapes = vec![
                    (wmid, h_mid_big),
                    (w_l, h_side_big),
                    (w_r, h_side_big),
                    (wml, h_mid_small),
                    (wmr, h_mid_small),
                    (w_l, h_side_small),
                    (w_r, h_side_small),
                ]
            }
            FrameShape::ExtendedLandscape => {
                let v_1_3_a = self.output_height / 3;
                let v_1_3_b = self.output_height / 3;
                let v_1_3_c = self.output_height - v_1_3_a - v_1_3_b;

                let wmid = self.output_width / 5 * 3;
                let w_l = (self.output_width - wmid) / 2;
                let w_r = self.output_width - wmid - w_l;

                let wml = wmid / 2;
                let wmr = wmid - wml;

                let h_mid_big = self.output_height / 5 * 3;
                let h_mid_small = self.output_height - h_mid_big;


                self.shapes = vec![
                    (wmid, h_mid_big),
                    (wml, h_mid_small),
                    (wmr, h_mid_small),
                    (w_l, v_1_3_a),
                    (w_l, v_1_3_b),
                    (w_l, v_1_3_c),
                    (w_r, v_1_3_a),
                    (w_r, v_1_3_b),
                    (w_r, v_1_3_c),
                ]
            }
            FrameShape::ExtendedLandscape2 => {
                let horizontal_thirds = self.output_width / 3;
                let horizontal_thirds_mid = self.output_width - horizontal_thirds - horizontal_thirds;

                let vertical_top_thirds = self.output_height / 3;
                let vertical_thirds = self.output_height - vertical_top_thirds - vertical_top_thirds;

                let wmid = self.output_width / 5 * 3;
                let w_l = (self.output_width - wmid) / 2;
                let w_r = self.output_width - wmid - w_l;

                self.shapes = vec![
                    (wmid, vertical_top_thirds + vertical_top_thirds),
                    (horizontal_thirds, vertical_thirds),
                    (horizontal_thirds_mid, vertical_thirds),
                    (horizontal_thirds, vertical_thirds),
                    (w_l, vertical_top_thirds),
                    (w_r, vertical_top_thirds),
                    (w_l, vertical_top_thirds),
                    (w_r, vertical_top_thirds),
                ]
            }
            FrameShape::OffsetVH4x4 => {
                let horizontal_023 = self.output_width / 4;
                let horizontal_1 = self.output_width - horizontal_023 * 3;
                let height_min = self.output_height / 4;
                let height_max = self.output_height - height_min;
                self.shapes = vec![
                    (horizontal_1, height_min),
                    (horizontal_023, height_min),
                    (horizontal_023, height_min),
                    (horizontal_023, height_min),
                    (horizontal_023, height_max),
                    (horizontal_023, height_max),
                    (horizontal_1, height_max),
                    (horizontal_023, height_max),
                ]
            }
        }
    }
}

#[cfg(test)]
impl PartialEq for VideoEditData {
    fn eq(&self, other: &Self) -> bool {
        for (a, b) in self.shapes.iter().zip(other.shapes.iter()) {
            if a != b {
                return false;
            }
        }
        if self.output_height != other.output_height {
            false
        } else if self.output_width != other.output_width {
            return false;
        } else {
            return true;
        }
    }
}

pub(crate) struct Video {
    pub(crate) src: PathBuf,
    frame_iterable: Option<FfmpegIterator>,
    pub(crate) length_millis: Option<i64>,
    frame_count: u64,
    frame_start: u64,
    width_height: (u32, u32),
    fps: f32,
}

impl Video {
    pub fn from_path(src: impl Into<PathBuf>) -> Video {
        #[cfg(feature = "hyperDebug")]
        helper_functions::parse_debug(" Video from Path ", file!(), line!());
        Video {
            src: src.into(),
            frame_iterable: None,
            length_millis: None,
            frame_count: 0u64,
            frame_start: 0u64,
            width_height: (0, 0),
            fps: 0f32,
        }
    }
    fn set_video_shape(&mut self, wxh: (u32, u32), fps: f32) {
        self.width_height = wxh;
        self.fps = fps;
    }


    //noinspection SpellCheckingInspection
    fn audio_export_proc_out(&self, out: &PathBuf) -> (bool, Option<FfmpegChild>) {
        if self.frame_count < 1 {
            return (false, None);
        };

        let tar = self.src.clone();
        let length = format!("{:.6}s", (self.frame_count.clone() as f32) / &self.fps);

        let mut ffm = FfmpegCommand::new();
        let ffm = ffm.input(tar.to_str().unwrap()).no_video();
        let ffm = ffm.filter(format!(
            "[0:a]apad=whole_dur={}[a]",
            length
        ));
        let ffm = ffm
            .args([
                "-t", &length,
                "-ar", "44100"
            ])
            .arg("-y")
            .output(out.to_str().unwrap());

        let complete = ffm.spawn().unwrap();
        (true, Some(complete))
    }

    //noinspection SpellCheckingInspection
    pub(crate) fn get_length(&mut self) -> Result<i64, &str> {
        #[cfg(feature = "hyperDebug")]
        helper_functions::parse_debug(" get length ", file!(), line!());
        match self.length_millis {
            None => {
                let info = match ffprobe::ffprobe(self.src.clone()) {
                    Ok(info) => info,
                    Err(_) => {
                        return Err("ffprobe failed");
                    }
                };
                match info.format.try_get_duration() {
                    None => {
                        return Err("failed to find duration");
                    }
                    Some(Err(_)) => {
                        return Err("float parse error for duration");
                    }
                    Some(Ok(t)) => self.length_millis = Some(t.as_millis() as i64),
                }
            }
            Some(_) => {}
        }
        Ok(self.length_millis.unwrap().clone())
    }

    //noinspection SpellCheckingInspection
    fn setup_video(&mut self) {
        #[cfg(feature = "hyperDebug")]
        helper_functions::parse_debug(" setup_video ", file!(), line!());

        #[cfg(feature = "hyperDebug")]
        helper_functions::parse_debug(format!("crop=w={:?}:h={:?} fps={}", self.width_height.0,
                                              self.width_height.1, self.fps).as_str(),
                                      file!(), line!());
        let filtergraph = {
            let scaler = format!("scale={:?}:{:?}:force_original_aspect_ratio=increase",
                                 self.width_height.0, self.width_height.1);
            let crop = format!("crop=w={:?}:h={:?}",
                               self.width_height.0, self.width_height.1);
            let fps = format!("fps=fps={}", self.fps);
            format!("[0:v]{}[a];[a]{}[b];[b]{}[output]",
                    scaler,
                    crop,
                    fps,
            )
        };

        let mut ffm = FfmpegCommand::new();
        let ffm = if DECODER[1].len() > 2 {
            ffm.args(DECODER)
        } else { &mut ffm };
        let ffm = ffm.input(self.src.to_str().unwrap());
        let ffm = ffm.filter(filtergraph);
        let ffm = ffm.rawvideo();
        #[cfg(feature = "hyperDebug")]
        let ffm = ffm.print_command();
        // for debugging ffmpeg bits, this will push the report per file into the /ffreport/ folder

        #[cfg(feature = "ffmpegReport")]
        println!("IF IT ERRORS OUT YOU WILL NEED TO MAKE THE /ffreport/ FOLDER");
        #[cfg(feature = "ffmpegReport")]
        ffm.as_inner_mut().env(
            "FFREPORT",
            format!(
                "file=ffreport/v_{}.log:level=48",
                self.src.file_name().unwrap().to_str().unwrap()
            ).as_str(),
        );

        self.frame_iterable = Some(ffm.spawn().unwrap().iter().unwrap());

        eprintln!("Begin Frames: {:?}", self.src)
    }

    pub fn next_frame(&mut self) -> Option<OutputVideoFrame> {
        #[cfg(feature = "hyperDebug")]
        helper_functions::parse_debug(" Video::next_frame() ", file!(), line!());
        for _ in 0..1000 {
            // frames aren't always a frame, only return a Frame or None
            // setup if no video currently setup
            if self.frame_iterable.is_none() {
                self.setup_video();
                continue;
            }

            // grab the next frame without dropping it from the Option<>
            let frame = self.frame_iterable.as_mut().map(|f| f.next())?;

            // if it doesn't return a frame it's probably done for
            let frame = match frame {
                None => {
                    return None;
                }
                Some(f) => f,
            };
            // return frame if the item is a frame, otherwise print updates
            match frame {
                FfmpegEvent::OutputFrame(frame) => {
                    self.frame_count += 1;
                    return Some(frame);
                }
                // FfmpegEvent::ParsedStreamMapping(m) => {
                //     eprintln!("ParsedStreamMapping: {}", m);
                // }
                FfmpegEvent::Done => {
                    eprintln!("Done: {:?}", self.src)
                }
                FfmpegEvent::Log(level, data) => {
                    match level {
                        LogLevel::Info => {
                            #[cfg(feature = "hyperDebug")]
                            helper_functions::parse_debug("LogLevel::Info", file!(), line!());
                            #[cfg(feature = "hyperDebug")]
                            eprintln!("LOG {:?} : {}", level, data);
                        }
                        LogLevel::Warning | LogLevel::Error | LogLevel::Fatal => {
                            eprintln!("LOG {:?} : {}  --  {:?}", level, data, self.src);
                            // return None;
                        }
                        LogLevel::Unknown => {
                            if data.len() > 2 {
                                eprintln!("LOG {:?} : {}", level, data)
                            }
                        }
                    }
                }
                // Ignored events
                #[cfg(feature = "hyperDebug")]
                FfmpegEvent::ParsedConfiguration(f) => {
                    eprintln!("LOG {:?} ", f);
                }
                #[cfg(feature = "hyperDebug")]
                FfmpegEvent::ParsedVersion(f) => {
                    eprintln!("LOG {:?} ", f);
                }
                #[cfg(feature = "hyperDebug")]
                FfmpegEvent::Progress(f) => {
                    eprintln!("LOG {:?} ", f);
                }

                _ => {
                    //eprintln!("___ {:?}", frame)
                }
            }
        }
        None
    }
}

struct VideoList {
    videos: VecDeque<Video>,

    complete_videos: VecDeque<Video>,
    video_sizer: VideoEditData,
    position: u32,
}


impl VideoList {
    pub fn from_videos(videos: Vec<Video>, pos: u32, sorter: SortOrder) -> VideoList {
        #[cfg(feature = "hyperDebug")]
        helper_functions::parse_debug(" from_videos ", file!(), line!());
        let vid_count = videos.len();
        let videos: Vec<Video> = sorter.apply_sort(videos);
        let mut vl = VideoList {
            videos: VecDeque::from(videos),
            complete_videos: VecDeque::with_capacity(vid_count),
            video_sizer: VideoEditData::init(),
            position: pos,
        };

        vl.set_video_sizer(vl.video_sizer.clone());
        vl
    }
    fn set_video_sizer(&mut self, vs: VideoEditData) {
        #[cfg(feature = "hyperDebug")]
        helper_functions::parse_debug("Setting VideoSizer in VideList", file!(), line!());
        self.video_sizer = vs;
        let vid_shape = self.video_sizer.get_shapes()[self.position as usize];
        for v in self.videos.iter_mut() {
            v.set_video_shape(vid_shape, self.video_sizer.fps)
        }
    }

    fn cheap_audio_exporter_out_proc(&mut self, grp: usize, temp_folder: &PathBuf) -> Vec<PathBuf> {
        let mut outputs = vec![];
        let mut out_proc = vec![];
        for (i, vid) in self.complete_videos.iter().enumerate() {
            println!("Audio Export: {}", vid.src.clone().to_str().unwrap());
            let out = temp_folder.clone().join(format!("g{}f{}.wav", grp, i));
            match vid.audio_export_proc_out(&out) {
                (false, _) => {}
                (true, Some(proc)) => {
                    outputs.push(out);
                    out_proc.push(proc);
                }
                (_, _) => {}
            }
        }
        for i in out_proc.iter_mut() {
            iter_ffmpeg_events(i);
            i.wait().unwrap();
        }
        outputs
    }

    /// returns frames until the end of all loaded videos, then returns None
    fn next_frame(&mut self, fc: &u64) -> Option<OutputVideoFrame> {
        match self.videos.front_mut() {
            None => {
                #[cfg(feature = "hyperDebug")]
                println!("Error matching front video, total videos {}", self.videos.len())
            }
            Some(v) => { v.frame_start = fc.clone() }
        }
        for _ in 0..10 {

            // if front video is None the stream of stream has been exhausted, end the process
            let frame = match self.videos.front_mut() {
                None => {
                    return None;
                }
                Some(f) => f.next_frame(),
            };
            match frame {
                // if frame is non the current video has ended, pop it then move onto the next one
                None => {
                    match self.videos.pop_front() {
                        Some(vid) => {
                            self.complete_videos.push_back(vid);
                            match self.videos.front_mut() {
                                Some(fm) => fm.frame_start = fc.clone(),
                                None => {}
                            };
                        }
                        None => {
                            break;
                        }
                    }

                    continue;
                }
                Some(f) => {
                    return Some(f);
                }
            }
        }
        println!("WARNING got to 10 calls without a frame in {}", self.videos.front()?.src.to_str()?);
        #[cfg(feature = "hyperDebug")]
        helper_functions::parse_debug("frame not found, next video failed to load", file!(), line!());

        None
    }
    fn pop_first_vid(&mut self) -> bool {
        match self.videos.pop_front() {
            Some(vid) => {
                self.complete_videos.push_back(vid);
                true
            }
            None => false,
        }
    }
}

pub struct VideoGroup {
    videos: Vec<VideoList>,
    output_target: PathBuf,
    video_sizer: VideoEditData,
    shape_style: FrameShape,
}

impl VideoGroup {
    pub(crate) fn print_time(&mut self, extra_info: bool) {
        let mut min_len = i64::MAX;
        for (x, vid) in self.videos.iter_mut().enumerate() {
            let len = vid.videos.iter_mut().fold(0, |a, f| {
                a + f.get_length().unwrap_or(0)
            }) / 1000;
            if len < min_len { min_len = len }
            println!("Video: {} length: {}", x, seconds_to_hhmmss(len as u64));
        }
        println!("end video length: {}", seconds_to_hhmmss(min_len as u64));

        if !extra_info { return; }
        for (x, vid) in self.videos.iter_mut().enumerate() {
            println!("Video Group: {}", x);
            for v in vid.videos.iter() {
                println!("\t{}", v.src.to_str().unwrap());
            }
        }
    }
}

impl VideoGroup {
    pub fn new_from_folder(
        src: impl Into<MultiPathBuf>,
        src_out: impl Into<PathBuf>,
        screens: FrameShape,
        sorter: SortOrder,
    ) -> VideoGroup {
        #[cfg(feature = "hyperDebug")]
        helper_functions::parse_debug("new_from_folder", file!(), line!());

        let videos = helper_functions::video_group_swap_n(src, screens.clone().count() as usize);

        // setup group for exporting
        VideoGroup {
            videos: videos
                .into_iter()
                .enumerate()
                .map(|(i, x)| VideoList::from_videos(x, i as u32, sorter.clone()))
                .collect(),
            output_target: src_out.into(),
            video_sizer: VideoEditData::init(),
            shape_style: screens,
        }
    }

    //noinspection DuplicatedCode
    pub fn new_from_folders(
        srcs: Vec<MultiPathBuf>,
        src_out: impl Into<PathBuf>,
        screens: FrameShape,
        sorter: SortOrder,
    ) -> VideoGroup {
        // Special cases for vertical and horuizontal input groups
        let vid_lists: Vec<VideoList> = match (screens.clone(), srcs.len()) {
            (FrameShape::VertEmph, 2) | (FrameShape::VertEmph2, 2) => {
                // vertical parts
                let videos1 = VideoList::from_videos(helper_functions::scan_dir_for_videos(srcs[0].clone()), 0, sorter.clone());
                // horizontal parts
                let mut videos2 = helper_functions::video_group_swap(srcs[1].clone(), FrameShape::Quad).into_iter();
                vec![
                    videos1,
                    VideoList::from_videos(videos2.next().unwrap(), 1, sorter.clone()),
                    VideoList::from_videos(videos2.next().unwrap(), 2, sorter.clone()),
                    VideoList::from_videos(videos2.next().unwrap(), 3, sorter.clone()),
                    VideoList::from_videos(videos2.next().unwrap(), 4, sorter.clone()),
                ]
            }
            (FrameShape::HorizEmph, 2) | (FrameShape::HorizEmph2, 2) => {
                let mut videos1 = helper_functions::video_group_swap(srcs[0].clone(), FrameShape::Dual).into_iter();
                let mut videos2 = helper_functions::video_group_swap(srcs[1].clone(), FrameShape::Dual).into_iter();
                vec![
                    VideoList::from_videos(videos1.next().unwrap(), 0, sorter.clone()),
                    VideoList::from_videos(videos2.next().unwrap(), 1, sorter.clone()),
                    VideoList::from_videos(videos2.next().unwrap(), 2, sorter.clone()),
                    VideoList::from_videos(videos1.next().unwrap(), 3, sorter.clone()),
                ]
            }
            (FrameShape::SideVert, 2) | (FrameShape::SideVert2, 2) => {
                // vertical parts
                let videos1 = VideoList::from_videos(helper_functions::scan_dir_for_videos(srcs[0].clone()), 0, sorter.clone());
                // horizontal parts
                let mut videos2 = helper_functions::video_group_swap(srcs[1].clone(), FrameShape::Dual).into_iter();
                vec![
                    videos1,
                    VideoList::from_videos(videos2.next().unwrap(), 1, sorter.clone()),
                    VideoList::from_videos(videos2.next().unwrap(), 2, sorter.clone()),
                ]
            }
            (FrameShape::CentreEmphVert, 3) | (FrameShape::CentreEmphVert2, 3) => {
                // top horizontal group
                let videos1 = VideoList::from_videos(helper_functions::scan_dir_for_videos(srcs[0].clone()), 0, sorter.clone());
                // vertical group
                let mut videos2 = helper_functions::video_group_swap(srcs[1].clone(), FrameShape::Dual).into_iter();
                // bottom horizontal group
                let mut videos3 = helper_functions::video_group_swap(srcs[2].clone(), FrameShape::Dual).into_iter();

                vec![
                    videos1,
                    VideoList::from_videos(videos2.next().unwrap(), 1, sorter.clone()),
                    VideoList::from_videos(videos2.next().unwrap(), 2, sorter.clone()),
                    VideoList::from_videos(videos3.next().unwrap(), 3, sorter.clone()),
                    VideoList::from_videos(videos3.next().unwrap(), 4, sorter.clone()),
                ]
            }
            (FrameShape::CentreEmphVert, 2) | (FrameShape::CentreEmphVert2, 2) => {
                // vertical group
                let mut videos2 = helper_functions::video_group_swap(srcs[0].clone(), FrameShape::Dual).into_iter();
                // bottom horizontal group
                let mut videos3 = helper_functions::video_group_swap(srcs[1].clone(), FrameShape::Triple).into_iter();

                vec![
                    VideoList::from_videos(videos3.next().unwrap(), 0, sorter.clone()),
                    VideoList::from_videos(videos2.next().unwrap(), 1, sorter.clone()),
                    VideoList::from_videos(videos2.next().unwrap(), 2, sorter.clone()),
                    VideoList::from_videos(videos3.next().unwrap(), 3, sorter.clone()),
                    VideoList::from_videos(videos3.next().unwrap(), 4, sorter.clone()),
                ]
            }
            (FrameShape::MoreHoriz, 3) | (FrameShape::MoreHoriz2, 3) => {
                // top horizontal group
                let videos1 = VideoList::from_videos(helper_functions::scan_dir_for_videos(srcs[0].clone()), 0, sorter.clone());
                // vertical group
                let mut videos2 = helper_functions::video_group_swap(srcs[1].clone(), FrameShape::Dual).into_iter();
                // bottom horizontal group
                let mut videos3 = helper_functions::video_group_swap(srcs[2].clone(), FrameShape::Quad).into_iter();
                vec![
                    videos1,
                    VideoList::from_videos(videos2.next().unwrap(), 1, sorter.clone()),
                    VideoList::from_videos(videos2.next().unwrap(), 2, sorter.clone()),
                    VideoList::from_videos(videos3.next().unwrap(), 3, sorter.clone()),
                    VideoList::from_videos(videos3.next().unwrap(), 4, sorter.clone()),
                    VideoList::from_videos(videos3.next().unwrap(), 5, sorter.clone()),
                    VideoList::from_videos(videos3.next().unwrap(), 6, sorter.clone()),
                ]
            }
            (FrameShape::ExtendedLandscape, 2) => {
                let videos0 = VideoList::from_videos(helper_functions::scan_dir_for_videos(srcs[0].clone()), 0, sorter.clone());
                let mut videos1 = helper_functions::video_group_swap_n(srcs[1].clone(), 8).into_iter();

                vec![
                    videos0,
                    VideoList::from_videos(videos1.next().unwrap(), 1, sorter.clone()),
                    VideoList::from_videos(videos1.next().unwrap(), 2, sorter.clone()),
                    VideoList::from_videos(videos1.next().unwrap(), 3, sorter.clone()),
                    VideoList::from_videos(videos1.next().unwrap(), 4, sorter.clone()),
                    VideoList::from_videos(videos1.next().unwrap(), 5, sorter.clone()),
                    VideoList::from_videos(videos1.next().unwrap(), 6, sorter.clone()),
                    VideoList::from_videos(videos1.next().unwrap(), 7, sorter.clone()),
                    VideoList::from_videos(videos1.next().unwrap(), 8, sorter.clone()),
                ]
            }
            (FrameShape::ExtendedLandscape, 3) => {
                let videos1 = VideoList::from_videos(helper_functions::scan_dir_for_videos(srcs[0].clone()), 0, sorter.clone());
                let mut videos2 = helper_functions::video_group_swap_n(srcs[1].clone(), 2).into_iter();
                let mut videos3 = helper_functions::video_group_swap_n(srcs[2].clone(), 6).into_iter();

                vec![
                    videos1,
                    VideoList::from_videos(videos2.next().unwrap(), 1, sorter.clone()),
                    VideoList::from_videos(videos2.next().unwrap(), 2, sorter.clone()),
                    VideoList::from_videos(videos3.next().unwrap(), 3, sorter.clone()),
                    VideoList::from_videos(videos3.next().unwrap(), 4, sorter.clone()),
                    VideoList::from_videos(videos3.next().unwrap(), 5, sorter.clone()),
                    VideoList::from_videos(videos3.next().unwrap(), 6, sorter.clone()),
                    VideoList::from_videos(videos3.next().unwrap(), 7, sorter.clone()),
                    VideoList::from_videos(videos3.next().unwrap(), 8, sorter.clone()),
                ]
            }
            (FrameShape::ExtendedLandscape, 4) => {
                let videos0 = VideoList::from_videos(helper_functions::scan_dir_for_videos(srcs[0].clone()), 0, sorter.clone());
                let mut videos1 = helper_functions::video_group_swap(srcs[1].clone(), FrameShape::Dual).into_iter();
                let mut videos2 = helper_functions::video_group_swap(srcs[2].clone(), FrameShape::Triple).into_iter();
                let mut videos3 = helper_functions::video_group_swap(srcs[3].clone(), FrameShape::Triple).into_iter();

                vec![
                    videos0,
                    VideoList::from_videos(videos1.next().unwrap(), 1, sorter.clone()),
                    VideoList::from_videos(videos1.next().unwrap(), 2, sorter.clone()),
                    VideoList::from_videos(videos2.next().unwrap(), 3, sorter.clone()),
                    VideoList::from_videos(videos2.next().unwrap(), 4, sorter.clone()),
                    VideoList::from_videos(videos2.next().unwrap(), 5, sorter.clone()),
                    VideoList::from_videos(videos3.next().unwrap(), 6, sorter.clone()),
                    VideoList::from_videos(videos3.next().unwrap(), 7, sorter.clone()),
                    VideoList::from_videos(videos3.next().unwrap(), 8, sorter.clone()),
                ]
            }
            (FrameShape::ExtendedLandscape2, 2) => {
                let videos1 = VideoList::from_videos(helper_functions::scan_dir_for_videos(srcs[0].clone()), 0, sorter.clone());
                let mut videos2 = helper_functions::video_group_swap_n(srcs[1].clone(), 7).into_iter();

                vec![
                    videos1,
                    VideoList::from_videos(videos2.next().unwrap(), 1, sorter.clone()),
                    VideoList::from_videos(videos2.next().unwrap(), 2, sorter.clone()),
                    VideoList::from_videos(videos2.next().unwrap(), 3, sorter.clone()),
                    VideoList::from_videos(videos2.next().unwrap(), 4, sorter.clone()),
                    VideoList::from_videos(videos2.next().unwrap(), 5, sorter.clone()),
                    VideoList::from_videos(videos2.next().unwrap(), 6, sorter.clone()),
                    VideoList::from_videos(videos2.next().unwrap(), 7, sorter.clone()),
                ]
            }
            (FrameShape::ExtendedLandscape2, 3) => {
                let videos1 = VideoList::from_videos(helper_functions::scan_dir_for_videos(srcs[0].clone()), 0, sorter.clone());
                let mut videos2 = helper_functions::video_group_swap_n(srcs[1].clone(), 3).into_iter();
                let mut videos3 = helper_functions::video_group_swap_n(srcs[2].clone(), 4).into_iter();
                vec![
                    videos1,
                    VideoList::from_videos(videos2.next().unwrap(), 1, sorter.clone()),
                    VideoList::from_videos(videos2.next().unwrap(), 2, sorter.clone()),
                    VideoList::from_videos(videos2.next().unwrap(), 3, sorter.clone()),
                    VideoList::from_videos(videos3.next().unwrap(), 4, sorter.clone()),
                    VideoList::from_videos(videos3.next().unwrap(), 5, sorter.clone()),
                    VideoList::from_videos(videos3.next().unwrap(), 6, sorter.clone()),
                    VideoList::from_videos(videos3.next().unwrap(), 7, sorter.clone()),
                ]
            }

            (FrameShape::OffsetVH4x4, 2) => {
                let mut videos2 = helper_functions::video_group_swap_n(srcs[0].clone(), 4).into_iter();
                let mut videos3 = helper_functions::video_group_swap_n(srcs[1].clone(), 4).into_iter();
                vec![
                    VideoList::from_videos(videos2.next().unwrap(), 0, sorter.clone()),
                    VideoList::from_videos(videos2.next().unwrap(), 1, sorter.clone()),
                    VideoList::from_videos(videos2.next().unwrap(), 2, sorter.clone()),
                    VideoList::from_videos(videos2.next().unwrap(), 3, sorter.clone()),
                    VideoList::from_videos(videos3.next().unwrap(), 4, sorter.clone()),
                    VideoList::from_videos(videos3.next().unwrap(), 5, sorter.clone()),
                    VideoList::from_videos(videos3.next().unwrap(), 6, sorter.clone()),
                    VideoList::from_videos(videos3.next().unwrap(), 7, sorter.clone()), ]
            }
            (_, _) => {
                srcs.into_iter()
                    .enumerate()
                    .map(|(i, x)| VideoList::from_videos(helper_functions::scan_dir_for_videos(x), i as u32, sorter.clone()))
                    .collect()
            }
        };
        VideoGroup {
            videos: vid_lists,
            output_target: src_out.into(),
            video_sizer: VideoEditData::init(),
            shape_style: screens,
        }
    }

    pub fn set_video_sizer(&mut self, video_sizer: VideoEditData) {
        self.video_sizer = video_sizer.clone();

        for vid in self.videos.iter_mut() {
            vid.set_video_sizer(video_sizer.clone())
        }
    }

    pub fn main_loop(&mut self, drop_audio: bool, encoder_args: Vec<String>) {
        let temp_folder = std::env::current_dir().unwrap().join("TempFolder");

        println!("TempFolder: {:?}", temp_folder);

        if temp_folder.exists().not() {
            std::fs::create_dir_all(temp_folder.clone()).unwrap();
        }

        let encoder_args: Vec<&str> = encoder_args.iter().map(|s| s.as_str()).collect();
        // Main loop **Video**
        let temp_file = self.main_loop_video(encoder_args.as_slice());

        if drop_audio {
            for _ in 0..3 {
                match std::fs::rename(temp_file.clone(), self.output_target.clone()) {
                    Ok(_) => { break }
                    Err(e) => {
                        println!("Sleeping due to Error:{:?}", e);
                        std::thread::sleep(std::time::Duration::from_secs_f32(0.2))
                    }
                }
            }
        } else {
            // Main loop **Audio**
            self.main_loop_audio(&temp_file, &temp_folder);
        }
        #[cfg(not(feature = "keepTempFiles"))]
        match std::fs::remove_dir_all(temp_folder) {
            Ok(_) => {}
            Err(e) => {
                eprintln!("Removing temp_dir failed Err: {:?}", e)
            }
        };
    }

    //noinspection SpellCheckingInspection
    fn main_loop_video(&mut self, encoder_args: &[&str]) -> PathBuf {
        #[cfg(feature = "hyperDebug")]
        helper_functions::parse_debug("main_loop", file!(), line!());
        // frame timer for simple speed testing:

        if self.videos[0..(self.shape_style.count() as usize)].iter().any(|a| a.videos.len() == 0) {
            println!("Error some required video parts did not contain videos");
            panic!();
        }

        let mut ft = helper_functions::FrameTimer::new();

        let temp_out_file = self.output_target.clone().with_file_name(format!(
            "__temp__{}", self.output_target.file_name().unwrap().to_str().unwrap()));

        // create output file
        let mut out_target = FfmpegCommand::new();
        out_target
            .args([
                "-loglevel",
                "error",
                "-f",
                "rawvideo",
                "-pix_fmt",
                "rgb24",
                "-s",
                &format!(
                    "{}x{}",
                    self.video_sizer.output_width, self.video_sizer.output_height
                ),
                "-r",
                &format!("{}", self.video_sizer.fps),
            ])
            .input("pipe:0")
            .args(["-y"])
            .args(encoder_args)
            // .arg(ENCODER_ARGS.get().unwrap())
            .output(temp_out_file.to_str().unwrap());


        println!("FFMPEG ouput Command:");
        out_target.print_command();

        let mut out_target = out_target.spawn().unwrap();
        let mut stdin = out_target.take_stdin().unwrap();

        let mut frame_counter = 0u64;
        // main loop
        let init_time = Instant::now();

        #[cfg(feature = "hyperDebug")]
        helper_functions::parse_debug("Beginning of Video Mainloop", file!(), line!());
        #[cfg(feature = "hyperDebug")]
        for i in self.videos.iter() {
            println!("Video Stats: {}", i.videos.len());
        }

        #[allow(unused_assignments)]
        let mut t_last = Instant::now();
        let mut t_now = Instant::now();
        let fps_u64 = self.video_sizer.fps.clone() as u64;
        let frames_between_update = fps_u64 * 30;
        let f64_frames_between_update = frames_between_update as f64;
        'mainloop: loop {
            // print data about video export
            frame_counter += 1;
            if frame_counter.rem(frames_between_update) == 0 {
                (t_last, t_now) = (t_now, Instant::now());
                let fps = f64_frames_between_update / (t_now - t_last).as_secs_f64();
                eprintln!(
                    "Frame: {} - Video Length {} - FPS {:.2}",
                    frame_counter,
                    seconds_to_hhmmss(frame_counter / fps_u64),
                    fps
                )
            }
            // collect set of frames to process
            #[cfg(feature = "hyperDebug")]
            helper_functions::parse_debug("Frame Prep Started for frame", file!(), line!());
            let frame_prep: Vec<Option<OutputVideoFrame>> = self
                .videos
                .iter_mut()
                .map(|x| x.next_frame(&frame_counter))
                .collect();

            // if no more frames are available break the loop
            if frame_prep.iter().any(|f| f.is_none()) {
                for (fp, vl) in frame_prep.iter().zip(self.videos.iter()) {
                    match fp {
                        None => {
                            println!("video: {}", vl.complete_videos.iter().last().unwrap().src.to_str().unwrap());
                        }
                        Some(_) => {}
                    }
                }
                #[cfg(feature = "hyperDebug")]
                helper_functions::parse_debug(
                    "Frame prep returned a none value, mainloop ended",
                    file!(), line!());
                break 'mainloop;
            }

            // unwrap frames into frame joiner format
            let frame_prep: Vec<OutputVideoFrame> =
                frame_prep.into_iter().map(|x1| x1.unwrap()).collect();

            // join frames from separate items using `FrameShape` 
            let frames = self.shape_style.frame_joiner(frame_prep, &self.video_sizer);
            #[cfg(feature = "hyperDebug")]
            println!("Read: {} bytes", frames.len());

            #[cfg(feature = "hyperDebug")]
            helper_functions::parse_debug("main_loop: frame_write", file!(), line!());
            // write to the buffer
            let written = stdin.write(&frames);
            // check that write worked
            match written {
                Ok(_c) => {
                    #[cfg(feature = "hyperDebug")]
                    helper_functions::parse_debug("main_loop: frames_joined", file!(), line!());
                    #[cfg(feature = "hyperDebug")]
                    println!("{} bytes written to Ffmpeg", _c);
                }
                Err(x) => {
                    #[cfg(feature = "hyperDebug")]
                    helper_functions::parse_debug(
                        "main_loop: frames_joined_failure",
                        file!(),
                        line!(),
                    );
                    eprintln!("main_loop: frames_joined_failure {:?}", x);
                    println!("ErrorKind = {:?}", x.kind());
                    println!("HELP: \n\tThis is usually because of an incorrectly configured video encoder. (see settings.ini)");
                    println!("\tIt could also be becasue ffmpeg cannot access the output file specified");
                    panic!("Un recoverable error in ffmpeg pipe")
                }
            }
            #[cfg(feature = "hyperDebug")]
            helper_functions::parse_debug("main_loop: frames_written", file!(), line!());
            ft.frame();
        }

        let min = &frame_counter / (self.video_sizer.fps as u64 * 60);
        let (hour, min) = (&min / 60, &min % 60);
        let fps = frame_counter.clone() as f64 / (Instant::now() - init_time).as_secs_f64();
        eprintln!(
            "END Frame: {} - Video Length {}:{}:00 - FPS {:.2}",
            frame_counter, hour, min, fps
        );
        println!("End of mainloop");

        let _ = out_target.kill();
        println!("kill command sent");
        out_target.wait().expect("Failed to wait on child");
        println!("wait complete");

        temp_out_file
    }

    fn main_loop_audio(&mut self, temp_out_file: &PathBuf, temp_folder: &PathBuf) {
        println!("main_loop_audio started");

        for i in self.videos.iter_mut() {
            i.pop_first_vid();
        }

        #[cfg(feature = "hyperDebug")]
        helper_functions::parse_debug("main_loop_audio", file!(), line!());

        let audio_segments: Vec<Vec<PathBuf>> = self.videos.iter_mut()
            .enumerate()
            .map(|(i, v)| v.cheap_audio_exporter_out_proc(i, &temp_folder))
            .collect();

        println!("Audio segments exported");
        join_audio_video_streams(
            audio_segments,
            &temp_folder,
            &temp_out_file,
            self.output_target.clone(),
            &self.shape_style,
        );

        println!("Audio Complete");
        #[cfg(feature = "hyperDebug")]
        helper_functions::parse_debug("Audio Export: ENDED", file!(), line!());
        let _ = std::fs::remove_file(temp_out_file);
    }
}


pub(crate) trait Joiner {
    fn frame_joiner(&self, frames: Vec<OutputVideoFrame>, out_sh: &VideoEditData) -> Vec<u8>;
}

impl Debug for Video {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Video")
            .field("src", &self.src.to_str().unwrap())
            .field("length_millis", &self.length_millis.unwrap())
            .finish_non_exhaustive()
    }
}

impl Debug for VideoList {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VideoList")
            .field("videos", &self.videos)
            .finish_non_exhaustive()
    }
}

impl Debug for VideoGroup {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VideoGroupTriple")
            .field("videos", &self.videos)
            .finish()
    }
}
