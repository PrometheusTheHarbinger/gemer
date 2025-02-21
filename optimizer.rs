use std::collections::HashMap;
use crate::items::{Enchantment, Food};
use crate::{Bonus, Bonuses, Color, CurStats, Gem, ItemSlot, Requirement, Stat, ITEM_SLOTS_ORDER};

pub struct BestBonusFinder {
    gem_pool: Vec<Gem>,
    enchant_pool: HashMap<ItemSlot, Vec<Enchantment>>,
    food_pool: Vec<Food>,
}

impl BestBonusFinder {
    pub fn new() -> Self {
        let mut gem_pool: Vec<Gem> = Vec::new();
        gem_pool.push(Gem::new(vec![Color::Red], Bonuses::new(vec![Bonus::new(Stat::APR, 20)]), String::from("fractured")));
        gem_pool.push(Gem::new(vec![Color::Yellow], Bonuses::new(vec![Bonus::new(Stat::HasteRate, 20)]), String::from("quick")));
        gem_pool.push(Gem::new(vec![Color::Red], Bonuses::new(vec![Bonus::new(Stat::Agility, 20)]), String::from("delicate")));
        gem_pool.push(Gem::new(vec![Color::Yellow], Bonuses::new(vec![Bonus::new(Stat::HitRate, 20)]), String::from("rigid")));
        gem_pool.push(Gem::new(vec![Color::Red], Bonuses::new(vec![Bonus::new(Stat::ExpertiseRate, 20)]), String::from("precise")));
        gem_pool.push(Gem::new(vec![Color::Red, Color::Yellow], Bonuses::new(vec![Bonus::new(Stat::Agility, 10), Bonus::new(Stat::CritRate, 10)]), String::from("deadly")));
        gem_pool.push(Gem::new(vec![Color::Red, Color::Yellow], Bonuses::new(vec![Bonus::new(Stat::ExpertiseRate, 10), Bonus::new(Stat::HitRate, 10)]), String::from("accurate")));
        gem_pool.push(Gem::new(vec![Color::Red, Color::Yellow], Bonuses::new(vec![Bonus::new(Stat::Agility, 10), Bonus::new(Stat::HasteRate, 10)]), String::from("deft")));
        gem_pool.push(Gem::new(vec![Color::Red, Color::Yellow], Bonuses::new(vec![Bonus::new(Stat::Agility, 10), Bonus::new(Stat::HitRate, 10)]), String::from("glinting")));
        gem_pool.push(Gem::new(vec![Color::Red, Color::Blue], Bonuses::new(vec![Bonus::new(Stat::APR, 10)]), String::from("puissant")));
        gem_pool.push(Gem::new(vec![Color::Red, Color::Blue, Color::Yellow], Bonuses::new(vec![Bonus::new(Stat::Agility, 10), Bonus::new(Stat::Strength, 10), Bonus::new(Stat::Stamina, 10)]), String::from("Nightmare's Tear")));
        let mut enchant_pool = HashMap::new();
        for item_slot in ITEM_SLOTS_ORDER {
            enchant_pool.insert(item_slot, vec![Enchantment::new(item_slot, Bonuses::new(vec![Bonus::new(Stat::Agility, 20)]), "temp_agi".to_owned())]);
        }
        enchant_pool.entry(ItemSlot::Gloves).or_default().push(Enchantment::new(ItemSlot::Gloves, Bonuses::new(vec![Bonus::new(Stat::ExpertiseRate, 15)]), "expertise".to_owned()));
        enchant_pool.entry(ItemSlot::Gloves).or_default().push(Enchantment::new(ItemSlot::Gloves, Bonuses::new(vec![Bonus::new(Stat::AttackPower, 44)]), "crusher".to_owned()));
        enchant_pool.entry(ItemSlot::Gloves).or_default().push(Enchantment::new(ItemSlot::Gloves, Bonuses::new(vec![Bonus::new(Stat::HitRate, 20)]), "precision".to_owned()));
        let mut food_pool = Vec::new();
        food_pool.push(Food::new("crit loin".to_owned(), Bonuses::new(vec![Bonus::new(Stat::CritRate, 40)])));
        food_pool.push(Food::new("agility loin".to_owned(), Bonuses::new(vec![Bonus::new(Stat::Agility, 40)])));
        return Self{gem_pool, enchant_pool, food_pool};
    }
    fn get_gain_by_bonus(&self, bonus: &Bonus, reference: &CurStats, reqs: &[Requirement]) -> f64 {
        let mut gain: f64 = 0.0;
        for req in reqs {
            gain += match req {
                Requirement::RequirementCap(ref cap) if cap.get_stat() == bonus.get_stat() => cap.calculate_gain_incremental(bonus.get_val(), reference.get_stat_val(bonus.get_stat())),
                Requirement::RequirementWeighted(ref weighted) if weighted.get_stat() == bonus.get_stat() => weighted.calculate_gain(bonus.get_val()),
                _ => 0.0,
            }
        }
        return gain;
    }
    pub fn get_best_gem(&self, reference: &CurStats, reqs: &[Requirement], allow_tear: bool) -> Gem {
        let mut temp_gems = self.gem_pool.clone();
        temp_gems.sort_by_key(|gem| {
            let mut gain: f64 = 0.0;
            for bonus in gem.get_bonuses().iter() {
                gain += self.get_gain_by_bonus(bonus, reference, reqs);
            }
            return unsafe { gain.round().to_int_unchecked::<u32>() };
        });
        temp_gems.reverse();
        if !allow_tear && temp_gems[0].get_name() == "Nightmare's Tear" { return temp_gems[1].clone(); }
        return temp_gems[0].clone();
    }
    pub fn get_best_enchantment_by_slot(&self, slot: ItemSlot, reference: &CurStats, reqs: &[Requirement]) -> Enchantment {
        let mut temp_enchants = self.enchant_pool.get(&slot).unwrap().clone();
        temp_enchants.sort_by_key(|enchant| {
            let mut gain: f64 = 0.0;
            for bonus in enchant.get_bonuses().iter() {
                gain += self.get_gain_by_bonus(bonus, reference, reqs);
            }
            return unsafe { gain.round().to_int_unchecked::<u32>() };
        });
        temp_enchants.reverse();
        return temp_enchants[0].clone();
    }
    pub fn get_useful_food(&self, reqs: &[Requirement]) -> Vec<Food> {
        let cur_stats_clear = CurStats::new();
        self.food_pool.iter().filter(|&food| {
            let mut gain: f64 = 0.0;
            for bonus in food.get_bonuses().iter() {
                gain += self.get_gain_by_bonus(bonus, &cur_stats_clear, reqs);
            }
            return gain > 0.0;
        }).cloned().collect::<Vec<Food>>()
    }
}