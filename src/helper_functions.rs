use std::fmt::{Debug, Formatter};

use crate::{group_split, group_splitter};
use crate::frame_shape::FrameShape;
use crate::video::Video;
use ffmpeg_sidecar::child::FfmpegChild;
use ffmpeg_sidecar::event::{FfmpegEvent, LogLevel};
use std::path::PathBuf;
use std::str::FromStr;
use std::time;


#[cfg(target_os = "windows")]
const PATH_DELIMITER: char = '|';

#[cfg(not(target_os = "windows"))]
const PATH_DELIMITER: char = '/';

pub fn iter_ffmpeg_events(child: &mut FfmpegChild) {
    for i in child.iter().unwrap() {
        match i {
            FfmpegEvent::Log(level, data) => {
                match level {
                    LogLevel::Info => {
                        //eprintln!("LOG INFO {:?} : {}", level, data)
                    }
                    LogLevel::Error | LogLevel::Fatal => {
                        eprintln!("LOG {:?} : {} ", level, data);
                        // return None;
                    }
                    LogLevel::Warning => {
                        eprintln!("LOG {:?} : {} ", level, data);
                        // return None;
                    }
                    LogLevel::Unknown => {
                        if data.len() > 2 {
                            eprintln!("LOG {:?} : {}", level, data)
                        }
                    }
                }
            }
            _ => {}
        };
    }
}


pub struct FrameTimer {
    _init_: time::Instant,
    pub(crate) _frame_count_: u128,
}

impl FrameTimer {
    pub fn new() -> FrameTimer {
        FrameTimer {
            _init_: time::Instant::now(),
            _frame_count_: 0,
        }
    }

    pub fn frame(&mut self) {
        self._frame_count_ += 1
    }

    pub fn fps(&self) -> f64 {
        self._frame_count_.clone() as f64 / self._init_.elapsed().as_secs_f64()
    }
}

impl Debug for FrameTimer {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FrameTimer")
            .field("Frame count", &self._frame_count_)
            .field("Elapsed Time", &self._init_.elapsed())
            .field("Compute FPS", &format!("{:.2}", self.fps()))
            .finish()
    }
}

pub fn seconds_to_hhmmss(sec: u64) -> String {
    let (min, sec) = (&sec / 60, &sec % 60);
    let (hour, min) = (&min / 60, &min % 60);
    format!("{:02}-{:02}-{:02}", hour, min, sec)
}

#[derive(Clone, Debug)]
pub(crate) struct MultiPathBuf {
    inner: Vec<PathBuf>,
}

impl FromStr for MultiPathBuf {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let data = if s.contains(PATH_DELIMITER) {
            s.split(PATH_DELIMITER).map(|f| { PathBuf::from(f) }).collect()
        } else {
            vec![PathBuf::from(s)]
        };
        Ok(
            MultiPathBuf {
                inner: data,
            }
        )
    }
}

impl MultiPathBuf {
    pub fn read_dir(&self) -> Vec<PathBuf> {
        let mut out = vec![];
        for f in self.inner.iter(){
            if f.is_file() {
                out.push(f.clone());
                continue
            }
            let o1 =  f.read_dir()
                .unwrap()
                .into_iter()
                .map(|p| p.unwrap().path());
            out.extend(o1)
        }
        out
    }
    fn is_dir(&self) -> bool {
        self.inner.iter().all(|a| a.is_dir() | a.is_file())
    }
}

impl Into<MultiPathBuf> for &MultiPathBuf {
    fn into(self) -> MultiPathBuf {
        MultiPathBuf {
            inner: self.inner.clone()
        }
    }
}


#[cfg(feature = "hyperDebug")]
#[inline]
pub fn parse_debug(_text: &str, _f: &str, _l: u32) {
    eprintln!("{}:{} - {}", _f, _l, _text);
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn frame_timer_counter_check() {
        let mut ft = FrameTimer::new();
        let iters = 20;
        for _ in 0..iters { ft.frame(); }
        assert_eq!(ft._frame_count_, iters);
    }


    #[test]
    fn frame_timer_time_check() {
        let float_duration = 1.5;
        let duration = Duration::from_secs_f64(float_duration);
        let frame_timer = FrameTimer::new();
        thread::sleep(duration);
        let frame_timer_duration = frame_timer._init_.elapsed().as_secs_f64();
        assert!((&float_duration - frame_timer_duration).abs() < 0.1);
    }
}

#[allow(unused)]
#[cfg(not(feature = "hyperDebug"))]
#[inline]
fn parse_debug(text: &str, f: &str, l: u32) {}

fn scan_dir_for_videos_with_len(dir: impl Into<MultiPathBuf>) -> Vec<(i64, Video)> {
    let mut all_videos = Vec::new();
    for i in dir.into().read_dir() {
        if i.is_file() {
            // setup vid items
            let mut vd = Video::from_path(i.as_path());
            let le = match vd.get_length() {
                Ok(le) => le,
                Err(_) => {
                    println!("Failed to get video Info for: {:?}", vd.src);
                    continue;
                }
            };
            all_videos.push((le, vd))
        }
    }
    all_videos
}

pub fn scan_dir_for_videos(dir: impl Into<MultiPathBuf>) -> Vec<Video> {
    let mut all_videos = Vec::new();
    for i in dir.into().read_dir() {
        if i.is_file() {
            // setup vid items
            let vd = Video::from_path(i.as_path());
            all_videos.push(vd);
        };
    }
    all_videos
}

pub fn video_group_swap(src: impl Into<MultiPathBuf>, screens: FrameShape) -> Vec<Vec<Video>> {
    let src = src.into();
    assert!(src.is_dir(), "Given Input Directory Does Not Exist"); // not my fault
    let all_videos = scan_dir_for_videos_with_len(src);
    let list_grp = group_split::ItemList::new_with_data(all_videos, screens.clone());
    list_grp.run_automatic_swaps();
    list_grp.export_to_data_lists()
}
pub fn video_group_swap_n(src: impl Into<MultiPathBuf>, groups:usize) -> Vec<Vec<Video>> {
    let src = src.into();
    assert!(src.is_dir(), "Given Input Directory Does Not Exist"); // not my fault
    let all_videos = scan_dir_for_videos_with_len(src);
    let list_grp = group_splitter::regrouper(
        all_videos.into_iter().map(|(_,b)|b).collect(),
        groups);
    print!("groups started: ");
    for  vgroup in list_grp.iter(){
        let mut grplen = 0;
        for i in vgroup.iter(){
            grplen += i.length_millis.unwrap()
        }
        print!(" {} |", seconds_to_hhmmss((grplen/1000) as u64));
    }
    println!("\n Regrouping Complete");
    list_grp
}