use crate::Stat;
use crate::char::CurStats;

#[derive(Clone)]
pub enum Requirement {
    RequirementCap(RequirementCap),
    RequirementWeighted(RequirementWeighted),
}

#[derive(Clone)]
pub struct RequirementCap {
    stat: Stat,
    val: u32,
    weight: f64,
}

impl RequirementCap {
    pub fn new(stat: Stat, val: u32, weight: f64) -> Self {
        return Self{stat, val, weight};
    }
    pub fn get_stat(&self) -> Stat { return self.stat; }
    pub fn get_val(&self) -> u32 { return self.val; }
    pub fn get_weight(&self) -> f64 { return self.weight; }
    pub fn make_incremental(&mut self, cur_stats: &CurStats) -> () {
        self.val -= cur_stats.get_stat_val(self.stat);
    }
    pub fn calculate_gain(&self, new_val: u32) -> f64 {
        if new_val > self.val {
            return self.val as f64 * self.weight;
        } else {
            return new_val as f64 * self.weight;
        }
    }
    pub fn calculate_gain_incremental(&self, inc: u32, reference: u32) -> f64 {
        if reference > self.val { return 0.0; }
        if inc + reference > self.val {
            return (self.val-reference) as f64 * self.weight;
        } else { return inc as f64 * self.weight; }
    }
}

#[derive(Clone)]
pub struct RequirementWeighted {
    stat: Stat,
    weight: f64,
}

impl RequirementWeighted {
    pub fn new(stat: Stat, weight: f64) -> Self {
        return Self{stat, weight};
    }
    pub fn get_stat(&self) -> Stat { return self.stat; }
    pub fn get_weight(&self) -> f64 { return self.weight; }
    pub fn calculate_gain(&self, new_val: u32) -> f64 {
        return new_val as f64 * self.weight;
    }
}