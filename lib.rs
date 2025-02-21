mod items;
mod simulator;
mod char;
mod optimizer;

pub use crate::char::{CurStats, ItemBuild, Rotatables};
pub use crate::items::{Item, Gem, GemSocket, Color};
pub use crate::simulator::{Requirement, RequirementCap, RequirementWeighted, Simulator};


#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum ItemSlot {
    Head,
    Neck,
    Shoulder,
    Back,
    Chest,
    Bracer,
    WpnMain,
    WpnOff,
    Idol,
    Gloves,
    Belt,
    Legs,
    Feet,
    Ring1,
    Ring2,
    Trinket1,
    Trinket2,
}

impl std::fmt::Display for ItemSlot {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        return std::fmt::Debug::fmt(self, f);
    }
}

const ITEM_SLOTS_ORDER: [ItemSlot; 17] = [ItemSlot::Head, ItemSlot::Neck, ItemSlot::Shoulder, ItemSlot::Back, ItemSlot::Chest, ItemSlot::Bracer, ItemSlot::WpnMain, ItemSlot::WpnOff, ItemSlot::Idol, ItemSlot::Gloves, ItemSlot::Belt, ItemSlot::Legs, ItemSlot::Feet, ItemSlot::Ring1, ItemSlot::Ring2, ItemSlot::Trinket1, ItemSlot::Trinket2];

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Stat {
    Agility,
    AttackPower,
    CritRate,
    APR,
    ExpertiseRate,
    HasteRate,
    HitRate,
    Strength,
    Stamina,
}

impl std::fmt::Display for Stat {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        return std::fmt::Debug::fmt(self, f);
    }
}

#[derive(Clone)]
pub struct Bonus {
    stat: Stat,
    val: u32,
}

impl Bonus {
    pub fn new(stat: Stat, val: u32) -> Self {
        return Self{stat, val};
    }
    pub fn new_proccable(stat: Stat, val: u32, duration: f64, cooldown: f64) -> Self {
        let true_val = val as f64 * duration / cooldown;
        return Self{stat, val: true_val.round() as u32};
    }
    pub fn apply_bonus(&self, cur_stats: &mut CurStats) -> () {
        cur_stats.add_stat(self.stat, self.val);
    }
    pub fn get_stat(&self) -> Stat { return self.stat; }
    pub fn get_val(&self) -> u32 { return self.val; } 
}

#[derive(Clone)]
pub struct Bonuses {
    bonuses: Vec<Bonus>,
}

impl Bonuses {
    pub fn new(bonuses: Vec<Bonus>) -> Self {
        return Self{bonuses};
    }
    pub fn apply_bonuses(&self, cur_stats: &mut CurStats) -> () {
        for bon in &self.bonuses {
            bon.apply_bonus(cur_stats);
        }
    }
    pub fn iter(&self) -> std::slice::Iter<Bonus> {
        return self.bonuses.iter();
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn does_bonus_apply() {
        let mut stats = CurStats::new();
        Bonus::new(Stat::Agility, 17).apply_bonus(&mut stats);
        assert_eq!(stats.get_stat_val(Stat::Agility), 17);
    }
    #[test]
    fn does_bonuses_apply() {
        let mut stats = CurStats::new();
        Bonuses::new(vec![Bonus::new(Stat::Agility, 9), Bonus::new(Stat::Agility, 8)]).apply_bonuses(&mut stats);
        assert_eq!(stats.get_stat_val(Stat::Agility), 17);
    }
    #[test]
    fn bonus_procs() {
        assert_eq!(Bonus::new_proccable(Stat::AttackPower, 400, 15.0, 60.0).get_val(), 100);
    }
}