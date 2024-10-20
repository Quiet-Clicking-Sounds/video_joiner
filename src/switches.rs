use crate::video::{Joiner, Video, VideoEditData};
use ffmpeg_sidecar::event::OutputVideoFrame;
use rand::prelude::SliceRandom;
use rand::rngs::StdRng;
use rand::thread_rng;
use std::ops::Rem;
use std::slice::ChunksExact;

use rand::SeedableRng;

#[derive(Clone, Debug)]
pub(crate) enum SortOrder {
    Random,
    RandomSeeded(u64),
    RandomSeededR(u64),
    ShortestFirst,
    LongestFirst,
    RandomWithLargestLast,
}
impl SortOrder {
    pub(crate) fn apply_sort(&self, mut videos: Vec<Video>) -> Vec<Video> {
        // if SortOrder needs the length we make sure it exists here first
        match self {
            SortOrder::ShortestFirst | SortOrder::LongestFirst |
            SortOrder::RandomWithLargestLast => {
                videos.iter_mut().for_each(|f| { let _ = f.get_length(); });
            }
            _ => {}
        }
        match self {
            SortOrder::Random => {
                let mut rng = thread_rng();
                videos.shuffle(&mut rng);
                videos
            }
            SortOrder::RandomSeeded(s) => {
                let mut rng = <StdRng as SeedableRng>::seed_from_u64(s.clone());
                videos.shuffle(&mut rng);
                videos
            }
            SortOrder::RandomSeededR(s) => {
                let mut rng = <StdRng as SeedableRng>::seed_from_u64(s.clone());
                videos.shuffle(&mut rng);
                videos.reverse();
                videos
            }
            SortOrder::ShortestFirst => {
                videos.sort_unstable_by_key(|f| f.length_millis.unwrap_or(0));
                videos
            }
            SortOrder::LongestFirst => {
                videos.sort_unstable_by_key(|f| 0 - f.length_millis.unwrap_or(0));
                videos
            }
            SortOrder::RandomWithLargestLast => {
                let mut rng = thread_rng();
                videos.shuffle(&mut rng);
                let longest = videos.iter().max_by_key(|x| x.length_millis.unwrap_or(0)).unwrap();
                let v_longest = videos.remove(
                    videos.iter().position(|v| std::ptr::addr_eq(v, longest)).unwrap()
                );
                videos.push(v_longest);
                videos
            }
        }
    }
}


#[derive(Clone, Debug)]
pub(crate) enum FrameShape {
    /// see [readme_data/frame_shapes_1.svg](../readme_data/frame_shapes_1.svg) for shape ref
    Dual,
    /// see [readme_data/frame_shapes_2.svg](../readme_data/frame_shapes_2.svg) for shape ref
    Triple,
    /// see [readme_data/frame_shapes_3.svg](../readme_data/frame_shapes_3.svg) for shape ref
    Quad,
    /// see [readme_data/frame_shapes_4.svg](../readme_data/frame_shapes_4.svg) for shape ref
    VertEmph,
    /// see [readme_data/frame_shapes_5.svg](../readme_data/frame_shapes_5.svg) for shape ref
    HorizEmph,
    /// see [readme_data/frame_shapes_6.svg](../readme_data/frame_shapes_6.svg) for shape ref
    VertEmph2,
    /// see [readme_data/frame_shapes_6.svg](../readme_data/frame_shapes_6.svg) for shape ref
    HorizEmph2,
    /// see [readme_data/frame_shapes_7.svg](../readme_data/frame_shapes_7.svg) for shape ref
    SideVert,
    /// see [readme_data/frame_shapes_7.svg](../readme_data/frame_shapes_7.svg) for shape ref
    SideVert2,
    /// see [readme_data/frame_shapes_8.svg](../readme_data/frame_shapes_8.svg) for shape ref
    CentreEmphVert,
    /// see [readme_data/frame_shapes_8v.svg](../readme_data/frame_shapes_8v.svg) for shape ref
    CentreEmphVert2,
    /// see [readme_data/frame_shapes_9.svg](../readme_data/frame_shapes_8v.svg) for shape ref
    MoreHoriz,
    /// see [readme_data/frame_shapes_9v.svg](../readme_data/frame_shapes_8v.svg) for shape ref
    MoreHoriz2,
}

impl FrameShape {
    //noinspection SpellCheckingInspection
    pub(crate) fn audio_args_with_vid(&self) -> String {
        match self {
            FrameShape::Dual => {
                "\
                [1:a]stereotools=balance_in=-0.4[a];[a]surround=chl_out=stereo:chl_in=stereo:angle=270[a];\
                [2:a]stereotools=balance_in=0.4[b];[b]surround=chl_out=stereo:chl_in=stereo:angle=90[b];\
                [a][b]amix=inputs=2[d];[d]loudnorm[d]\
                ".to_string()
            }
            FrameShape::Triple => {
                "\
                [1:a]stereotools=balance_in=-0.4[a];[a]surround=chl_out=stereo:chl_in=stereo:angle=270[a];\
                [2:a]surround=chl_out=stereo:chl_in=stereo:angle=0[b];\
                [3:a]stereotools=balance_in=0.4[c];[c]surround=chl_out=stereo:chl_in=stereo:angle=90[c];\
                [a][b][c]amix=inputs=3[d];[d]loudnorm[d]\
                ".to_string()
            }
            FrameShape::Quad => {
                "\
                [1:a]surround=chl_out=stereo:chl_in=stereo:angle=315[a];\
                [2:a]surround=chl_out=stereo:chl_in=stereo:angle=45[b];\
                [3:a]stereotools=balance_in=-0.4[c];[c]surround=chl_out=stereo:chl_in=stereo:angle=270[c];\
                [4:a]stereotools=balance_in=0.4[d];[d]surround=chl_out=stereo:chl_in=stereo:angle=90[d];\
                [a][b][c][d]amix=inputs=4[d];[d]loudnorm[d]\
                ".to_string()
            }
            FrameShape::VertEmph | FrameShape::VertEmph2 => {
                "\
                [1:a]surround=chl_out=stereo:chl_in=stereo:angle=0[a];\
                [2:a]surround=chl_out=stereo:chl_in=stereo:angle=315[b];\
                [3:a]surround=chl_out=stereo:chl_in=stereo:angle=45[c];\
                [4:a]stereotools=balance_in=-0.4[d];[d]surround=chl_out=stereo:chl_in=stereo:angle=270[d];\
                [5:a]stereotools=balance_in=0.4[e];[e]surround=chl_out=stereo:chl_in=stereo:angle=90[e];\
                [b]volume=-5dB[b];[c]volume=-5dB[c];[d]volume=-8dB[d];[e]volume=-8dB[e];\
                [a][b][c][d][e]amix=inputs=5[d];[d]loudnorm[d]\
                ".to_string()
            }
            FrameShape::HorizEmph | FrameShape::HorizEmph2 => {
                "\
                [1:a]stereotools=balance_in=-0.4[a];[a]surround=chl_out=stereo:chl_in=stereo:angle=270[a];\
                [2:a]surround=chl_out=stereo:chl_in=stereo:angle=0[b];\
                [3:a]surround=chl_out=stereo:chl_in=stereo:angle=0[c];[c]volume=-2dB[c];\
                [4:a]stereotools=balance_in=0.4[d];[d]surround=chl_out=stereo:chl_in=stereo:angle=90[d];\
                [a]volume=-5dB[a];[d]volume=-5dB[d];\
                [a][b][c][d]amix=inputs=4[d];[d]loudnorm[d]\
                ".to_string()
            }
            FrameShape::SideVert => {
                "\
                [1:a]stereotools=balance_in=-0.5[a];[a]surround=chl_out=stereo:chl_in=stereo:angle=270[a];\
                [2:a]stereotools=balance_in=0.1[b];[b]surround=chl_out=stereo:chl_in=stereo:angle=20[b];\
                [3:a]stereotools=balance_in=0.1[c];[c]surround=chl_out=stereo:chl_in=stereo:angle=20[c];\
                [c]volume=-5dB[c];\
                [a][b][c]amix=inputs=3[d];[d]loudnorm[d]\
                ".to_string()
            }
            FrameShape::SideVert2 => {
                "\
                [1:a]stereotools=balance_in=0.5[a];[a]surround=chl_out=stereo:chl_in=stereo:angle=90[a];\
                [2:a]stereotools=balance_in=-0.1[b];[b]surround=chl_out=stereo:chl_in=stereo:angle=340[b];\
                [3:a]stereotools=balance_in=-0.1[c];[c]surround=chl_out=stereo:chl_in=stereo:angle=340[c];\
                [c]volume=-5dB[c];\
                [a][b][c]amix=inputs=3[d];[d]loudnorm[d]\
                ".to_string()
            }
            FrameShape::CentreEmphVert|FrameShape::CentreEmphVert2 => {
                "\
                [2:a]stereotools=balance_in=-0.4[a];[a]surround=chl_out=stereo:chl_in=stereo:angle=270[a];\
                [1:a]surround=chl_out=stereo:chl_in=stereo:angle=0[b];\
                [3:a]stereotools=balance_in=0.4[c];[c]surround=chl_out=stereo:chl_in=stereo:angle=90[c];\
                [4:a]stereotools=balance_in=0.4[d];[d]surround=chl_out=stereo:chl_in=stereo:angle=270[d];\
                [5:a]stereotools=balance_in=0.4[e];[e]surround=chl_out=stereo:chl_in=stereo:angle=90[e];\
                [a]volume=-2dB[a];[c]volume=-2dB[c];[d]volume=-5dB[d];[e]volume=-5dB[e];\
                [a][b][c][d][e]amix=inputs=5[d];[d]loudnorm[d]\
                ".to_string()
            }
            FrameShape::MoreHoriz|FrameShape::MoreHoriz2 => {
                "\
                [6:a]stereotools=balance_in=-0.4[x];[x]surround=chl_out=stereo:chl_in=stereo:angle=90[x];[x]volume=-5dB[b];\
                [2:a]stereotools=balance_in=-0.6[x];[x]surround=chl_out=stereo:chl_in=stereo:angle=90[x];[x]volume=-2dB[c];\
                [1:a]surround=chl_out=stereo:chl_in=stereo:angle=0[a];\
                [4:a]stereotools=balance_in=-0.2[x];[x]surround=chl_out=stereo:chl_in=stereo:angle=90[x];[x]volume=-5dB[d];\
                [5:a]stereotools=balance_in=-0.2[x];[x]surround=chl_out=stereo:chl_in=stereo:angle=270[x];[x]volume=-5dB[e];\
                [7:a]stereotools=balance_in=-0.4[x];[x]surround=chl_out=stereo:chl_in=stereo:angle=270[x];[x]volume=-5dB[f];\
                [3:a]stereotools=balance_in=-0.6[x];[x]surround=chl_out=stereo:chl_in=stereo:angle=270[x];[x]volume=-2dB[g];\
                [a][b][c][d][e][f][g]amix=inputs=7[d];[d]loudnorm[d]\
                ".to_string()
            }
        }
    }
    pub(crate) fn count(&self) -> u32 {
        match self {
            FrameShape::Dual => 2,
            FrameShape::Triple => 3,
            FrameShape::Quad => 4,
            FrameShape::VertEmph | FrameShape::VertEmph2 | 
            FrameShape::CentreEmphVert| FrameShape::CentreEmphVert2 => 5,
            FrameShape::HorizEmph | FrameShape::HorizEmph2 => 4,
            FrameShape::SideVert | FrameShape::SideVert2 => 3,
            FrameShape::MoreHoriz|FrameShape::MoreHoriz2 => 7
        }
    }
}

impl Joiner for FrameShape {
    
    /// join methods for a single frame.
    /// 
    /// new frame shapes must be implemented here
    /// 
    /// # Arguments 
    /// 
    /// * `frames`: list of frames from each video input
    /// * `out_sh`: shape data
    /// 
    /// returns: Vec<u8, Global> 
    /// 
    fn frame_joiner(&self, mut frames: Vec<OutputVideoFrame>, out_sh: &VideoEditData) -> Vec<u8> {
        let mut out =
            Vec::with_capacity((&out_sh.output_height * &out_sh.output_width * 3) as usize);

        let mut chunks: Vec<ChunksExact<u8>> = frames
            .iter_mut()
            .map(
                |x|
                    x.data
                        .chunks_exact(x.width as usize * 3)
                        .into_iter()
            )
            .collect();
        match self {
            FrameShape::Dual | FrameShape::Triple => {
                'outter: loop {
                    for chunk in chunks.iter_mut() {
                        match chunk.next() {
                            None => break 'outter,
                            Some(ch) => out.extend_from_slice(ch),
                        }
                    }
                }
            }
            FrameShape::Quad => {
                assert_eq!(chunks.len().rem(2), 0);
                'outter: loop {
                    for chunk in chunks[0..2].iter_mut() {
                        match chunk.next() {
                            None => break 'outter,
                            Some(ch) => out.extend_from_slice(ch),
                        }
                    }
                }
                'outter: loop {
                    for chunk in chunks[2..4].iter_mut() {
                        match chunk.next() {
                            None => break 'outter,
                            Some(ch) => out.extend_from_slice(ch),
                        }
                    }
                }
            }
            FrameShape::VertEmph | FrameShape::VertEmph2 => {
                'outter: loop {
                    match chunks[1].next() {
                        None => break 'outter,
                        Some(ch) => out.extend_from_slice(ch),
                    }
                    match chunks[0].next() {
                        None => break 'outter,
                        Some(ch) => out.extend_from_slice(ch),
                    }
                    match chunks[2].next() {
                        None => break 'outter,
                        Some(ch) => out.extend_from_slice(ch),
                    }
                }
                'outter: loop {
                    match chunks[3].next() {
                        None => break 'outter,
                        Some(ch) => out.extend_from_slice(ch),
                    }
                    match chunks[0].next() {
                        None => break 'outter,
                        Some(ch) => out.extend_from_slice(ch),
                    }
                    match chunks[4].next() {
                        None => break 'outter,
                        Some(ch) => out.extend_from_slice(ch),
                    }
                }
            }
            FrameShape::HorizEmph | FrameShape::HorizEmph2 => {
                let mut switch: bool = true;
                'outter: loop {
                    match chunks[0].next() {
                        None => break 'outter,
                        Some(ch) => out.extend_from_slice(ch),
                    }
                    if switch {
                        match chunks[1].next() {
                            None => { switch = false }
                            Some(ch) => out.extend_from_slice(ch),
                        }
                    } 
                    // Important note: Do not change this to an else statement
                    // if switch is None !switch must happen
                    if !switch  {
                        match chunks[2].next() {
                            None => break 'outter,
                            Some(ch) => out.extend_from_slice(ch),
                        }
                    }
                    match chunks[3].next() {
                        None => break 'outter,
                        Some(ch) => out.extend_from_slice(ch),
                    }
                }
            }
            FrameShape::SideVert => {
                let mut switch: bool = true;
                'outter: loop {
                    match chunks[0].next() {
                        None => break 'outter,
                        Some(ch) => out.extend_from_slice(ch),
                    }
                    if switch {
                        match chunks[1].next() {
                            None => { switch = false }
                            Some(ch) => out.extend_from_slice(ch),
                        }
                    } 
                    // Important note: Do not change this to an else statement
                    // if switch is None !switch must happen
                    if !switch  {
                        match chunks[2].next() {
                            None => break 'outter,
                            Some(ch) => out.extend_from_slice(ch),
                        }
                    }
                }
            }
            FrameShape::SideVert2 => {
                let mut switch: bool = true;
                'outter: loop {
                    if switch {
                        match chunks[1].next() {
                            None => { switch = false }
                            Some(ch) => out.extend_from_slice(ch),
                        }
                    } 
                    // Important note: Do not change this to an else statement
                    // if switch is None !switch must happen
                    if !switch {
                        match chunks[2].next() {
                            None => break 'outter,
                            Some(ch) => out.extend_from_slice(ch),
                        }
                    }
                    match chunks[0].next() {
                        None => break 'outter,
                        Some(ch) => out.extend_from_slice(ch),
                    }
                }
            }
            FrameShape::CentreEmphVert => {
                let mut switch: bool = true;
                'outter: loop {
                    // this is a full height item, always iter over it
                    match chunks[1].next() {
                            None => break 'outter, 
                            Some(ch) => out.extend_from_slice(ch),
                    }
                    // the top part and bottom part are diferent, once the top has been consumed
                    // we need to use the bottom part
                    if switch {
                        match chunks[0].next() {
                            None => { switch = false }
                            Some(ch) => out.extend_from_slice(ch),
                        }
                    } 
                    // Important note: Do not change this to an else statement
                    // if switch is None !switch must happen
                    if !switch  { 
                        match chunks[3].next() {
                            None => break 'outter,
                            Some(ch) => out.extend_from_slice(ch),
                        }
                        match chunks[4].next() {
                            None => break 'outter,
                            Some(ch) => out.extend_from_slice(ch),
                        }
                    }
                    // this is a full height item, always iter over it
                    match chunks[2].next() {
                            None => break 'outter, 
                            Some(ch) => out.extend_from_slice(ch),
                    }
                }
            }
            FrameShape::CentreEmphVert2 => {
                let mut switch: bool = true;
                'outter: loop {
                    // this is a full height item, always iter over it
                    match chunks[1].next() {
                            None => break 'outter, 
                            Some(ch) => out.extend_from_slice(ch),
                    }
                    // the top part and bottom part are diferent, once the top has been consumed
                    // we need to use the bottom part
                    if switch {
                        match chunks[3].next() {
                            None => { switch = false }
                            Some(ch) => out.extend_from_slice(ch),
                        }
                        match chunks[4].next() {
                            None => { switch = false }
                            Some(ch) => out.extend_from_slice(ch),
                        }
                    } 
                    // Important note: Do not change this to an else statement
                    // if switch is None !switch must happen
                    if !switch  { 
                        match chunks[0].next() {
                            None => break 'outter,
                            Some(ch) => out.extend_from_slice(ch),
                        }
                    }
                    // this is a full height item, always iter over it
                    match chunks[2].next() {
                            None => break 'outter, 
                            Some(ch) => out.extend_from_slice(ch),
                    }
                }
            }
            FrameShape::MoreHoriz => {
                let mut switch1: bool = true;
                let mut switch2: bool = true;
                let mut switch3: bool = true;
                loop {
                    if switch1 {
                        match chunks[5].next() { 
                            None => switch1=false,
                            Some(ch) => out.extend_from_slice(ch)
                        }
                    }
                    if !switch1 {
                        match chunks[1].next() { 
                            None => break,
                            Some(ch) => out.extend_from_slice(ch)
                        }
                    }
                    
                    if switch2 {
                        match chunks[0].next() { 
                            None => switch2=false,
                            Some(ch) => out.extend_from_slice(ch)
                        }
                    }
                    if !switch2 {
                        match chunks[3].next() { 
                            None => break,
                            Some(ch) => out.extend_from_slice(ch)
                        }
                        match chunks[4].next() { 
                            None => break,
                            Some(ch) => out.extend_from_slice(ch)
                        }
                    }
                    
                    if switch3 {
                        match chunks[6].next() { 
                            None => switch3=false,
                            Some(ch) => out.extend_from_slice(ch)
                        }
                    }
                    if !switch3 {
                        match chunks[2].next() { 
                            None => break,
                            Some(ch) => out.extend_from_slice(ch)
                        }
                    }
                }
            }
            FrameShape::MoreHoriz2 => {
                let mut switch1: bool = true;
                let mut switch2: bool = true;
                let mut switch3: bool = true;
                loop {
                    if switch1 {
                        match chunks[1].next() { 
                            None => switch1=false,
                            Some(ch) => out.extend_from_slice(ch)
                        }
                    }
                    if !switch1 {
                        match chunks[5].next() { 
                            None => break,
                            Some(ch) => out.extend_from_slice(ch)
                        }
                    }
                    
                    if switch2 {
                        match chunks[3].next() { 
                            None => switch2=false,
                            Some(ch) => out.extend_from_slice(ch)
                        }
                        match chunks[4].next() { 
                            None => switch2=false,
                            Some(ch) => out.extend_from_slice(ch)
                        }
                    }
                    if !switch2 {
                        match chunks[0].next() { 
                            None => break,
                            Some(ch) => out.extend_from_slice(ch)
                        }
                    }
                    
                    if switch3 {
                        match chunks[2].next() { 
                            None => switch3=false,
                            Some(ch) => out.extend_from_slice(ch)
                        }
                    }
                    if !switch3 {
                        match chunks[6].next() { 
                            None => break,
                            Some(ch) => out.extend_from_slice(ch)
                        }
                    }
                }
            }
        }

        out
    }
}
