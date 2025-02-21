use crate::{Bonus, Bonuses, ItemSlot};
use crate::char::CurStats;


#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Color {
    Red,
    Blue,
    Yellow,
}

impl std::fmt::Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        return std::fmt::Debug::fmt(self, f);
    }
}

#[derive(Clone)]
pub struct Item {
    name: String,
    slot: ItemSlot,
    stats: Bonuses,
    sockets: Vec<GemSocket>,
    socket_bonus: Option<Bonus>,
    enchant: Option<Enchantment>,
}

impl Item {
    pub fn new(name: String, slot: ItemSlot, stats: Bonuses, sockets: Vec<GemSocket>, socket_bonus: Option<Bonus>, enchant: Option<Enchantment>) -> Self {
        return Self{name, slot, stats, sockets, socket_bonus, enchant};
    }
    pub fn sockets_match(&self) -> bool {
        for socket in &self.sockets {
            if !socket.does_match() { return false; }
        }
        return true;
    }
    pub fn get_name(&self) -> &str {
        return &self.name;
    }
    pub fn get_slot(&self) -> ItemSlot {
        return self.slot;
    }
    pub fn get_stats_bonuses(&self) -> &Bonuses {
        return &self.stats;
    }
    pub fn get_sockets(&self) -> &Vec<GemSocket> {
        return &self.sockets;
    }
    pub fn get_socket_mut(&mut self, ind: usize) -> &mut GemSocket {
        return &mut self.sockets[ind];
    }
    pub fn get_socket_bonus(&self) -> &Option<Bonus> {
        return &self.socket_bonus;
    }
    pub fn apply_socket_bonus(&self, cur_stats: &mut CurStats) -> () {
        if let Some(ref bonus) = self.socket_bonus {
            bonus.apply_bonus(cur_stats);
        }
    }
    pub fn is_enchanted(&self) -> bool {
        return self.enchant.is_some();
    }
    pub fn get_enchantment(&self) -> &Option<Enchantment> { return &self.enchant; }
    pub fn set_enchantment(&mut self, enchant: &Enchantment) -> () {
        self.enchant = Some(enchant.clone());
    }
    pub fn remove_enchantment(&mut self) -> () {
        self.enchant = None;
    }
}

#[derive(Clone)]
pub struct Gem {
    colors: Vec<Color>,
    bonuses: Bonuses,
    name: String,
}

impl Gem {
    pub fn new(colors: Vec<Color>, bonuses: Bonuses, name: String) -> Self {
        return Self{colors, bonuses, name};
    }
    pub fn get_colors(&self) -> &Vec<Color> { return &self.colors; }
    pub fn get_bonuses(&self) -> &Bonuses { return &self.bonuses; }
    pub fn get_name(&self) -> &str { return &self.name; } 
}

#[derive(Clone)]
pub struct GemSocket {
    color: Color,
    gem: Option<Gem>,
}

impl GemSocket {
    pub fn new(color: Color) -> Self {
        return Self{color, gem: None};
    }
    pub fn get_color(&self) -> Color { return self.color; } 
    pub fn get_gem(&self) -> &Option<Gem> { return &self.gem; }
    pub fn is_empty(&self) -> bool { if self.gem.is_none() { return true; } else { return false; }}
    pub fn set_gem(&mut self, gem: &Gem) -> () { self.gem = Some(gem.clone()); }
    pub fn set_empty(&mut self) -> () { self.gem = None; }
    pub fn does_match(&self) -> bool {
        if self.gem.is_none() { return false; }
        if self.gem.as_ref().unwrap().get_colors().contains(&self.color) { return true; } else { return false; }
    }
}

#[derive(Clone)]
pub struct Enchantment {
    slot: ItemSlot,
    bonuses: Bonuses,
    name: String,
}

impl Enchantment {
    pub fn new(slot: ItemSlot, bonuses: Bonuses, name: String) -> Self {
        return Self{slot, bonuses, name};
    }
    pub fn get_slot(&self) -> ItemSlot { return self.slot; }
    pub fn get_bonuses(&self) -> &Bonuses { return &self.bonuses; }
    pub fn get_name(&self) -> &str { return &self.name; } 
}

#[derive(Clone)]
pub struct Food {
    name: String,
    bonuses: Bonuses,
}

impl Food {
    pub fn new(name: String, bonuses: Bonuses) -> Self {
        return Self{name, bonuses};
    }
    pub fn get_bonuses(&self) -> &Bonuses { return &self.bonuses; }
    pub fn get_name(&self) -> &str { return &self.name; }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matching_sockets() {
        let v = vec![Gem::new(vec![Color::Red], Bonuses::new(vec![]), String::new()), Gem::new(vec![Color::Yellow, Color::Blue], Bonuses::new(vec![]), String::new()), Gem::new(vec![Color::Yellow], Bonuses::new(vec![]), String::new())];
        let mut item = Item::new(String::new(), ItemSlot::Feet, Bonuses::new(vec![]), vec![GemSocket::new(Color::Red), GemSocket::new(Color::Blue), GemSocket::new(Color::Yellow)], None, None);
        for ind_socket in 0..item.get_sockets().len() {
            item.get_socket_mut(ind_socket).set_gem(&v[ind_socket]);
        }
        assert!(item.sockets_match());
    }
}