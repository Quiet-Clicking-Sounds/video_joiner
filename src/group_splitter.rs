use crate::video::Video;
use std::cell::RefCell;
use std::cmp::Ordering;
use std::sync::atomic::AtomicUsize;
use std::time::{Duration, Instant};
use crate::helper_functions::seconds_to_hhmmss;

fn get_id() -> usize {
    static COUNTER: AtomicUsize = AtomicUsize::new(1);
    COUNTER.fetch_add(1, core::sync::atomic::Ordering::Relaxed)
}

impl Video {
    fn into_lenv(mut self) -> LenV {
        LenV {
            group: 0,
            inner: self.get_length().unwrap(),
            tar: self,
            _id: RefCell::new(get_id()),
            _grp: RefCell::new(0),
        }
    }
}


struct LenV {
    group: usize,
    inner: i64,
    tar: Video,
    _id: RefCell<usize>,
    _grp: RefCell<usize>,
}
impl LenV {
    fn into_video(self) -> Video {
        self.tar
    }
}


impl Eq for LenV {}
impl PartialOrd<Self> for LenV {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.inner.partial_cmp(&other.inner)
    }
}
impl PartialEq<Self> for LenV {
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}

impl Ord for LenV {
    fn cmp(&self, other: &Self) -> Ordering {
        self.inner.cmp(&other.inner)
    }
}


fn position_sums(lst: &Vec<LenV>, groups: &usize) -> Vec<i64> {
    let mut counts: Vec<i64> = vec![0; *groups];
    for i in lst.iter() { counts[i.group] += i.inner }
    counts
}

fn position_sums_min_max(p0: &Vec<i64>) -> ((usize, i64), (usize, i64)) {
    let min_ = p0.iter().enumerate()
        .reduce(|a, b| { if a.1 < b.1 { a } else { b } }).unwrap();
    let max_ = p0.iter().enumerate()
        .reduce(|a, b| { if a.1 > b.1 { a } else { b } }).unwrap();
    (
        (min_.0, min_.1.clone()),
        (max_.0, max_.1.clone())
    )
}

/// average of the difference between each item and 0, lower is better
fn group_diff_avg(lst: &Vec<LenV>, groups: &usize) -> i64 {
    let sums = position_sums(lst, &groups);
    let minsums = sums.iter().min().unwrap();
    sums.iter()
        .fold(0i64, |a, b| a + b - minsums) / (groups.clone() as i64)
}

fn group_move(mut v_list: Vec<LenV>, groups: usize) -> Vec<LenV> {
    v_list.sort_unstable_by_key(|k| i64::MAX - k.inner);
    let mut last_avg: i64 = group_diff_avg(&v_list, &groups);
    for _i in 0..256 {
        let grp_sum = position_sums(&v_list, &groups);
        let (grp_min, grp_max) = position_sums_min_max(&grp_sum);
        let grp_diff = grp_max.1 - grp_min.1;
        match v_list.iter_mut().position(|f| (f.group != grp_min.0) & (f.inner < grp_diff)) {
            Some(position) => {
                let pre = v_list[position].group;
                v_list[position].group = grp_min.0;
                if last_avg < group_diff_avg(&v_list, &groups) {
                    v_list[position].group = pre;
                    break;
                }
                last_avg = group_diff_avg(&v_list, &groups);
            }
            None => { break }
        }
    }
    v_list
}


fn video_regroup(mut v_list: Vec<LenV>, groups: usize) -> Vec<Vec<LenV>> {
    for (i, v) in v_list.iter_mut().enumerate() {
        v.group = i.rem_euclid(groups)
    }
    v_list = group_move(v_list, groups);
    let mut out_lists: Vec<Vec<LenV>> = (0..groups).map(|_| { Vec::default() }).collect();
    for v in v_list { out_lists[v.group].push(v) };
    out_lists
}

enum ReGroupStatus {
    Complete(Vec<Vec<LenV>>),
    Error(Vec<LenV>),
}

pub fn regrouper(v_list: Vec<Video>, group_count: usize) -> Vec<Vec<Video>> {
    let items: Vec<LenV> = v_list.into_iter().map(|f| f.into_lenv()).collect();

    let comp = match generalized_regrouper(items, group_count.clone()) {
        ReGroupStatus::Complete(items) => { items }
        ReGroupStatus::Error(items) => {
            println!("smart re-grouper failed, using less smart re-grouper");
            video_regroup(items, group_count)
        }
    };

    let vid_groups: Vec<Vec<Video>> = comp.into_iter().map(|f| f.into_iter().map(|i| i.into_video()).collect()).collect();

    let group_len: Vec<i64> = vid_groups.iter().map(|vg|
        vg.iter().fold(0i64, |x: i64, y| x + y.length_millis.unwrap())
    ).collect();
    let min_group = group_len.iter().reduce(|a,b|{a.min(b)}).unwrap();
    
    let group_len:Vec<String> = group_len.iter().map(|f|seconds_to_hhmmss(f.clone() as u64 /1000)).collect();
    println!("Group sizes:  {}", group_len.join(" | "));
    println!("Video Length: {}",seconds_to_hhmmss(min_group.clone() as u64 /1000));
    
    vid_groups
}


fn generalized_regrouper(items: Vec<LenV>, group_count: usize) -> ReGroupStatus {
    match partition_into_groups(&items, group_count, 10) {
        Some(pargrp) => {
            for (g, group) in pargrp.iter().enumerate() {
                for item in group.iter() {
                    item._grp.replace(g);
                }
            }
        }
        None => { return ReGroupStatus::Error(items) }
    };

    let mut regrouped: Vec<Vec<LenV>> = (0..group_count).map(|_| vec![]).collect();

    for i in items.into_iter() {
        let g = i._grp.borrow().clone();
        regrouped[g].push(i)
    }
    ReGroupStatus::Complete(regrouped)
}


fn partition_into_groups<'a>(vec: &'a [LenV], group_count: usize, time_limit: u64) -> Option<Vec<Vec<&'a LenV>>> {
    let total_sum: i64 = vec.iter().map(|item| item.inner).sum();
    let target_sum = total_sum / group_count as i64;

    let deviation_mul = target_sum / 100;
    let max_attempts = 10; // Number of attempts to find a valid partition

    let mut best_solution: Option<Vec<Vec<&'a LenV>>> = None;
    let mut best_deviation = i64::MAX;

    for attempt in 1..max_attempts {
        let time_limit = Duration::from_secs(time_limit.saturating_sub(attempt as u64));
        let start_time = Instant::now();
        best_solution = match best_solution {
            None => { None }
            Some(mut v) => {
                v.reverse();
                Some(v)
            }
        };
        let current_max_deviation = attempt * deviation_mul; // Increase deviation with each attempt

        let mut groups: Vec<Vec<&'a LenV>> = vec![Vec::new(); group_count];
        let mut current_sums = vec![0; group_count];

        if backtrack(vec, 0, &mut current_sums, &mut groups, target_sum, current_max_deviation, &start_time, time_limit, &mut best_solution, &mut best_deviation) {
            return Some(groups);
        }
    }
    best_solution
}

fn backtrack<'a>(
    vec: &'a [LenV],
    index: usize,
    current_sums: &mut Vec<i64>,
    groups: &mut Vec<Vec<&'a LenV>>,
    target_sum: i64,
    max_deviation: i64,
    start_time: &Instant,
    time_limit: Duration,
    best_solution: &mut Option<Vec<Vec<&'a LenV>>>,
    best_deviation: &mut i64,
) -> bool {
    // Check if time limit is exceeded
    if start_time.elapsed() > time_limit {
        return false;
    }

    if index == vec.len() {
        // Calculate the current deviation
        let current_deviation: i64 = current_sums.iter().map(|&sum| (sum - target_sum).abs()).sum();

        // Check if this is the best solution so far
        if current_deviation < *best_deviation {
            *best_deviation = current_deviation;
            *best_solution = Some(groups.clone());
        }

        return current_sums.iter().all(|&sum| {
            let deviation = (sum - target_sum).abs();
            deviation <= max_deviation
        });
    }

    for i in 0..groups.len() {
        if current_sums[i] + vec[index].inner <= target_sum + max_deviation {
            groups[i].push(&vec[index]);
            current_sums[i] += vec[index].inner;

            if backtrack(vec, index + 1, current_sums, groups, target_sum, max_deviation, start_time, time_limit, best_solution, best_deviation) {
                return true;
            }

            groups[i].pop();
            current_sums[i] -= vec[index].inner;
        }

        if groups[i].is_empty() {
            break;
        }
    }

    false
}