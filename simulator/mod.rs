mod requirements;
mod distribution;

use crate::optimizer::BestBonusFinder;
pub use crate::simulator::requirements::{Requirement, RequirementCap, RequirementWeighted};
use crate::simulator::distribution::Distribution;
use crate::char::{CurStats, ItemBuild, Rotatables, RotateVariantsGenerator};
use crate::items::Item;
use std::thread;
use std::sync::Arc;

pub struct Simulator {
    reqs: Vec<Requirement>,
    build: ItemBuild,
    distribution: Option<Distribution>,
    rotatable: Option<Rotatables>,
}

impl Simulator {
    pub fn new(cur_stats: CurStats, mut reqs: Vec<Requirement>, build: ItemBuild, rotatable: Option<Rotatables>) -> Self {
        for req in reqs.iter_mut() {
            if let Requirement::RequirementCap(ref mut cap) = req {
                cap.make_incremental(&cur_stats);
            }
        }
        return Self{reqs, build, distribution: None, rotatable};
    }
    fn next_build(&mut self, to_skip: usize) -> Option<CurStats> {
        let mut to_return = CurStats::new();
        let rotator: RotateVariantsGenerator;
        if self.rotatable.is_some() {
            rotator = self.rotatable.as_ref().unwrap().iter_variants();
        } else {
            // If there's nothing to rotate and it's the first call, we use the build we already have. Since None indicates end, we should return empty stats instead. 
            // On further calls None is the way to go.
            if to_skip == 0 { return Some(to_return); } else { return None; } 
        }
        // Essentialy variants[to_skip].
        let items_to_lock = rotator.skip(to_skip).next();
        if items_to_lock.is_some() {
            for rotatable_item in items_to_lock.unwrap() {
                rotatable_item.get_stats_bonuses().apply_bonuses(&mut to_return);
                self.build.lock_item(rotatable_item.clone());
            }
            return Some(to_return);
        } else { return None; }
    }
    pub fn run(&mut self, enable_gems: bool, enable_chants: bool, enable_food: bool, enable_prechant: bool, allow_tear: bool) -> () {
        let optimizer = Arc::new(BestBonusFinder::new());
        let reqs_arc = Arc::new(self.reqs.clone());
        for food in optimizer.get_useful_food(&reqs_arc) {
            // This cycle works only once if food is disabled, so it's fine.
            let mut skip_that_much_variants = 0;
            loop {
                let cur_stats = self.next_build(skip_that_much_variants);
                if cur_stats.is_none() {
                    break;
                } else {
                    let mut cur_stats = cur_stats.unwrap();
                    skip_that_much_variants += 1;
                    // At the end of the loop this variant of equipment will already be processed, we will need the next one.
                    if enable_food {
                        food.get_bonuses().apply_bonuses(&mut cur_stats);
                    }
                    let mut main_state: Vec<Item> = Vec::new();
                    for (_, opt_item) in self.build.item_iter() {
                        if opt_item.is_some() { main_state.push(opt_item.as_ref().unwrap().clone()); }
                    }
                    let mut distr: Option<Distribution> = None;
                    if enable_chants && !enable_gems {
                        distr = Some(prechants_only(main_state.clone(), cur_stats.clone(), &reqs_arc, &optimizer));
                    }
                    if !enable_chants && enable_gems {
                        distr = Some(solve_recursively(main_state.clone(), cur_stats.clone(), &reqs_arc, &optimizer, false, allow_tear, true));
                    }
                    if enable_chants && enable_gems && !enable_prechant {
                        distr = Some(solve_recursively(main_state.clone(), cur_stats.clone(), &reqs_arc, &optimizer, true, allow_tear, true));
                    }
                    if enable_chants && enable_gems && enable_prechant {
                        let chanted_distr = prechants_only(main_state.clone(), cur_stats.clone(), &reqs_arc, &optimizer);
                        let distr_pre = solve_recursively(chanted_distr.get_items().to_vec(), chanted_distr.get_stat_growth().clone(), &reqs_arc, &optimizer, false, allow_tear, true);
                        // Firstly chanting, then gems, second time both at the same time.
                        let distr_post = solve_recursively(main_state, cur_stats.clone(), &reqs_arc, &optimizer, true, allow_tear, true);
                        distr = if distr_pre.is_new_better(&Some(&distr_post)) { Some(distr_post) } else { Some(distr_pre) };
                    }
                    if self.distribution.is_none() || self.distribution.as_ref().unwrap().is_new_better(&distr.as_ref()) {
                        self.distribution = distr;
                        if enable_food { self.distribution.as_mut().unwrap().set_food(food.clone()) }
                    }
                }
            }
            if !enable_food { return; }
        }
    }
    pub fn result(&self) -> String {
        match self.distribution {
            None => String::new(),
            Some(ref dis) => dis.to_string(),
        }
    }
    pub fn get_gain(&self) -> f64 {
        match self.distribution {
            None => 0.0,
            Some(ref dis) => dis.get_gain(),
        }
    }
}

fn prechants_only(main_state: Vec<Item>, mut cur_stats: CurStats, reqs: &Arc<Vec<Requirement>>, optimizer: &Arc<BestBonusFinder>) -> Distribution {
    let mut only_chants = Vec::new();
    for (_item_ind, mut item) in main_state.iter().cloned().enumerate() {
        if !item.is_enchanted() {
            let enchantment = optimizer.get_best_enchantment_by_slot(item.get_slot(), &cur_stats, &reqs);
            item.set_enchantment(&enchantment);
        }
        item.get_enchantment().as_ref().unwrap().get_bonuses().apply_bonuses(&mut cur_stats);
        only_chants.push(item);
    }
    return Distribution::new(cur_stats.clone(), &reqs, &only_chants);
}

fn solve_recursively(main_state: Vec<Item>, cur_stats: CurStats, reqs: &Arc<Vec<Requirement>>, optimizer: &Arc<BestBonusFinder>, try_chanting: bool, allow_tear: bool, concurrent: bool) -> Distribution {
    let mut distr_best = Distribution::new(cur_stats.clone(), reqs, &main_state);
    let allow_tear_further = allow_tear && !main_state.iter().any(|item| item.get_sockets().iter().any(|socket| if socket.get_gem().is_none() { false } else { socket.get_gem().as_ref().unwrap().get_name() == "Nightmare's Tear" }));
    let mut resulting_distrs = Vec::new();
    let mut thread_handlers = Vec::new();
    // TLDR: for each gem socket in each item fill gem, add its bonuses, chant item if needed, swap original item with chanted and gemmed, then solve for this state, in separate thread or not.
    for (item_ind, mut item) in main_state.iter().cloned().enumerate() {
        for ind_socket in 0..item.get_sockets().len() {
            if !item.get_socket_mut(ind_socket).is_empty() { continue; }
            let mut temp_stats = cur_stats.clone();
            let best_gem = optimizer.get_best_gem(&temp_stats, reqs, allow_tear_further);
            item.get_socket_mut(ind_socket).set_gem(&best_gem);
            best_gem.get_bonuses().apply_bonuses(&mut temp_stats);
            if try_chanting && !item.is_enchanted() {
                let enchantment = optimizer.get_best_enchantment_by_slot(item.get_slot(), &temp_stats, reqs);
                item.set_enchantment(&enchantment);
                enchantment.get_bonuses().apply_bonuses(&mut temp_stats);
            }
            let mut altered_state = main_state.clone();
            let item_tmp = item.clone();
            altered_state[item_ind] = item_tmp;
            if item.sockets_match() { item.apply_socket_bonus(&mut temp_stats); }
            if concurrent {
                let state_threaded = altered_state.clone();
                let stats_threaded = temp_stats.clone();
                let reqs_threaded = Arc::clone(reqs);
                let optimizer_threaded = Arc::clone(optimizer);
                thread_handlers.push(thread::spawn(move || {
                    return solve_recursively(state_threaded, stats_threaded, &reqs_threaded, &optimizer_threaded, try_chanting, allow_tear_further, false);
                }));
            } else {
                resulting_distrs.push(solve_recursively(altered_state, temp_stats, reqs, optimizer, try_chanting, allow_tear_further, false));
            }
            item.get_socket_mut(ind_socket).set_empty();
            if try_chanting { item.remove_enchantment(); }
        }
    }
    // Now collect and compare results.
    if concurrent {
        for handler in thread_handlers {
            resulting_distrs.push(handler.join().unwrap_or(Distribution::new(cur_stats.clone(), reqs, &main_state)));
        }
    }
    for distr in resulting_distrs {
        if distr_best.is_new_better(&Some(&distr)) { distr_best = distr; }
    }
    return distr_best;
}

#[cfg(test)]
mod tests {
    use crate::Stat;
    use super::*;

    #[test]
    fn caps_incrementary() {
        let mut cap = RequirementCap::new(Stat::APR, 1400, 2.35);
        let mut state = CurStats::new();
        state.set_stat(Stat::APR, 1383);
        cap.make_incremental(&state);
        assert_eq!(cap.get_val(), 17);
    }
    #[test]
    fn all_systems_go() {
        use crate::char::{CurStats, ItemBuild, Rotatables};
        use crate::items::{Item, GemSocket, Color};
        use crate::{Bonus, Bonuses, ItemSlot, Stat};
        let mut sp = CurStats::new();
        sp.set_stat(Stat::APR, 1345);
        sp.set_stat(Stat::ExpertiseRate, 106);
        sp.set_stat(Stat::HitRate, 201);
        let mut my_build = ItemBuild::new();
        let item = Item::new(String::from("neck"), ItemSlot::Neck, Bonuses::new(vec![]), vec![GemSocket::new(Color::Blue)], Some(Bonus::new(Stat::Agility, 4)), None);	
        my_build.lock_item(item);
        let item = Item::new(String::from("chest"), ItemSlot::Chest, Bonuses::new(vec![]), vec![GemSocket::new(Color::Yellow)], Some(Bonus::new(Stat::Agility, 6)), None);	
        my_build.lock_item(item);
        let item = Item::new(String::from("feet"), ItemSlot::Feet, Bonuses::new(vec![]), vec![GemSocket::new(Color::Yellow), GemSocket::new(Color::Yellow)], Some(Bonus::new(Stat::Agility, 6)), None);	
        my_build.lock_item(item);
        let mut reqs: Vec<Requirement> = vec![];
        reqs.push(Requirement::RequirementCap(RequirementCap::new(Stat::APR, 1400, 100.0)));
        reqs.push(Requirement::RequirementCap(RequirementCap::new(Stat::ExpertiseRate, 132, 2.19)));
        reqs.push(Requirement::RequirementCap(RequirementCap::new(Stat::HitRate, 230, 2.19)));
        reqs.push(Requirement::RequirementWeighted(RequirementWeighted::new(Stat::HasteRate, 1.5)));
        reqs.push(Requirement::RequirementWeighted(RequirementWeighted::new(Stat::Agility, 1.91)));
        reqs.push(Requirement::RequirementWeighted(RequirementWeighted::new(Stat::CritRate, 1.42)));
        let mut rotatable = Rotatables::new();
        rotatable.rotate(Item::new(String::from("ring1_1"), ItemSlot::Ring1, Bonuses::new(vec![Bonus::new(Stat::Agility, 0)]), vec![GemSocket::new(Color::Yellow)], Some(Bonus::new(Stat::Agility, 6)), None));
        rotatable.rotate(Item::new(String::from("ring1_2"), ItemSlot::Ring1, Bonuses::new(vec![Bonus::new(Stat::Agility, 100)]), vec![GemSocket::new(Color::Red)], Some(Bonus::new(Stat::Agility, 60)), None));
        rotatable.rotate(Item::new(String::from("ring2_1"), ItemSlot::Ring2, Bonuses::new(vec![Bonus::new(Stat::Agility, 100)]), vec![GemSocket::new(Color::Yellow)], Some(Bonus::new(Stat::Agility, 6)), None));
        rotatable.rotate(Item::new(String::from("ring2_2"), ItemSlot::Ring2, Bonuses::new(vec![Bonus::new(Stat::Agility, 0)]), vec![GemSocket::new(Color::Red)], Some(Bonus::new(Stat::Agility, 0)), None));
        let mut sim = Simulator::new(sp, reqs, my_build, Some(rotatable));
        sim.run(true, true, true, true, false);
        assert_eq!(6413.33, sim.distribution.expect("No solutions were found!").get_gain());
    }
    #[test]
    fn rotateless() {
        use crate::char::{CurStats, ItemBuild};
        use crate::items::{Item, GemSocket, Color};
        use crate::{Bonus, Bonuses, ItemSlot, Stat};
        let mut sp = CurStats::new();
        sp.set_stat(Stat::APR, 1345);
        sp.set_stat(Stat::ExpertiseRate, 106);
        sp.set_stat(Stat::HitRate, 201);
        let mut my_build = ItemBuild::new();
        let item = Item::new(String::from("neck"), ItemSlot::Neck, Bonuses::new(vec![]), vec![GemSocket::new(Color::Blue)], Some(Bonus::new(Stat::Agility, 4)), None);	
        my_build.lock_item(item);
        let item = Item::new(String::from("chest"), ItemSlot::Chest, Bonuses::new(vec![]), vec![GemSocket::new(Color::Yellow)], Some(Bonus::new(Stat::Agility, 6)), None);	
        my_build.lock_item(item);
        let item = Item::new(String::from("feet"), ItemSlot::Feet, Bonuses::new(vec![]), vec![GemSocket::new(Color::Yellow), GemSocket::new(Color::Yellow)], Some(Bonus::new(Stat::Agility, 6)), None);	
        my_build.lock_item(item);
        let mut reqs: Vec<Requirement> = vec![];
        reqs.push(Requirement::RequirementCap(RequirementCap::new(Stat::APR, 1400, 100.0)));
        reqs.push(Requirement::RequirementCap(RequirementCap::new(Stat::ExpertiseRate, 132, 2.19)));
        reqs.push(Requirement::RequirementCap(RequirementCap::new(Stat::HitRate, 230, 2.19)));
        reqs.push(Requirement::RequirementWeighted(RequirementWeighted::new(Stat::HasteRate, 1.5)));
        reqs.push(Requirement::RequirementWeighted(RequirementWeighted::new(Stat::Agility, 1.91)));
        reqs.push(Requirement::RequirementWeighted(RequirementWeighted::new(Stat::CritRate, 1.42)));
        let mut sim = Simulator::new(sp, reqs, my_build, None);
        sim.run(true, true, true, true, false);
        assert_eq!(5746.259999999999, sim.distribution.expect("No solutions were found!").get_gain());
    }
    #[test]
    fn rotateless_and_foodless() {
        use crate::char::{CurStats, ItemBuild};
        use crate::items::{Item, GemSocket, Color};
        use crate::{Bonus, Bonuses, ItemSlot, Stat};
        let mut sp = CurStats::new();
        sp.set_stat(Stat::APR, 1345);
        sp.set_stat(Stat::ExpertiseRate, 106);
        sp.set_stat(Stat::HitRate, 201);
        let mut my_build = ItemBuild::new();
        let item = Item::new(String::from("neck"), ItemSlot::Neck, Bonuses::new(vec![]), vec![GemSocket::new(Color::Blue)], Some(Bonus::new(Stat::Agility, 4)), None);	
        my_build.lock_item(item);
        let item = Item::new(String::from("chest"), ItemSlot::Chest, Bonuses::new(vec![]), vec![GemSocket::new(Color::Yellow)], Some(Bonus::new(Stat::Agility, 6)), None);	
        my_build.lock_item(item);
        let item = Item::new(String::from("feet"), ItemSlot::Feet, Bonuses::new(vec![]), vec![GemSocket::new(Color::Yellow), GemSocket::new(Color::Yellow)], Some(Bonus::new(Stat::Agility, 6)), None);	
        my_build.lock_item(item);
        let mut reqs: Vec<Requirement> = vec![];
        reqs.push(Requirement::RequirementCap(RequirementCap::new(Stat::APR, 1400, 100.0)));
        reqs.push(Requirement::RequirementCap(RequirementCap::new(Stat::ExpertiseRate, 132, 2.19)));
        reqs.push(Requirement::RequirementCap(RequirementCap::new(Stat::HitRate, 230, 2.19)));
        reqs.push(Requirement::RequirementWeighted(RequirementWeighted::new(Stat::HasteRate, 1.5)));
        reqs.push(Requirement::RequirementWeighted(RequirementWeighted::new(Stat::Agility, 1.91)));
        reqs.push(Requirement::RequirementWeighted(RequirementWeighted::new(Stat::CritRate, 1.42)));
        let mut sim = Simulator::new(sp, reqs, my_build, None);
        sim.run(true, true, false, true, false);
        assert_eq!(5669.86, sim.distribution.expect("No solutions were found!").get_gain());
    }
    #[test]
    fn only_chants() {
        use crate::char::{CurStats, ItemBuild};
        use crate::items::{Item, GemSocket, Color};
        use crate::{Bonus, Bonuses, ItemSlot, Stat};
        let mut sp = CurStats::new();
        sp.set_stat(Stat::APR, 1345);
        sp.set_stat(Stat::ExpertiseRate, 106);
        sp.set_stat(Stat::HitRate, 201);
        let mut my_build = ItemBuild::new();
        let item = Item::new(String::from("neck"), ItemSlot::Neck, Bonuses::new(vec![]), vec![GemSocket::new(Color::Blue)], Some(Bonus::new(Stat::Agility, 4)), None);	
        my_build.lock_item(item);
        let item = Item::new(String::from("chest"), ItemSlot::Chest, Bonuses::new(vec![]), vec![GemSocket::new(Color::Yellow)], Some(Bonus::new(Stat::Agility, 6)), None);	
        my_build.lock_item(item);
        let item = Item::new(String::from("feet"), ItemSlot::Feet, Bonuses::new(vec![]), vec![GemSocket::new(Color::Yellow), GemSocket::new(Color::Yellow)], Some(Bonus::new(Stat::Agility, 6)), None);	
        my_build.lock_item(item);
        let mut reqs: Vec<Requirement> = vec![];
        reqs.push(Requirement::RequirementCap(RequirementCap::new(Stat::APR, 1400, 100.0)));
        reqs.push(Requirement::RequirementCap(RequirementCap::new(Stat::ExpertiseRate, 132, 2.19)));
        reqs.push(Requirement::RequirementCap(RequirementCap::new(Stat::HitRate, 230, 2.19)));
        reqs.push(Requirement::RequirementWeighted(RequirementWeighted::new(Stat::HasteRate, 1.5)));
        reqs.push(Requirement::RequirementWeighted(RequirementWeighted::new(Stat::Agility, 1.91)));
        reqs.push(Requirement::RequirementWeighted(RequirementWeighted::new(Stat::CritRate, 1.42)));
        let mut sim = Simulator::new(sp, reqs, my_build, None);
        sim.run(false, true, false, true, false);
        assert_eq!(114.6, sim.distribution.expect("No solutions were found!").get_gain());
    }
    #[test]
    fn only_gems() {
        use crate::char::{CurStats, ItemBuild};
        use crate::items::{Item, GemSocket, Color};
        use crate::{Bonus, Bonuses, ItemSlot, Stat};
        let mut sp = CurStats::new();
        sp.set_stat(Stat::APR, 1345);
        sp.set_stat(Stat::ExpertiseRate, 106);
        sp.set_stat(Stat::HitRate, 201);
        let mut my_build = ItemBuild::new();
        let item = Item::new(String::from("neck"), ItemSlot::Neck, Bonuses::new(vec![]), vec![GemSocket::new(Color::Blue)], Some(Bonus::new(Stat::Agility, 4)), None);	
        my_build.lock_item(item);
        let item = Item::new(String::from("chest"), ItemSlot::Chest, Bonuses::new(vec![]), vec![GemSocket::new(Color::Yellow)], Some(Bonus::new(Stat::Agility, 6)), None);	
        my_build.lock_item(item);
        let item = Item::new(String::from("feet"), ItemSlot::Feet, Bonuses::new(vec![]), vec![GemSocket::new(Color::Yellow), GemSocket::new(Color::Yellow)], Some(Bonus::new(Stat::Agility, 6)), None);	
        my_build.lock_item(item);
        let mut reqs: Vec<Requirement> = vec![];
        reqs.push(Requirement::RequirementCap(RequirementCap::new(Stat::APR, 1400, 100.0)));
        reqs.push(Requirement::RequirementCap(RequirementCap::new(Stat::ExpertiseRate, 132, 2.19)));
        reqs.push(Requirement::RequirementCap(RequirementCap::new(Stat::HitRate, 230, 2.19)));
        reqs.push(Requirement::RequirementWeighted(RequirementWeighted::new(Stat::HasteRate, 1.5)));
        reqs.push(Requirement::RequirementWeighted(RequirementWeighted::new(Stat::Agility, 1.91)));
        reqs.push(Requirement::RequirementWeighted(RequirementWeighted::new(Stat::CritRate, 1.42)));
        let mut sim = Simulator::new(sp, reqs, my_build, None);
        sim.run(true, false, false, false, false);
        assert_eq!(5555.259999999999, sim.distribution.expect("No solutions were found!").get_gain());
    }
}