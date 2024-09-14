# Modification notes

## Adding a new frame shape

1. Add variant to enum [src/switches.rs] `FrameShape` 
2. Add call method to [src/main.rs] `run_from_cli` > `let split_format`
   1. note all match parts should be lowercase
3. add count data to [src/switches.rs] `FrameShape::count` (this is the number of frames being used)
4. Add implement member in [src/switches.rs] `FrameShape::audio_args_with_vid`
5. implement joiner, notes below [src/switches.rs] `FrameShape::frame_joiner`

## Automatic folder, extra not required
[src/video.rs] `VideoGroup::new_from_folders` add ot this to implement [README.md] `Special case`  
