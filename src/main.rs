use std::path::PathBuf;
use std::str::FromStr;
use crate::switches::SortOrder;
use crate::video::{VideoEditData, VideoGroup};
use clap::{arg, Parser};
use ini::Ini;
use switches::FrameShape;
use crate::helper_functions::MultiPathBuf;

pub(crate) mod group_split;
pub(crate) mod helper_functions;
pub(crate) mod video;
pub(crate) mod audio;
mod switches;
pub fn main() {
    let args = Cli::parse();

    let (mut vid, audio, encoder_args) = run_from_cli(args);

    vid.main_loop(audio, encoder_args);
}

#[deny(missing_docs)]
#[derive(Parser, Debug)]
#[command(author, version)]
struct Cli {
    /// Target *folder* holding videos to join: if unused a request will be given via text input.
    /// - Example to automatically split items into groups: -f "C:\Users\username\Videos\videos_to_join\"
    /// - Example to use folder as a group: -f "D:\videos\left" -f "D:\videos\mid" -f "D:\videos\right"
    /// - Example to use more than one folder for a single input: -f "D:\videos\left1|D:\videos\left2"
    #[arg(short = 'f', long = "folder", action=clap::ArgAction::Append, verbatim_doc_comment)]
    input_folder: Vec<MultiPathBuf>,

    /// Target *file* to output joined video (requires extension)
    /// if unused a request will be given via text input;
    /// - Example: C:\Users\username\Videos\example.mp4
    #[arg(short = 'o', long = "output", verbatim_doc_comment)]
    output_file: Option<PathBuf>,

    /// Shape Format: 'Double' 'Triple' 'Quad' 'VertEmph' 'HorizEmph'
    /// see README.md for more layouts and layout diagrams
    /// if unused a request will be given via text input
    #[arg(short = 's', long = "shape", verbatim_doc_comment)]
    split_format: Option<String>,

    /// Output height in pixels [default:1440]
    #[arg(short = 'y', long = "height")]
    output_height: Option<u32>,

    /// Output width in pixels [default:2560]
    #[arg(short = 'x', long = "width")]
    output_width: Option<u32>,

    /// Frame Per Second
    #[arg(short = 'r', long = "fps", default_value_t = 30.0)]
    fps: f32,

    /// Apply sorting method Options include: 
    ///     "1", "Random", "rand" (default)
    ///     "2", "RandomSeeded", "seed"
    ///     "2r", "RandomSeededR", "seedR"
    ///     "3", "ShortestFirst", "shortest"
    ///     "4", "LongestFirst", "longest"
    ///     "5", "RandomWithLargestLast", "rwll"
    #[arg(long = "ord", verbatim_doc_comment)]
    ord: Option<String>,

    /// select seed for ord when RandomSeeded is chosen
    #[arg(long = "ord-opt", default_value_t = 1337)]
    ord_opt: u64,

    /// removes audio completely
    #[arg(long = "no-audio", action)]
    audio: bool,

    /// set hardware encoder to AMD d3d11va
    #[arg(long = "encode-amd", visible_alias = "amd", action, conflicts_with = "hardware_nvidea")]
    hardware_amd: bool,
    /// set hardware encoder to Nvidea nvenc
    #[arg(long = "encode-nvidea", visible_alias = "nvidea", action)]
    hardware_nvidea: bool,
    /// set output file encoding to  H264
    #[arg(long="h264", action,  conflicts_with_all= ["encode_hvec", "encode_av1"])]
    encode_h264: bool,
    /// set output file encoding to  H265
    #[arg(long = "hvec", visible_alias = "h265", action, conflicts_with = "encode_av1")]
    encode_hvec: bool,
    /// set output file encoding to  AV1
    #[arg(long = "av1", action)]
    encode_av1: bool,

}

fn try_check_local_settings() -> Option<Vec<String>> {
    let current_env = match std::env::current_dir() {
        Ok(e) => { e }
        Err(_) => {
            println!("Local Env Not Found");
            return None;
        }
    };
    let current_ini = current_env.join("settings.ini");

    println!("INI FILE {:?}", current_ini);

    let ini_file = Ini::load_from_file(current_ini);
    let ini_file = match ini_file {
        Ok(e) => { e }
        Err(_) => { return None }
    };

    let mut keys = vec![];

    for kv in ini_file.section("Encoder".into())?.iter() {
        match kv {
            ("video_encoder", v) => {
                keys.push("-c:v");
                keys.push(v)
            }
            (k, v) => {
                keys.push(k);
                keys.push(v);
            }
        }
    }
    let keys = keys.iter().map(|f| f.to_string()).collect();
    Some(keys)
}


fn set_encoder_args(hardware_amd: bool, hardware_nvidea: bool, encode_av1: bool,
                    encode_hvec: bool, encode_h264: bool) -> Vec<String> {
    if !(hardware_nvidea | hardware_nvidea | encode_h264 | encode_hvec | encode_av1) {
        match try_check_local_settings() {
            Some(v) => { return v }
            None => {}
        }
    }

    let hardware = if hardware_amd {
        "amd"
    } else if hardware_nvidea {
        "nvidea"
    } else {
        "none"
    };
    let encoder = if encode_av1 {
        "av1"
    } else if encode_hvec {
        "h265"
    } else if encode_h264 {
        "h264"
    } else {
        "none"
    };
    match (hardware, encoder) {
        ("amd", "av1") => { ["-c:v", "av1_amf", "-rc", "cqp", "-qp_i", "34", "-qp_p", "34", ].as_slice() }
        ("amd", "h265") => { ["-c:v", "hevc_amf", "-rc", "cqp", "-qp_i", "34", "-qp_p", "34", ].as_slice() }
        ("amd", "h264") => { ["-c:v", "h264_amf", "-rc", "cqp", "-qp_i", "34", "-qp_p", "34", ].as_slice() }
        ("nvidea", "av1") => { ["-c:v", "av1_nvenc", "-preset", "slow"].as_slice() }
        ("nvidea", "h265") => { ["-c:v", "hevc_nvenc", "-preset", "slow"].as_slice() }
        ("nvidea", "h264") => { ["-c:v", "h264_nvenc", "-preset", "slow"].as_slice() }
        ("none", "av1") => { ["-c:v", "libaom-av1"].as_slice() }
        ("none", "h265") => { ["-c:v", "libx265", "-speed", "slow", "-crf", "19"].as_slice() }
        ("none", "h264") => { ["-c:v", "libx264", "-speed", "slow", "-crf", "19"].as_slice() }
        _ => { panic!("entered encoders did not work") }
    }.iter().map(|f| f.to_string()).collect()
}

fn request_input(message: &str) -> String {
    let mut buffer = String::new();
    println!("{}", message);
    let stdin = std::io::stdin(); // We get `Stdin` here.
    stdin.read_line(&mut buffer).unwrap();
    buffer
}

fn get_folders_multi(shape: FrameShape) -> Vec<MultiPathBuf> {
    let switch: bool = match shape {
        FrameShape::Dual | FrameShape::Triple | FrameShape::Quad => {
            let req = request_input(
                "use automatic video list from a single folder?"
            ).to_lowercase();
            req.contains('y') | req.contains('1')
        }
        _ => false,
    };
    let count = if switch { 1 } else { shape.count() };

    let items: Vec<MultiPathBuf> = (1..=count).into_iter().map(|x| {
        MultiPathBuf::from_str(request_input(format!("Input Folder Name #{}: ", x).as_str()).as_str().trim()).unwrap()
    }).collect();
    items
}

fn run_from_cli(args: Cli) -> (VideoGroup, bool, Vec<String>) {
    let split_format = match args.split_format
        .unwrap_or_else(|| {
            request_input("Split Format 'Double' / 'Triple' / 'Quad' (see README.md for more options): ")
        }).to_lowercase().as_str().trim() {
        "double" | "d" | "2" => { FrameShape::Dual }
        "triple" | "t" | "3" => { FrameShape::Triple }
        "quad" | "q" | "4" => { FrameShape::Quad }
        "vertemph" | "vert" | "v" | "5" => { FrameShape::VertEmph }
        "horizemph" | "horiz" | "h" | "6" => { FrameShape::HorizEmph }
        "vertemph2" | "vert2" | "v2" | "7" => { FrameShape::VertEmph2 }
        "horizemph2" | "horiz2" | "h2" | "8" => { FrameShape::HorizEmph2 }
        "sidevert" | "vd" | "9" => { FrameShape::SideVert }
        "sidevert2" | "dv" | "10" => { FrameShape::SideVert2 }
        "centreemphvert" | "ce" | "11" => { FrameShape::CentreEmphVert }
        "centreemphvert2" | "ce2" | "12" => { FrameShape::CentreEmphVert2 }
        "morehoriz" | "mh" | "13" => { FrameShape::MoreHoriz }
        "morehoriz2" | "mh2" | "14" => { FrameShape::MoreHoriz2 }
        x => { panic!("No match found for split format: {}", x) }
    };

    let sort_ord = match args.ord.unwrap_or("Random".to_string()).to_lowercase().as_str().trim() {
        "1" | "random" | "rand" => SortOrder::Random,
        "2" | "randomseeded" | "seed" => SortOrder::RandomSeeded(args.ord_opt),
        "2r" | "randomseededr" | "seedr" => SortOrder::RandomSeededR(args.ord_opt),
        "3" | "shortestfirst" | "shortest" => SortOrder::ShortestFirst,
        "4" | "longestfirst" | "longest" => SortOrder::LongestFirst,
        "5" | "randomwithlargestlast" | "rwll" => SortOrder::RandomWithLargestLast,
        _ => { panic!("That is not a valid SortOrder") }
    };


    let mut folder_target = args.input_folder;
    if folder_target.len() == 0 {
        folder_target = get_folders_multi(split_format.clone());
    }

    let output_file = args.output_file.unwrap_or_else(|| {
        PathBuf::from(request_input("Output File Name: ").as_str().trim())
    });

    let matcher = (args.output_width, args.output_height, split_format.clone());
    let mut vid_edit_data: VideoEditData = match matcher {
        (Some(w), Some(h), sf) => { VideoEditData::init_wxh(w, h, sf.count()) }
        (_, _, sf) => { VideoEditData::init_wxh(2560, 1440, sf.count()) }
    };

    vid_edit_data.set_fps(args.fps);


    let mut vid = match folder_target.len() {
        0 => { panic!("No folders found") }
        1 => {
            VideoGroup::new_from_folder(
                folder_target.first().unwrap(),
                output_file,
                split_format.clone(),
                sort_ord,
            )
        }
        2..=5 => {
            VideoGroup::new_from_folders(
                folder_target,
                output_file,
                split_format.clone(),
                sort_ord,
            )
        }
        _ => { panic!("More than 5 folders is currently unsupported") }
    };
    let encoder_args = set_encoder_args(
        args.hardware_amd,
        args.hardware_nvidea,
        args.encode_av1,
        args.encode_hvec,
        args.encode_h264,
    );
    vid_edit_data.set_shape(split_format.clone());
    vid.set_video_sizer(vid_edit_data);

    (vid, args.audio, encoder_args)
}
