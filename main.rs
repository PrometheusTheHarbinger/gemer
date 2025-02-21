use gemer::*;

fn main() {
    let mut sp = CurStats::new();
    sp.set_stat(Stat::Strength, 113);
    sp.set_stat(Stat::Agility, 2095);
    sp.set_stat(Stat::APR, 1345);
    sp.set_stat(Stat::AttackPower, 9944);
    sp.set_stat(Stat::ExpertiseRate, 106);
    sp.set_stat(Stat::CritRate, 1890);
    sp.set_stat(Stat::HitRate, 201);
    sp.set_stat(Stat::HasteRate, 340);
    let mut my_build = ItemBuild::new();
    let item = Item::new(String::from("neck"), ItemSlot::Neck, Bonuses::new(vec![]), vec![GemSocket::new(Color::Blue)], Some(Bonus::new(Stat::Agility, 4)), None);	
    my_build.lock_item(item);
    let item = Item::new(String::from("chest"), ItemSlot::Chest, Bonuses::new(vec![]), vec![GemSocket::new(Color::Yellow)], Some(Bonus::new(Stat::Agility, 6)), None);	
    my_build.lock_item(item);
    let item = Item::new(String::from("belt"), ItemSlot::Belt, Bonuses::new(vec![]), vec![GemSocket::new(Color::Red)], Some(Bonus::new(Stat::Agility, 6)), None);	
    my_build.lock_item(item);
    let item = Item::new(String::from("legs"), ItemSlot::Legs, Bonuses::new(vec![]), vec![GemSocket::new(Color::Yellow)], Some(Bonus::new(Stat::Agility, 6)), None);	
    my_build.lock_item(item);
    let item = Item::new(String::from("feet"), ItemSlot::Feet, Bonuses::new(vec![]), vec![GemSocket::new(Color::Yellow), GemSocket::new(Color::Yellow)], Some(Bonus::new(Stat::Agility, 6)), None);	
    my_build.lock_item(item);
    let item = Item::new(String::from("gloves"), ItemSlot::Gloves, Bonuses::new(vec![]), vec![GemSocket::new(Color::Yellow)], Some(Bonus::new(Stat::Agility, 6)), None);	
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
    let now = std::time::Instant::now();
    sim.run(true, true, true, true, false);
    let elapsed_time = now.elapsed();
    println!("Running took {} seconds.", elapsed_time.as_millis() as f64 /1000.0);
    println!("{}Total gain of this build is {}", sim.result(), sim.get_gain()); // 6665.45
}
