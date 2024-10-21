use crate::group_split::AutoSwap::Continue;
use crate::helper_functions::seconds_to_hhmmss;
use crate::frame_shape::FrameShape;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::fmt::Debug;
use std::fmt::Formatter;
#[cfg(feature = "hyperDebug")]
use crate::helper_functions;

pub enum AutoSwap {
    Continue,
    Done,
}

#[derive(Copy, Clone, PartialEq)]
enum ItemGroup {
    A,
    B,
    C,
    D,
}

impl Debug for ItemGroup {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            ItemGroup::A => "ItemGroup::A",
            ItemGroup::B => "ItemGroup::B",
            ItemGroup::C => "ItemGroup::C",
            ItemGroup::D => "ItemGroup::D",
        })
    }
}

#[derive(PartialEq)]
enum SwapHistory {
    #[allow(unused)] // may be implemented soon 
    Single(usize, ItemGroup),
    Double(usize, ItemGroup, usize, ItemGroup),
}

struct Item<D> {
    data: i64,
    _external_data_: D,
    group: RefCell<ItemGroup>,
}

impl<D> Item<D> {
    /// sets the item group based on the index
    /// (gets close to a useful distribution with ordered inputs)
    pub fn from_data_group(data: i64, grp: ItemGroup, ext_data: D) -> Item<D> {
        Item {
            data,
            _external_data_: ext_data,
            group: RefCell::from(grp),
        }
    }
}

impl<D> Debug for Item<D> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&*match &*self.group.borrow() {
            ItemGroup::A => {
                format!("A{}", self.data)
            }
            ItemGroup::B => {
                format!("B{}", self.data)
            }
            ItemGroup::C => {
                format!("C{}", self.data)
            }
            ItemGroup::D => {
                format!("D{}", self.data)
            }
        })
    }
}

pub struct ItemList<D> {
    items: Vec<Item<D>>,
    screens: FrameShape,
    swap_history: RefCell<VecDeque<SwapHistory>>,
}

impl<D> ItemList<D> {
    pub fn new_with_data(mut data: Vec<(i64, D)>, screens: FrameShape) -> ItemList<D> {
        #[cfg(feature = "hyperDebug")]
        helper_functions::parse_debug("ItemList::new_with_data ", file!(), line!());
        #[cfg(feature = "hyperDebug")]
        println!("Frame Shape = {}", screens.clone().count());

        match &screens {
            FrameShape::Dual | FrameShape::Triple | FrameShape::Quad => {}
            _ => {
                panic!("Automatic Grouping is not enabled for {:?}", screens)
            }
        }

        data.sort_unstable_by_key(|&(f, _)| f);

        let target = data.iter().fold(0i64, |acc, (e, _)| acc + e)
            / screens.clone().count() as i64;
        println!("Target length: {:?}", seconds_to_hhmmss(target.clone() as u64 / 1000));
        let mut v: Vec<Item<D>> = vec![];

        let sclen = screens.count() as usize;

        for (idx, (dta, ext)) in data.into_iter().enumerate() {
            v.push(
                Item::from_data_group(
                    dta,
                    match idx % sclen {
                        0 => ItemGroup::A,
                        1 => ItemGroup::B,
                        2 => ItemGroup::C,
                        _ => ItemGroup::D,
                    },
                    ext,
                )
            )
        }

        ItemList {
            items: v,
            screens,
            swap_history: RefCell::new(VecDeque::new()),
        }
    }

    fn print_group_len(&self){
        let minlen = match (&self.screens, self.sum_each()) {
            (FrameShape::Dual, (a,b, ..)) => {
                a.min(b)
            }
            (FrameShape::Triple, (a,b,c, ..)) => {
                a.min(b).min(c)
            }
            (FrameShape::Quad, (a,b,c,d)) => {
                a.min(b).min(c).min(d)
            }
            _=>{0}
        };
        println!("True length of ouput video: {:?}",seconds_to_hhmmss(minlen as u64 / 1000));
    }
    
    /// Lifetime Ends - return groups
    pub fn export_to_data_lists(self) -> Vec<Vec<D>> {
        let mut out = vec![vec![], vec![], vec![], vec![]];
        #[cfg(feature = "hyperDebug")]
        helper_functions::parse_debug(" Export Data to Lsit ", file!(), line!());
        #[cfg(feature = "hyperDebug")]
        println!("Len sum of video splits: {:?}", self.sum_each());
        self.print_group_len();
        for i in self.items.into_iter() {
            match &*i.group.borrow() {
                ItemGroup::A => out[0].push(i._external_data_),
                ItemGroup::B => out[1].push(i._external_data_),
                ItemGroup::C => out[2].push(i._external_data_),
                ItemGroup::D => out[3].push(i._external_data_),
            }
        }
        
        #[cfg(feature = "hyperDebug")]
        println!("Group sizes: {}, {}, {}, {}",
                 out[0].len(), out[1].len(), out[2].len(), out[3].len());
        
        
        
        match self.screens {
            FrameShape::Dual => {
                out.remove(3);
                out.remove(2);
                out
            }
            FrameShape::Triple => {
                out.remove(3);
                out
            }
            FrameShape::Quad => {
                out
            }
            _ => {
                panic!("Automatic sorting is not enabled for {:?}", self.screens)
            }
        }
    }

    fn reduce_swap_history_length(&self) {
        while self.swap_history.borrow_mut().len() > 5 {
            self.swap_history.borrow_mut().pop_front();
        }
    }

    /// return the totals of each variable
    /// todo: make this a part of the struct to save on compute time
    pub fn sum_each(&self) -> (i64, i64, i64, i64) {
        let mut abc = (0i64, 0i64, 0i64, 0i64);
        for i in self.items.iter() {
            match &*i.group.borrow() {
                ItemGroup::A => abc.0 += &i.data,
                ItemGroup::B => abc.1 += &i.data,
                ItemGroup::C => abc.2 += &i.data,
                ItemGroup::D => abc.3 += &i.data,
            }
        }
        abc
    }

    /// function to check if an item is in history.
    /// `idx0` and `idx1` ignore order
    fn is_item_in_history_double(&self, idx0: usize, idx1: usize) -> bool {
        self.swap_history.borrow().iter().any(|f| match f {
            SwapHistory::Double(id0, _, id1, _) => {
                (id0, id1) == (&idx0, &idx1) || (id1, id0) == (&idx0, &idx1)
            }
            _ => false,
        })
    }


    /// function to update the history item
    fn update_item_with_history_double(
        &self,
        idx0: usize,
        grp0: ItemGroup,
        idx1: usize,
        grp1: ItemGroup,
    ) {
        #[cfg(feature = "hyperDebug")]
        helper_functions::parse_debug("ItemList::update_item_with_history_double",
                                      file!(), line!());
        #[cfg(feature = "hyperDebug")]
        println!("idx0:{:?} grp0:{:?} idx1:{:?} grp1:{:?} ",
                 idx0, grp0, idx1, grp1);

        let typ0 = self.items[idx0].group.replace(grp0);
        let typ1 = self.items[idx1].group.replace(grp1);
        //println!("update_item_with_history_double i[{:?}] {:?}>{:?} | i[{:?}] {:?}>{:?}", idx0, grp0,typ0, idx1, grp1, typ1);
        self.swap_history
            .borrow_mut()
            .push_back(SwapHistory::Double(idx0.clone(), typ0,
                                           idx1.clone(), typ1));
        self.reduce_swap_history_length()
    }

    /// helper function to return
    ///`((minimum group size, minimum group ref), (maximum group size, maximum group ref))`
    fn typ_ref_sum_min_max(&self) -> ((i64, ItemGroup), (i64, ItemGroup)) {
        let (a, b, c, d) = self.sum_each();

        match self.screens {
            FrameShape::Dual => {
                if a < b {
                    ((a, ItemGroup::A), (b, ItemGroup::B))
                } else {
                    ((b, ItemGroup::B), (a, ItemGroup::A))
                }
            }
            FrameShape::Triple => {
                let ign = [
                    (a, ItemGroup::A), (b, ItemGroup::B), (c, ItemGroup::C)
                ];
                (*ign.iter().max_by_key(|(n, _)| n).unwrap(),
                 *ign.iter().max_by_key(|(n, _)| n).unwrap())
            }
            FrameShape::Quad => {
                let ign = [
                    (a, ItemGroup::A), (b, ItemGroup::B), (c, ItemGroup::C), (d, ItemGroup::D)
                ];
                (*ign.iter().max_by_key(|(n, _)| n).unwrap(),
                 *ign.iter().max_by_key(|(n, _)| n).unwrap())
            }
            _ => { panic!("Shape {:?} is unsupported", self.screens) }
        }
    }

    ///
    /// Swap items between the smallest and largest groups
    /// aims to find numbers from the minimum list and maximum list which are near
    /// 75% of the difference between them
    ///
    ///
    pub fn auto_double_swap(&self) -> AutoSwap {
        #[cfg(feature = "hyperDebug")]
        helper_functions::parse_debug("ItemList::auto_double_swap",
                                      file!(), line!());
        let ((min_t, min_g), (max_t, max_g)) = self.typ_ref_sum_min_max();
        // 
        // let diff = max_t - min_t;
        // for item in self.items.iter(){
        //     
        // } 
        // 
        // let max_t_min_i = self.items.iter().filter(|a|{
        //     a.group.borrow().get_dt() == min_g.get_dt()
        // }).min_by_key(|a|a.data).unwrap();
        // if max_t - min_t > max_t_min_i.data {
        //     max_t_min_i.group.replace(min_g);
        //     return Continue
        // }

        let min_to_max_diff = (max_t - min_t).abs() * 3 / 4;
        let mut best_swap = None;
        for (idx0, item0) in self.items.iter().enumerate() {
            if *item0.group.borrow() != min_g {
                continue;
            }

            for (idx1, item1) in self.items.iter().enumerate() {
                if *item1.group.borrow() != max_g {
                    continue;
                }
                if item0.data >= item1.data {
                    continue;
                };
                let i_diff = item1.data.clone() - item0.data.clone();
                if i_diff > min_to_max_diff {
                    break;
                };
                if self.is_item_in_history_double(idx0.clone(), idx1) {
                    continue;
                };
                best_swap = match &best_swap {
                    None => Some((idx0.clone(), idx1.clone(), i_diff)),
                    Some(best) => {
                        if best.2 < i_diff {
                            Some((idx0.clone(), idx1.clone(), i_diff))
                        } else { best_swap }
                    }
                }
            }
        }
        let best_swap = match best_swap {
            None => {
                return AutoSwap::Done;
            }
            Some(x) => x,
        };
        self.update_item_with_history_double(best_swap.0, max_g,
                                             best_swap.1, min_g);
        Continue
    }

    pub fn run_automatic_swaps(&self) {
        loop {
            match self.auto_double_swap() {
                AutoSwap::Continue => {}
                AutoSwap::Done => break,
            }
        }
    }
}

