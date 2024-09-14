use std::path::PathBuf;

use crate::helper_functions::iter_ffmpeg_events;
#[cfg(feature = "hyperDebug")]
use crate::helper_functions::parse_debug;
use crate::switches::FrameShape;
use ffmpeg_sidecar;
use ffmpeg_sidecar::child::FfmpegChild;
use ffmpeg_sidecar::command::FfmpegCommand;


//noinspection SpellCheckingInspection
// const STREAM_PRE_CONCAT_ARG: &str = " -fa dynaudnorm=p=0.9:s=5:t=0.4:f=1000:m=2 afade=t=in:ss=0:d=2";
// const STREAM_PRE_CONCAT_ARG: &str = " -fa afade=t=in:ss=0:d=2 dynaudnorm=p=0.9:s=5:t=0.4:f=1000:m=2";
const STREAM_PRE_CONCAT_ARG: &str = " -fa loudnorm afade=t=in:ss=0:d=2 dynaudnorm=p=0.9:s=5:t=0.4:f=1000:m=2";

fn concat_audio_streams(p_list: Vec<PathBuf>, temp: &PathBuf, stream_id: usize) -> (PathBuf, FfmpegChild) {
    // generate temporary file containing a list of files
    // this is a preemptive workaround for the problem of having too many files to concat
    println!("Audio Concat Started for #{}", stream_id);

    #[cfg(feature = "hyperDebug")]
    parse_debug("\tgenerating text_template", file!(), line!());
    let temp_tar = temp.join(format!("temp{:?}.txt", stream_id));

    let temp_tar_content = p_list
        .iter()
        .map(|f| format!("file '{}' {}",
                         f.to_str().unwrap(), STREAM_PRE_CONCAT_ARG))
        .collect::<Vec<String>>().join("\n");
    std::fs::write(&temp_tar, temp_tar_content).unwrap();
    let out_tar = temp.join(format!("g{:?}.wav", stream_id));

    #[cfg(feature = "hyperDebug")]
    parse_debug("    running", file!(), line!());
    let mut worker = FfmpegCommand::new();
    let worker = worker
        .args(["-loglevel", "error"])
        //.input(temp_tar.to_str().unwrap())
        .args([
            "-vn",
            "-f", "concat",
            "-safe", "0",
            "-i", temp_tar.to_str().unwrap(),
            // "-c", "copy",
        ])
        .arg("-y")
        .output(&out_tar.to_str().unwrap());
    (out_tar, worker.spawn().unwrap())
}
pub fn join_audio_video_streams(audio_segments: Vec<Vec<PathBuf>>,
                                temp: &PathBuf, video_temp: &PathBuf, video_out: PathBuf,
                                frame_shape: &FrameShape) -> PathBuf {
    println!("Audio/Video joiner Started, this can be slow");

    #[cfg(feature = "hyperDebug")]
    parse_debug("Audio stream concat", file!(), line!());
    // list of paths of audio items to be joined
    let audio_items: Vec<(PathBuf, FfmpegChild)> = audio_segments.into_iter().enumerate()
        .map(|(i, v)| {
            concat_audio_streams(v, &temp, i)
        }).collect();

    let audio_items: Vec<PathBuf> = audio_items.into_iter().map(|(pb, mut ffm)| {
        let _ = ffm.wait();
        pb
    }).collect();
    
    println!("Audio Export finished for all streams, now joining streams to file");

    #[cfg(feature = "hyperDebug")]
    parse_debug("Single A/V Filter Setup", file!(), line!());

    let mut worker = FfmpegCommand::new();
    worker.args(["-loglevel", "error"]);
    worker.input(video_temp.to_str().unwrap());
    for inp in audio_items {
        worker.input(inp.to_str().unwrap());
    }
    worker.args(["-filter_complex", &*frame_shape.audio_args_with_vid()]);
    worker.args(["-c:v", "copy", ]).arg("-y");
    worker.map("0:v:0").map("[d]");
    worker.output(&video_out.to_str().unwrap());

    #[cfg(feature = "hyperDebug")]
    parse_debug("Single A/V joiner started", file!(), line!());
    let mut complete = worker.spawn().unwrap();
    iter_ffmpeg_events(&mut complete);
    complete.wait().unwrap();

    #[cfg(feature = "hyperDebug")]
    parse_debug("join_audio_to_video end", file!(), line!());

    video_out
}
