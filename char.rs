use std::collections::HashMap;
use crate::{ItemSlot, Stat, ITEM_SLOTS_ORDER};
use crate::items::Item;
use crate::simulator::Requirement;
use itertools::Itertools;


pub struct ItemBuild {
    build: HashMap<ItemSlot, Option<Item>>,
} 

impl ItemBuild {
    pub fn new() -> Self {
        return Self{build: HashMap::<ItemSlot, Option<Item>>::new()};
    }
    pub fn lock_item(&mut self, item: Item) -> () {
        let slot = item.get_slot();
        *self.build.entry(slot).or_insert(None) = Some(item);
    }
    pub fn item_iter(&self) -> ItemBuildIter {
        return ItemBuildIter{curr_pos: 0, target: self};
    }
    pub fn get_item(&self, slot: ItemSlot) -> &Option<Item> {
        if self.build.contains_key(&slot) { return self.build.get(&slot).unwrap(); } else { return &None }
    }
    pub fn clear(&mut self) -> () {
        self.build.clear();
    }
}

pub struct ItemBuildIter<'a> {
    curr_pos: usize,
    target: &'a ItemBuild,
}

impl<'a> Iterator for ItemBuildIter<'a> {
    type Item = (ItemSlot, &'a Option<Item>);
    fn next(&mut self) -> Option<Self::Item> {
        if self.curr_pos == ITEM_SLOTS_ORDER.len() { return None; }
        let res = Some((ITEM_SLOTS_ORDER[self.curr_pos], self.target.get_item(ITEM_SLOTS_ORDER[self.curr_pos])));
        self.curr_pos += 1;
        return res;
    }
}

pub struct Rotatables {
    rotatable_items: HashMap<ItemSlot, Vec<Item>>,
    which_slots_to_rotate: Vec<ItemSlot>,
}

impl Rotatables {
    pub fn new() -> Self {
        return Self{rotatable_items: HashMap::new(), which_slots_to_rotate: vec![]};
    }
    pub fn rotate(&mut self, item: Item) -> () {
        if !self.which_slots_to_rotate.contains(&item.get_slot()) { self.which_slots_to_rotate.push(item.get_slot()); }
        self.rotatable_items.entry(item.get_slot()).or_insert(vec![]).push(item);
    }
    pub fn slots_in_rotation(&self) -> &[ItemSlot] {
        return &self.which_slots_to_rotate;
    }
    pub fn get_from_slot(&self, slot: ItemSlot) -> &[Item] {
        return self.rotatable_items.get(&slot).unwrap();
    }
    pub fn iter_variants(&self) -> RotateVariantsGenerator {
        return RotateVariantsGenerator{rotatables: &self, to_skip: 0};
    }
}

pub struct RotateVariantsGenerator<'a> {
    to_skip: usize,
    rotatables: &'a Rotatables,
}

impl<'a> Iterator for RotateVariantsGenerator<'a> {
    type Item = Vec<&'a Item>;
    fn next(&mut self) -> Option<Self::Item> {
        let res = self.rotatables.slots_in_rotation().iter().map(|slot| self.rotatables.get_from_slot(*slot)).multi_cartesian_product().skip(self.to_skip).next();
        self.to_skip += 1;
        return res;
    }
}

#[derive(Clone)]
pub struct CurStats {
    stats: HashMap<Stat, u32>,
}

impl CurStats {
    pub fn new() -> Self {
        return Self{stats: HashMap::<Stat, u32>::new()};
    }
    pub fn set_stat(&mut self, stat: Stat, val: u32) -> () {
        *self.stats.entry(stat).or_insert(0) = val;
    }
    pub fn add_stat(&mut self, stat: Stat, val: u32) -> () {
        *self.stats.entry(stat).or_insert(0) += val;
    }
    pub fn get_stat_val(&self, stat: Stat) -> u32 {
        if self.stats.contains_key(&stat) { *self.stats.get(&stat).unwrap() } else { 0 }
    }
    pub fn clear(&mut self) -> () {
        self.stats.clear();
    }
    pub fn to_string(&self) -> String {
        let mut res = String::with_capacity(200);
        for (k, v) in self.stats.iter() {
            res += &format!("{k}: {v}\n", k=&k.to_string(), v=&v.to_string());
        }
        return res;
    }
    pub fn iter_stats(&self) -> std::collections::hash_map::Iter<Stat, u32> {
        return self.stats.iter();
    }
    pub fn sum_of(&self, other: &CurStats) -> CurStats {
        let mut res = self.clone();
        for (k, v) in other.iter_stats() {
            res.add_stat(*k, *v);
        }
        return res;
    }
    pub fn calculate_gain(&self, reqs: &[Requirement]) -> f64 {
        let mut gain: f64 = 0.0;
        for req in reqs {
            match req {
                Requirement::RequirementCap(cap) => gain += cap.calculate_gain(self.get_stat_val(cap.get_stat())),
                Requirement::RequirementWeighted(weighted) => gain += weighted.calculate_gain(self.get_stat_val(weighted.get_stat())),
            }
        }
        return gain;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RequirementWeighted;

    #[test]
    fn gain_correct() {
        let reqs = vec![Requirement::RequirementWeighted(RequirementWeighted::new(Stat::Agility, 10.0)), Requirement::RequirementWeighted(RequirementWeighted::new(Stat::HasteRate, 9.0))];
        let mut sp = CurStats::new();
        sp.set_stat(Stat::HasteRate, 10);
        sp.set_stat(Stat::Agility, 100);
        assert_eq!(sp.calculate_gain(&reqs), 1090.0);
    }
    #[test]
    fn incremental_cloning() {
        let mut sp = CurStats::new();
        sp.set_stat(Stat::HasteRate, 10);
        sp.set_stat(Stat::Agility, 10);
        let mut sp2 = CurStats::new();
        sp2.set_stat(Stat::CritRate, 10);
        sp2.set_stat(Stat::Agility, 10);
        let res = sp.sum_of(&sp2);
        assert!(res.get_stat_val(Stat::CritRate) == 10 && res.get_stat_val(Stat::HasteRate) == 10 && res.get_stat_val(Stat::Agility) == 20);
    }
}