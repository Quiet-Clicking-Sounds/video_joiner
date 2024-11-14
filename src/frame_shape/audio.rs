use crate::frame_shape::FrameShape;

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
                "\
                [1:a]surround=chl_out=stereo:chl_in=stereo:angle=0[a];\
                [2:a]stereotools=balance_in=-0.4[x];[x]surround=chl_out=stereo:chl_in=stereo:angle=270[x];[x]volume=-3dB[b];\
                [3:a]surround=chl_out=stereo:chl_in=stereo:angle=0[x];[x]volume=-3dB[c];\
                [4:a]stereotools=balance_in=0.4[x];[x]surround=chl_out=stereo:chl_in=stereo:angle=90[x];[x]volume=-3dB[d];\
                [5:a]stereotools=balance_in=0.4[x];[x]surround=chl_out=stereo:chl_in=stereo:angle=270[x];[x]volume=-5dB[e];\
                [6:a]stereotools=balance_in=0.4[x];[x]surround=chl_out=stereo:chl_in=stereo:angle=90[x];[x]volume=-6dB[f];\
                [7:a]stereotools=balance_in=-0.4[x];[x]surround=chl_out=stereo:chl_in=stereo:angle=270[x];[x]volume=-5dB[g];\
                [8:a]stereotools=balance_in=-0.4[x];[x]surround=chl_out=stereo:chl_in=stereo:angle=90[x];[x]volume=-6dB[h];\
                [a][b][c][d][e][f][g][h]amix=inputs=8[d];[d]loudnorm[d]\
                ".to_string()
            }
        }
    }
}