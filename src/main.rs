use std::path::PathBuf;

use clap::{arg, Parser};
use crate::video::{FrameShape, VideoEditData, VideoGroup};

pub(crate) mod group_split;
pub(crate) mod helper_functions;
pub(crate) mod video;
// mod cmd_video;
pub(crate) mod audio;


pub fn main() {
    let args = Cli::parse();

    let (mut vid, audio) = run_from_cli(args);

    vid.main_loop(audio);
}

#[deny(missing_docs)]
#[derive(Parser, Debug)]
#[command(author, version)]
struct Cli {
    /// Target *folder* holding videos to join: if unused a request will be given via text input.
    /// - Example to automatically split items into groups: -f "C:\Users\username\Videos\videos_to_join\"
    /// - Example to use folder as a group: -f "D:\videos\left" -f "D:\videos\mid" -f "D:\videos\right"
    #[arg(short = 'f', long = "folder", action=clap::ArgAction::Append, verbatim_doc_comment)]
    input_folder: Vec<PathBuf>,

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

    /// Output width in pixels [default:2550]
    #[arg(short = 'x', long = "width")]
    output_width: Option<u32>,

    /// Frame Per Second
    #[arg(short = 'r', long = "fps", default_value_t = 30.0)]
    fps: f32,

    #[arg(long = "no-audio", action)]
    audio: bool,

}

fn request_input(message: &str) -> String {
    let mut buffer = String::new();
    println!("{}", message);
    let stdin = std::io::stdin(); // We get `Stdin` here.
    stdin.read_line(&mut buffer).unwrap();
    buffer
}

fn get_folders_multi(shape: FrameShape) -> Vec<PathBuf> {
    let switch: bool = match shape {
        FrameShape::Dual | FrameShape::Triple | FrameShape::Quad => {
            let req = request_input(
                "use automatic video list from a single folder?"
            ).to_lowercase();
            req.contains('y') | req.contains('1')
        }
        FrameShape::VertEmph => false,
        FrameShape::HorizEmph => false,
        FrameShape::VertEmph2 => false,
        FrameShape::HorizEmph2 => false,
    };
    let count = if switch { 1 } else { shape.count() };

    let items: Vec<PathBuf> = (1..=count).into_iter().map(|x| {
        PathBuf::from(request_input(format!("Input Folder Name #{}: ", x).as_str()).as_str().trim())
    }).collect();
    items
}

fn run_from_cli(args: Cli) -> (VideoGroup, bool) {
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
        x => { panic!("No match found for split format: {}", x) }
    };

    let mut folder_target = args.input_folder;
    if folder_target.len() == 0 {
        folder_target = get_folders_multi(split_format.clone())
    }

    let output_file = args.output_file.unwrap_or_else(|| {
        PathBuf::from(request_input("Output File Name: ").as_str().trim())
    });

    let matcher = (args.output_width, args.output_height, split_format.clone());
    let mut vid_edit_data: VideoEditData = match matcher {
        (Some(w), Some(h), sf) => { VideoEditData::init_wxh(w, h, sf.count()) }
        (_, _, FrameShape::Dual) => { VideoEditData::init_wxh(1280 * 2, 1440, split_format.count()) }
        (_, _, FrameShape::Triple) => { VideoEditData::init_wxh(850 * 3, 1440, split_format.count()) }
        (_, _, FrameShape::Quad) => { VideoEditData::init_wxh(1280 * 2, 1440, split_format.count()) }
        (_, _, FrameShape::VertEmph) | (_, _, FrameShape::VertEmph2) => {
            VideoEditData::init_wxh(850 * 3, 1440, split_format.count())
        }
        (_, _, FrameShape::HorizEmph) | (_, _, FrameShape::HorizEmph2) => {
            VideoEditData::init_wxh(850 * 3, 1440, split_format.count())
        }
    };

    vid_edit_data.set_fps(args.fps);

    let mut vid = match folder_target.len() {
        0 => { panic!("No folders found") }
        1 => {
            VideoGroup::new_from_folder(
                folder_target.first().unwrap(),
                output_file,
                split_format.clone(),
            )
        }
        2..=5 => {
            VideoGroup::new_from_folders(
                folder_target,
                output_file,
                split_format.clone(),
            )
        }
        _ => { panic!("More than 5 folders is currently unsupported") }
    };

    vid_edit_data.set_shape(split_format.clone());
    vid.set_video_sizer(vid_edit_data);

    (vid, args.audio)
}

