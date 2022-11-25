#[cfg(test)]
#[tokio::test]
async fn it_works() {
    use log::info;
    use nohash_hasher::IntMap;

    use crate::{
        calc::{calc_data_manager::CalcDataManager, constant::MAX_SLOT_LEVEL},
        calculate_skillset,
        data::data_manager::DataManager,
    };

    let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf();

    let dm = DataManager::from_sources(
        &manifest_dir.join("../src-tauri/data/armor.json"),
        &manifest_dir.join("../src-tauri/data/skill.json"),
        &manifest_dir.join("../src-tauri/data/deco.json"),
    )
    .await;

    let cm = CalcDataManager::new(&dm);

    info!("Armors length: {}", dm.get_armors().len());

    let mut selected_skills = IntMap::default();
    let weapon_slots = vec![3, 0, 0];
    let free_slots = vec![0; MAX_SLOT_LEVEL];

    selected_skills.insert(dm.get_skill_uid("water_attack"), 5);
    selected_skills.insert(dm.get_skill_uid("element_exploit"), 1);

    selected_skills.insert(dm.get_skill_uid("bow_charge_plus"), 1);
    selected_skills.insert(dm.get_skill_uid("spread_up"), 3);
    selected_skills.insert(dm.get_skill_uid("reload_speed"), 2);

    selected_skills.insert(dm.get_skill_uid("weakness_exploit"), 3);
    selected_skills.insert(dm.get_skill_uid("burst"), 2);

    selected_skills.insert(dm.get_skill_uid("constitution"), 5);
    selected_skills.insert(dm.get_skill_uid("stamina_surge"), 3);

    selected_skills.insert(dm.get_skill_uid("attack_boost"), 4);
    selected_skills.insert(dm.get_skill_uid("bloodlust"), 1);
    selected_skills.insert(dm.get_skill_uid("charge_master"), 1);
    selected_skills.insert(dm.get_skill_uid("critical_boost"), 3);
    selected_skills.insert(dm.get_skill_uid("critical_element"), 1);
    selected_skills.insert(dm.get_skill_uid("critical_eye"), 3);

    selected_skills.insert(dm.get_skill_uid("spiribirds_call"), 5);

    let (_log, _results) = calculate_skillset(
        weapon_slots,
        selected_skills,
        free_slots,
        crate::data::armor::SexType::Female,
        &dm,
        &cm,
    );
}

#[cfg(test)]
#[test]
fn deco_comb_compare1() {
    use crate::calc::deco_combination::DecorationCombination;
    use crate::calc::types::SlotsVec;

    assert!(DecorationCombination::is_possible_static_lp(
        &SlotsVec::from_vec(vec![5, 2, 2, 0]),
        &SlotsVec::from_vec(vec![5, 2, 0, 0])
    ));
}

#[cfg(test)]
#[test]
fn armor_stat_compare() {
    use crate::data::armor::ArmorStat;

    let mut stat1 = ArmorStat::new_empty();
    stat1.defense = 5;
    stat1.fire_res = -1;

    let mut stat2 = ArmorStat::new_empty();
    stat2.defense = 3;
    stat2.fire_res = 1;

    let mut stat3 = ArmorStat::new_empty();
    stat3.defense = 5;
    stat3.fire_res = -2;

    let mut stat4 = ArmorStat::new_empty();
    stat4.defense = 5;
    stat4.fire_res = -1;

    assert!(stat1 != stat2);
    assert!(stat1 != stat2);
    assert!(stat1 > stat2);
    assert!(stat2 > stat1);
    assert!(stat1 >= stat2);
    assert!(stat2 >= stat1);

    assert!(stat1 != stat3);
    assert!(stat1 != stat3);
    assert!(stat3 <= stat1);
    assert!(stat1 > stat3);
    assert!(stat3 >= stat1);
    assert!(stat1 >= stat3);

    assert!(stat1 == stat4);
    assert!(stat1 == stat4);
    assert!(stat4 <= stat1);
    assert!(stat1 <= stat4);
    assert!(stat4 >= stat1);
    assert!(stat1 >= stat4);
}

#[cfg(test)]
#[tokio::test]

async fn le_equips_compare1() {
    use std::sync::Arc;

    use crate::{
        calc::{
            calc_data_manager::CalcDataManager, calc_equipment::CalcEquipment,
            calc_ui_generator::CalcEquipmentUid,
        },
        data::data_manager::DataManager,
    };

    let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf();

    let dm = DataManager::from_sources(
        &manifest_dir.join("../src-tauri/data/armor.json"),
        &manifest_dir.join("../src-tauri/data/skill.json"),
        &manifest_dir.join("../src-tauri/data/deco.json"),
    )
    .await;

    let mut slot_armors = Vec::new();
    let mut equip_uid = CalcEquipmentUid::default();

    for part_armors in dm.get_slot_only_armors() {
        for armor in part_armors.values() {
            slot_armors.push(Arc::new(CalcEquipment::new_original_armor(
                equip_uid.next_uid(),
                armor.clone(),
                &dm,
            )));
        }
    }

    CalcDataManager::remove_le_equipments(slot_armors.iter().collect(), None);
}

#[cfg(test)]
#[test]
fn skill_count_test() {
    use crate::calc::constant::SKILLS_COUNT;

    assert_eq!(SKILLS_COUNT, 136); // TODO parse from json file
}

#[cfg(test)]
#[tokio::test]
async fn skill_equality_test() {
    use std::collections::HashMap;

    use crate::{calc::skills::SkillsContainer, data::data_manager::DataManager};

    let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf();

    let dm = DataManager::from_sources(
        &manifest_dir.join("../src-tauri/data/armor.json"),
        &manifest_dir.join("../src-tauri/data/skill.json"),
        &manifest_dir.join("../src-tauri/data/deco.json"),
    )
    .await;

    let mut skills1 = SkillsContainer::new();
    skills1.set(dm.get_skill_uid("spread_up"), 3);
    skills1.set(dm.get_skill_uid("critical_element"), 1);
    skills1.set(dm.get_skill_uid("stamina_surge"), 3);
    skills1.set(dm.get_skill_uid("water_attack"), 5);

    let mut skills2 = SkillsContainer::new();
    skills2.set(dm.get_skill_uid("water_attack"), 5);
    skills2.set(dm.get_skill_uid("stamina_surge"), 3);
    skills2.set(dm.get_skill_uid("critical_element"), 1);
    skills2.set(dm.get_skill_uid("spread_up"), 3);

    let mut map = HashMap::new();
    map.insert(skills1.get_list(), 0);

    assert_eq!(skills1, skills2);
    assert!(map.contains_key(&skills2.get_list()));
    assert_ne!(map.get(&skills2.get_list()), None);
}
