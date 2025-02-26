use crate::frame_shape::FrameShape;

struct FfmAudioText {
    inputs: Vec<char>,
    text: Vec<String>,
}

impl FfmAudioText {
    pub(crate) fn mix_out_with_overlay(self, db: i32) -> String {
        let mut txt = String::new();

        for i in self.text.iter() {
            txt.push_str(i.as_str())
        }

        for i in self.inputs.iter() {
            txt.push('[');
            txt.push(*i);
            txt.push(']');
        };
        txt.push_str(
            &format!("amix=inputs={}[d];[d]loudnorm[d]", self.inputs.len())
        );

        let number = self.inputs.len();
        txt.push_str(
            format!(
                "[d]volume={}dB[d];[{number}:a]anull[x];[d][x]amix=inputs=2[d];[d]loudnorm[d]",
                -db
            ).as_str()
        );

        txt
    }

    fn mix_out(self) -> String {
        // [a][b][c][d]amix=inputs=4[d];[d]loudnorm[d]\
        let mut txt = String::new();

        for i in self.text.iter() {
            txt.push_str(i.as_str())
        }

        for i in self.inputs.iter() {
            txt.push('[');
            txt.push(*i);
            txt.push(']');
        };
        txt.push_str(
            &format!("amix=inputs={}[d];[d]loudnorm[d]", self.inputs.len())
        );
        txt
    }
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
    fn stereo_l(self, db: i32) -> Self {
        self.stereo_bal(BALANCE_LR[0]).surround(SUR_L).volume(db)
    }
    fn stereo_lc(self, db: i32) -> Self {
        self.stereo_bal(BALANCE_LR[1]).surround(SUR_C).volume(db)
    }
    fn stereo_c(self, db: i32) -> Self {
        self.stereo_bal(BALANCE_LR[2]).surround(SUR_C).volume(db)
    }
    fn stereo_rc(self, db: i32) -> Self {
        self.stereo_bal(BALANCE_LR[3]).surround(SUR_C).volume(db)
    }
    fn stereo_r(self, db: i32) -> Self {
        self.stereo_bal(BALANCE_LR[4]).surround(SUR_R).volume(db)
    }
}
const BALANCE_LR: [f32; 5] = [0.9, 0.45, 0.0, -0.45, -0.9];
const SUR_L: i32 = -90;
const SUR_C: i32 = 0;
const SUR_R: i32 = 90;


impl FrameShape {
    //noinspection SpellCheckingInspection
    pub(crate) fn audio_args_with_vid(&self) -> String {
        match self {
            FrameShape::Mono => {
                let out = FfmAudioText::new()
                    .add_input()
                    .mix_out();
                out
            }
            FrameShape::Dual => {
                let out = FfmAudioText::new()
                    .add_input().stereo_lc(-0)
                    .add_input().stereo_rc(-0)
                    .mix_out();
                out
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
                let out = FfmAudioText::new()
                    .add_input().stereo_lc(0)
                    .add_input().stereo_rc(0)
                    .add_input().stereo_l(-3)
                    .add_input().stereo_r(-3)
                    .mix_out();
                out
            }
            FrameShape::VertEmph | FrameShape::VertEmph2 => {
                let out = FfmAudioText::new()
                    .add_input().stereo_c(0)
                    .add_input().stereo_l(-3)
                    .add_input().stereo_r(-3)
                    .add_input().stereo_l(-5)
                    .add_input().stereo_r(-5)
                    .mix_out();
                out
            }
            FrameShape::HorizEmph | FrameShape::HorizEmph2 => {
                let out = FfmAudioText::new()
                    .add_input().stereo_l(-3)
                    .add_input().stereo_c(0)
                    .add_input().stereo_c(-5)
                    .add_input().stereo_r(-3)
                    .mix_out();
                out
            }
            FrameShape::SideVert => {
                let out = FfmAudioText::new()
                    .add_input().stereo_l(0)
                    .add_input().stereo_rc(0)
                    .add_input().stereo_rc(-3)
                    .mix_out();
                out
            }
            FrameShape::SideVert2 => {
                let out = FfmAudioText::new()
                    .add_input().stereo_r(0)
                    .add_input().stereo_lc(0)
                    .add_input().stereo_lc(-3)
                    .mix_out();
                out
            }
            FrameShape::CentreEmphVert | FrameShape::CentreEmphVert2 => {
                let out = FfmAudioText::new()
                    .add_input().stereo_c(0)
                    .add_input().stereo_l(-2)
                    .add_input().stereo_r(-2)
                    .add_input().stereo_lc(-5)
                    .add_input().stereo_rc(-5)
                    .mix_out();
                out
            }
            FrameShape::MoreHoriz | FrameShape::MoreHoriz2 => {
                let out = FfmAudioText::new()
                    .add_input().stereo_c(0)
                    .add_input().stereo_l(-2)
                    .add_input().stereo_r(-2)
                    .add_input().stereo_lc(-5)
                    .add_input().stereo_rc(-5)
                    .add_input().stereo_l(-2)
                    .add_input().stereo_r(-2)
                    .mix_out();
                out
            }
            FrameShape::ExtendedLandscape => {
                let out = FfmAudioText::new()
                    .add_input().stereo_c(0)
                    .add_input().stereo_lc(-5)
                    .add_input().stereo_rc(-5)
                    .add_input().stereo_l(-2)
                    .add_input().stereo_r(-2)
                    .add_input().stereo_l(-3)
                    .add_input().stereo_r(-3)
                    .mix_out();
                out
            }
            FrameShape::ExtendedLandscape2 => {
                let out = FfmAudioText::new()
                    .add_input().surround(SUR_C)
                    .add_input().stereo_lc(-4)
                    .add_input().stereo_c(-3)
                    .add_input().stereo_rc(-4)
                    .add_input().stereo_l(-5)
                    .add_input().stereo_r(-5)
                    .add_input().stereo_l(-6)
                    .add_input().stereo_r(-6)
                    .mix_out();
                out
            }
            FrameShape::OffsetVH4x4 => {
                let out = FfmAudioText::new()
                    .add_input().stereo_lc(0)
                    .add_input().stereo_rc(0)
                    .add_input().stereo_l(-2 - 3)
                    .add_input().stereo_r(-2 - 3)
                    .add_input().stereo_l(-2)
                    .add_input().stereo_r(-2)
                    .add_input().stereo_lc(-3)
                    .add_input().stereo_rc(-3)
                    .mix_out();
                out
            }
            FrameShape::TripleMirroredSides => {
                let out = FfmAudioText::new()
                    .add_input().stereo_c(-8)
                    .add_input().stereo_c(-0)
                    .mix_out();
                out
            }
        }
    }
}
impl FrameShape {
    ///
    ///
    /// # Arguments 
    ///
    /// * `string_input`: 
    ///
    /// returns: String 
    ///
    /// # Examples 
    ///
    /// ``` rust
    /// let text1 = FrameShape::Triple.audio_arg_string(Some("l.-1 c.3 r.-1"))
    /// let text2 = FfmAudioText::new()
    ///             .add_input().stereo_l(-1)
    ///             .add_input().stereo_c(3)
    ///             .add_input().stereo_r(-1)
    ///             .mix_out()
    /// assert_eq!(text1, text2)
    ///
    /// ```
    //noinspection SpellCheckingInspection
    pub(crate) fn audio_arg_string(&self, string_input: Option<String>, audio_overlay_db: Option<i32>) -> String {
        let strin = match string_input {
            None => { return self.audio_args_with_vid() }
            Some(s) => { s.to_lowercase() }
        };
        let string_input: Vec<&str> = strin.split(' ').collect();

        assert_eq!(string_input.len(), (self.count() as usize), "Error, target not same length as input string");

        let mut out = FfmAudioText::new();

        for st in string_input.into_iter() {
            let (position_token, delta_token) = st.split_once('.').unwrap();
            let delta = delta_token.parse::<i32>().unwrap();

            let (p0_token, p1_token) = {
                let mut chr = position_token.chars();
                (chr.next().unwrap(), chr.next().unwrap_or('-'))
            };

            match (p0_token, p1_token) {
                ('l', 'c') => { out = out.add_input().stereo_lc(delta) }
                ('r', 'c') => { out = out.add_input().stereo_rc(delta) }
                ('l', _) => { out = out.add_input().stereo_l(delta) }
                ('c', _) => { out = out.add_input().stereo_c(delta) }
                ('r', _) => { out = out.add_input().stereo_r(delta) }
                _ => panic!("Undefined Tokens in audio_arg_string"),
            }
        }
        let outmix = match audio_overlay_db {
            None => { out.mix_out() }
            Some(db) => { out.mix_out_with_overlay(db) }
        };
        print!("outmix from input: {}", outmix);
        outmix
    }
}
