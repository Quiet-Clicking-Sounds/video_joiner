use std::fmt::{Debug, Formatter};


use std::time;
use ffmpeg_sidecar::child::FfmpegChild;
use ffmpeg_sidecar::event::{FfmpegEvent, LogLevel};


pub fn iter_ffmpeg_events(child:&mut FfmpegChild){
    for i in child.iter().unwrap() {
        match i {
            FfmpegEvent::Log(level, data) => {
                match level {
                    LogLevel::Info => {
                        //eprintln!("LOG INFO {:?} : {}", level, data)
                    }
                    LogLevel::Error | LogLevel::Fatal => {
                        eprintln!("LOG {:?} : {} ", level, data);
                        // return None;
                    }
                    LogLevel::Warning=> {
                        eprintln!("LOG {:?} : {} ", level, data);
                        // return None;
                    }
                    LogLevel::Unknown => {
                        if data.len() > 2 {
                            eprintln!("LOG {:?} : {}", level, data)
                        }
                    }
                }
            }
            _ => {}
        };
    }
}


pub struct FrameTimer {
    _init_: time::Instant,
    pub(crate) _frame_count_: u128,
}

impl FrameTimer {
    pub fn new() -> FrameTimer {
        FrameTimer {
            _init_: time::Instant::now(),
            _frame_count_: 0,
        }
    }

    pub fn frame(&mut self) {
        self._frame_count_ += 1
    }

    pub fn fps(&self) -> f64 {
        self._frame_count_.clone() as f64 / self._init_.elapsed().as_secs_f64()
    }
}

impl Debug for FrameTimer {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FrameTimer")
            .field("Frame count", &self._frame_count_)
            .field("Elapsed Time", &self._init_.elapsed())
            .field("Compute FPS",&format!("{:.2}",self.fps()))
            .finish()
    }
}

pub fn seconds_to_hhmmss(sec: u64)->String{
    let (min, sec) = (&sec / 60, &sec % 60);
    let (hour, min) = (&min / 60, &min % 60);
    format!("{:02}-{:02}-{:02}", hour,min,sec)
}



#[cfg(feature = "hyperDebug")]
#[inline]
pub fn parse_debug(_text: &str, _f: &str, _l: u32) {
    eprintln!("{}:{} - {}", _f, _l, _text);
}




#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn frame_timer_counter_check() {
        let mut ft = FrameTimer::new();
        let iters = 20;
        for _ in 0..iters { ft.frame();}
        assert_eq!(ft._frame_count_, iters);
    }


    #[test]
    fn frame_timer_time_check() {
        let float_duration = 1.5;
        let duration = Duration::from_secs_f64(float_duration);
        let frame_timer = FrameTimer::new();
        thread::sleep(duration);
        let frame_timer_duration = frame_timer._init_.elapsed().as_secs_f64();
        assert!((&float_duration - frame_timer_duration).abs() < 0.1 );
    }


}
