use crate::video::Video;
use rand::prelude::SliceRandom;
use rand::rngs::StdRng;
use rand::thread_rng;
use rand::SeedableRng;

#[derive(Clone, Debug)]
pub(crate) enum SortOrder {
    Random,
    RandomSeeded(u64),
    RandomSeededR(u64),
    ShortestFirst,
    LongestFirst,
    RandomWithLargestLast,
    Nan,
}
impl SortOrder {
    pub(crate) fn apply_sort(&self, mut videos: Vec<Video>) -> Vec<Video> {
        // if SortOrder needs the length we make sure it exists here first
        match self {
            SortOrder::ShortestFirst | SortOrder::LongestFirst |
            SortOrder::RandomWithLargestLast => {
                videos.iter_mut().for_each(|f| { let _ = f.get_length(); });
            }
            _ => {}
        }
        match self {
            SortOrder::Nan => {
                videos
            }
            SortOrder::Random => {
                let mut rng = thread_rng();
                videos.shuffle(&mut rng);
                videos
            }
            SortOrder::RandomSeeded(s) => {
                let mut rng = <StdRng as SeedableRng>::seed_from_u64(s.clone());
                videos.shuffle(&mut rng);
                videos
            }
            SortOrder::RandomSeededR(s) => {
                let mut rng = <StdRng as SeedableRng>::seed_from_u64(s.clone());
                videos.shuffle(&mut rng);
                videos.reverse();
                videos
            }
            SortOrder::ShortestFirst => {
                videos.sort_unstable_by_key(|f| f.length_millis.unwrap_or(0));
                videos
            }
            SortOrder::LongestFirst => {
                videos.sort_unstable_by_key(|f| 0 - f.length_millis.unwrap_or(0));
                videos
            }
            SortOrder::RandomWithLargestLast => {
                let mut rng = thread_rng();
                videos.shuffle(&mut rng);
                let longest = videos.iter().max_by_key(|x| x.length_millis.unwrap_or(0)).unwrap();
                let v_longest = videos.remove(
                    videos.iter().position(|v| std::ptr::addr_eq(v, longest)).unwrap()
                );
                videos.push(v_longest);
                videos
            }
        }
    }
}


