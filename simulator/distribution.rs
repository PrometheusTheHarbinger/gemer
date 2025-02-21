use crate::char::CurStats;
use crate::items::{Item, Food};
use crate::simulator::requirements::Requirement;

pub struct Distribution {
    stat_growth: CurStats,
    gain: f64,
    items: Vec<Item>,
    food: Option<Food>,
}

impl Distribution {
    pub fn new(stat_growth: CurStats, reqs: &[Requirement], item_refs: &[Item]) -> Self {
        let mut items: Vec<Item> = Vec::new();
        for item in item_refs {
            items.push(item.clone());
        }
        let gain = stat_growth.calculate_gain(reqs);
        return Self{stat_growth, gain, items, food: None};
    }
    pub fn is_new_better(&self, other: &Option<&Distribution>) -> bool {
        if other.is_none() { return false; }
        return self.gain < other.unwrap().gain;
    }
    pub fn to_string(&self) -> String {
        let mut res = String::with_capacity(120);
        if self.food.is_some() {
            res += &format!("Eat {} for this gain in stats:\n", self.food.as_ref().unwrap().get_name());
        }
        for item in &self.items {
            res += &format!("[{}]{} <- {}:\n", &item.get_slot().to_string(), &item.get_name(), if item.get_enchantment().is_some() { &item.get_enchantment().as_ref().unwrap().get_name() } else { "None" });
            for socket in item.get_sockets() {
                res += &format!("\t{} <- {} ({})\n", &socket.get_color().to_string(), if socket.get_gem().is_some() { socket.get_gem().as_ref().unwrap().get_name() } else { "None" }, if socket.does_match() {"match"} else {"mismatch"});
            }
        }
        return self.stat_growth.to_string() + &res;
    }
    pub fn get_gain(&self) -> f64 {
        return self.gain;
    }
    pub fn get_stat_growth(&self) -> &CurStats {
        return &self.stat_growth;
    }
    pub fn get_items(&self) -> &[Item] {
        return &self.items;
    }
    pub fn set_food(&mut self, f: Food) -> () {
        self.food = Some(f);
    }
}