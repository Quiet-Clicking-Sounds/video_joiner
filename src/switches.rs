use crate::video::{Joiner, VideoEditData};
use ffmpeg_sidecar::event::OutputVideoFrame;
use std::ops::Rem;
use std::slice::ChunksExact;

#[derive(Clone, Debug)]
pub(crate) enum FrameShape {
    Dual,
    Triple,
    Quad,
    VertEmph,
    HorizEmph,
    VertEmph2,
    HorizEmph2,
    SideVert,
    SideVert2,
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
        }
    }
    pub(crate) fn count(&self) -> u32 {
        match self {
            FrameShape::Dual => 2,
            FrameShape::Triple => 3,
            FrameShape::Quad => 4,
            FrameShape::VertEmph | FrameShape::VertEmph2 => 5,
            FrameShape::HorizEmph | FrameShape::HorizEmph2 => 4,
            FrameShape::SideVert | FrameShape::SideVert2 => 3,
        }
    }
}

impl Joiner for FrameShape {
    //noinspection SpellCheckingInspection
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
        }

        out
    }
}