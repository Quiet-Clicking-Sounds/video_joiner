use ffmpeg_sidecar::event::OutputVideoFrame;
use std::slice::ChunksExact;
use std::ops::Rem;
use crate::frame_shape::FrameShape;
use crate::video::{Joiner, VideoEditData};

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
                    if !switch {
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
                    if !switch {
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
                    if !switch {
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
                    if !switch {
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
                            None => switch1 = false,
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
                            None => switch2 = false,
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
                            None => switch3 = false,
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
                            None => switch1 = false,
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
                            None => switch2 = false,
                            Some(ch) => out.extend_from_slice(ch)
                        }
                        match chunks[4].next() {
                            None => switch2 = false,
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
                            None => switch3 = false,
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
            FrameShape::ExtendedLandscape => {
                let mut switch1: u8 = 0u8;
                let mut switch2: bool = true;
                let mut switch3: u8 = 0u8;

                loop {

                    if switch1 == 0{
                        match chunks[3].next() {
                            None => switch1 += 1,
                            Some(ch) => out.extend_from_slice(ch)
                        }
                    }
                    if switch1 == 1{
                        match chunks[5].next() {
                            None => switch1 += 1,
                            Some(ch) => out.extend_from_slice(ch)
                        }
                    }
                    if switch1 == 2{
                        match chunks[7].next() {
                            None => break,
                            Some(ch) => out.extend_from_slice(ch)
                        }
                    }


                    if switch2 {
                        match chunks[0].next() {
                            None => switch2 = false,
                            Some(ch) => out.extend_from_slice(ch)
                        }
                    }
                    if !switch2 {
                        match chunks[1].next() {
                            None => break,
                            Some(ch) => out.extend_from_slice(ch)
                        }
                        match chunks[2].next() {
                            None => break,
                            Some(ch) => out.extend_from_slice(ch)
                        }
                    }

                    if switch3 == 0{
                        match chunks[4].next() {
                            None => switch3 += 1,
                            Some(ch) => out.extend_from_slice(ch)
                        }
                    }
                    if switch3 == 1{
                        match chunks[6].next() {
                            None => switch3 += 1,
                            Some(ch) => out.extend_from_slice(ch)
                        }
                    }
                    if switch3 == 2{
                        match chunks[8].next() {
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