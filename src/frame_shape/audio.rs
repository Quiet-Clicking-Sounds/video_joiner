use crate::frame_shape::FrameShape;

struct FfmAudioText {
    inputs: Vec<char>,
    text: Vec<String>,
}

impl FfmAudioText {
    pub fn new() -> Self {
        FfmAudioText {
            inputs: Vec::new(),
            text: Vec::new(),
        }
    }
    fn add_input(mut self) -> Self {
        let number = self.inputs.len();
        let character = char::from_u32((number + 97) as u32).unwrap();
        self.inputs.push(character);
        self.text.push(format!("[{}:a]anull[{}];", number + 1, character));
        self
    }
    fn stereo_bal(mut self, bal: f32) -> Self {
        let character = self.inputs.last().unwrap();
        self.text.last_mut().unwrap().push_str(
            format!("[{}]stereotools=balance_in=-{}[{}];", character, bal, character).as_str()
        );
        self
    }
    fn surround(mut self, angle: i32) -> Self {
        let character = self.inputs.last().unwrap();
        let angle = if angle > 360 { angle } else if angle < 0 { 360 + angle } else { angle };
        assert!((angle < 360) && (angle > -1), "Error in angles");

        self.text.last_mut().unwrap().push_str(
            format!("[{}]surround=chl_out=stereo:chl_in=stereo:angle={}[{}];", character, angle, character).as_str()
        );
        self
    }
    fn volume(mut self, db: i32) -> Self {
        let character = self.inputs.last().unwrap();
        self.text.last_mut().unwrap().push_str(
            format!("[{}]volume={}dB[{}];", character, db, character).as_str()
        );
        self
    }
    fn mix_out(self) -> String {
        // [a][b][c][d]amix=inputs=4[d];[d]loudnorm[d]\
        let mut txt = String::new();
        
        for i in self.text.iter(){
            txt.push_str(i.as_str())
        }
        
        for i in self.inputs.iter(){
            txt.push('[');
            txt.push(*i);
            txt.push(']');
        };
        txt.push_str(
            &format!("amix=inputs={}[d];[d]loudnorm[d]",self.inputs.len())
        );
        txt
    }
    fn stereo_l(self, db: i32) -> Self {
        self.stereo_bal(BAL_L).surround(SUR_L).volume(db)
    }
    fn stereo_c(self, db: i32) -> Self {
        self.stereo_bal(BAL_C).surround(SUR_C).volume(db)
    }
    fn stereo_r(self, db: i32) -> Self {
        self.stereo_bal(BAL_R).surround(SUR_R).volume(db)
    }
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
                let out = FfmAudioText::new()
                    .add_input().stereo_l(-3)
                    .add_input().stereo_c(0)
                    .add_input().stereo_r(-3)
                    .mix_out();
                out
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
            FrameShape::CentreEmphVert | FrameShape::CentreEmphVert2 => {
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
            FrameShape::MoreHoriz | FrameShape::MoreHoriz2 => {
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
            FrameShape::ExtendedLandscape => {
                "\
                [1:a]surround=chl_out=stereo:chl_in=stereo:angle=0[a];\
                [2:a]stereotools=balance_in=-0.2[x];[x]surround=chl_out=stereo:chl_in=stereo:angle=90[x];[x]volume=-3dB[b];\
                [3:a]stereotools=balance_in=0.2[x];[x]surround=chl_out=stereo:chl_in=stereo:angle=270[x];[x]volume=-3dB[c];\
                [4:a]stereotools=balance_in=-0.4[x];[x]surround=chl_out=stereo:chl_in=stereo:angle=90[x];[x]volume=-5dB[d];\
                [5:a]stereotools=balance_in=0.4[x];[x]surround=chl_out=stereo:chl_in=stereo:angle=270[x];[x]volume=-5dB[e];\
                [6:a]stereotools=balance_in=-0.4[x];[x]surround=chl_out=stereo:chl_in=stereo:angle=90[x];[x]volume=-6dB[f];\
                [7:a]stereotools=balance_in=0.4[x];[x]surround=chl_out=stereo:chl_in=stereo:angle=270[x];[x]volume=-6dB[g];\
                [8:a]stereotools=balance_in=-0.4[x];[x]surround=chl_out=stereo:chl_in=stereo:angle=90[x];[x]volume=-7dB[h];\
                [9:a]stereotools=balance_in=0.4[x];[x]surround=chl_out=stereo:chl_in=stereo:angle=270[x];[x]volume=-7dB[i];\
                [a][b][c][d][e][f][g][h][i]amix=inputs=9[d];[d]loudnorm[d]\
                ".to_string()
            }
            FrameShape::ExtendedLandscape2 => {
                let out = FfmAudioText::new()
                    .add_input().surround(SUR_C)
                    .add_input().stereo_l(-4)
                    .add_input().stereo_c(-3)
                    .add_input().stereo_r(-4)
                    .add_input().stereo_l(-5)
                    .add_input().stereo_r(-5)
                    .add_input().stereo_l(-6)
                    .add_input().stereo_r(-6)
                    .mix_out();
                out
            }
        }
    }
}
const BAL_L:f32 = -0.4;
const BAL_C:f32 = 0.0;
const BAL_R:f32 = 0.4;
const SUR_L:i32  = -90;
const SUR_C:i32 = 0;
const SUR_R:i32 = 90;
