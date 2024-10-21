use crate::video::Video;
use std::cmp::Ordering;

impl Video {
    fn into_lenv(mut self) -> LenV {
        LenV {
            group: 0,
            inner: self.get_length().unwrap(),
            tar: self,
        }
    }
}


struct LenV {
    group: usize,
    inner: i64,
    tar: Video,
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
    let last_avg: i64 = group_diff_avg(&v_list, &groups);
    loop {
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
            }
            None => { break }
        }
    }
    v_list
}

fn group_swap(mut v_list: Vec<LenV>, groups: usize) -> Vec<LenV> {
    v_list.sort_unstable_by_key(|k| i64::MAX - k.inner);
    let last_avg: i64 = position_sums(&v_list, &groups).iter()
        .fold(0i64, |a, b| a + b) / (groups as i64);
    loop {
        let grp_sum = position_sums(&v_list, &groups);
        let (grp_min, grp_max) = position_sums_min_max(&grp_sum);

        let filtermin = v_list.iter().filter(|f| f.group == grp_min.0);
        let filtermax = v_list.iter().filter(|f| f.group == grp_max.0);
        
        let mut current_diff = grp_max.1 - grp_min.1;
        let mut set_action = None;
        for fmin in filtermin {
            for fmax in filtermax.clone() {
                let diff = fmax.inner - fmin.inner;
                if diff < current_diff {
                    set_action = Some((fmin, fmax));
                    current_diff = diff;
                }
            }
        }
        match set_action {
            None => { break }
            Some((a, v)) => {
                let a_pos = v_list.binary_search(a).unwrap();
                let v_pos = v_list.binary_search(v).unwrap();
                (v_list[a_pos].group, v_list[v_pos].group) = (v_list[v_pos].group, v_list[a_pos].group);
                if last_avg < group_diff_avg(&v_list, &groups) {
                    (v_list[a_pos].group, v_list[v_pos].group) = (v_list[v_pos].group, v_list[a_pos].group);
                    break;
                }
            }
        }
    }
    v_list
}

pub fn video_regroup(v_list: Vec<Video>, groups: usize) -> Vec<Vec<Video>> {
    let mut v_list: Vec<LenV> = v_list.into_iter().map(|f| f.into_lenv()).collect();
    for (i, v) in v_list.iter_mut().enumerate() {
        v.group = i.rem_euclid(groups)
    }
    v_list = group_move(v_list, groups);
    v_list = group_swap(v_list, groups);

    let mut out_lists: Vec<Vec<Video>> = (0..groups).map(|_| { Vec::default() }).collect();
    for v in v_list { out_lists[v.group].push(v.into_video()) }
    out_lists
}