#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use data::armor::SexType;
use log::{debug, info};
use mhr_calculator::{
    calc::{
        calc_data_manager::CalcDataManager, calc_result::CalculateResult, calculator::Calculator,
        types::SkillSlotCount,
    },
    data::{
        armor::{AnomalyArmor, ArmorPart, ArmorStat, BaseArmor, SkillIdLevel, Talisman},
        data_manager::DataManager,
        parser::{parse_anomaly, parse_talisman},
        skill::Skill,
    },
    *,
};
use nohash_hasher::IntMap;
use serde::Serialize;
use std::{
    collections::{HashMap, HashSet},
    sync::RwLock,
};

#[tauri::command]
fn cmd_get_file_anomalies(dm: tauri::State<RwLock<DataManager>>) -> Vec<AnomalyArmor> {
    let dm = dm.read().unwrap();

    let anomalies = dm.get_file_anomaly_armors();

    let mut ret = Vec::with_capacity(anomalies.len());

    for armor in anomalies.values() {
        let clone = armor.as_ref().clone();

        ret.push(clone);
    }

    ret
}

#[tauri::command]
fn cmd_clear_file_anomalies(
    dm: tauri::State<RwLock<DataManager>>,
    cm: tauri::State<RwLock<CalcDataManager>>,
) {
    let mut dm = dm.write().unwrap();
    let mut cm = cm.write().unwrap();

    dm.clear_file_anomalies();
    cm.load_anomalies(&dm);
}

#[tauri::command]
fn cmd_parse_anomaly(
    filename: &str,
    dm: tauri::State<RwLock<DataManager>>,
    cm: tauri::State<RwLock<CalcDataManager>>,
) -> Vec<AnomalyArmor> {
    let anomalies = {
        let dm = dm.read().unwrap();

        parse_anomaly(
            filename.as_ref(),
            dm.get_armors(),
            dm.get_armor_name_dict(),
            dm.get_skill_name_dict(),
        )
    };

    let mut dm = dm.write().unwrap();
    let mut cm = cm.write().unwrap();

    dm.set_file_anomalies(anomalies.clone());
    cm.load_anomalies(&dm);

    anomalies
}

#[tauri::command]
fn cmd_add_manual_anomaly(
    original_id: &str,
    skill_diffs: Vec<SkillIdLevel>,
    slot_diffs: Vec<SkillSlotCount>,
    stat_diff: ArmorStat,
    dm: tauri::State<RwLock<DataManager>>,
    cm: tauri::State<RwLock<CalcDataManager>>,
) -> Option<AnomalyArmor> {
    let mut dm = dm.write().unwrap();
    let mut cm = cm.write().unwrap();

    debug!(
        "{} {:?} {:?} {:?}",
        original_id, skill_diffs, slot_diffs, stat_diff
    );

    if original_id.is_empty() {
        return None;
    }

    let anomaly = dm.create_anomaly(original_id, &skill_diffs, &slot_diffs, &stat_diff);

    let inserted = dm.add_manual_anomaly(anomaly);
    cm.load_anomalies(&dm);

    Some((*inserted).clone())
}

#[tauri::command]
fn cmd_clear_manual_anomalies(
    dm: tauri::State<RwLock<DataManager>>,
    cm: tauri::State<RwLock<CalcDataManager>>,
) -> bool {
    let mut dm = dm.write().unwrap();
    let mut cm = cm.write().unwrap();

    dm.clear_manual_anomalies();
    cm.load_anomalies(&dm);

    true
}

#[tauri::command]
fn cmd_set_manual_anomalies(
    anomalies: HashMap<ArmorPart, Vec<AnomalyArmor>>,
    dm: tauri::State<RwLock<DataManager>>,
    cm: tauri::State<RwLock<CalcDataManager>>,
) -> bool {
    let mut dm = dm.write().unwrap();
    let mut cm = cm.write().unwrap();

    dm.clear_manual_anomalies();

    for (_, part_anomalies) in anomalies {
        for anomaly in part_anomalies {
            dm.add_manual_anomaly(anomaly);
        }
    }

    cm.load_anomalies(&dm);

    true
}

#[tauri::command]
fn cmd_get_file_talismans(dm: tauri::State<RwLock<DataManager>>) -> Vec<Talisman> {
    let dm = dm.write().unwrap();

    let talismans = dm.get_file_talismans();

    let mut ret = Vec::with_capacity(talismans.len());

    for tali in talismans {
        let ret_tali = tali.as_ref().clone();
        ret.push(ret_tali);
    }

    ret
}

#[tauri::command]
fn cmd_parse_talisman(
    filename: &str,
    dm: tauri::State<RwLock<DataManager>>,
    cm: tauri::State<RwLock<CalcDataManager>>,
) -> Vec<Talisman> {
    let talismans = {
        let dm = dm.read().unwrap();
        parse_talisman(filename.as_ref(), dm.get_skill_name_dict())
    };

    let mut dm = dm.write().unwrap();
    let mut cm = cm.write().unwrap();

    dm.set_file_talismans(talismans.clone());
    cm.load_talismans(&dm);

    talismans
}

#[tauri::command]
fn cmd_clear_file_talismans(
    dm: tauri::State<RwLock<DataManager>>,
    cm: tauri::State<RwLock<CalcDataManager>>,
) {
    let mut dm = dm.write().unwrap();
    let mut cm = cm.write().unwrap();

    dm.clear_file_talismans();
    cm.load_talismans(&dm);
}

#[tauri::command]
fn cmd_add_manual_talisman(
    talisman: Talisman,
    dm: tauri::State<RwLock<DataManager>>,
    cm: tauri::State<RwLock<CalcDataManager>>,
) -> Talisman {
    let mut dm = dm.write().unwrap();
    let mut cm = cm.write().unwrap();

    let inserted = dm.add_manual_talisman(&talisman);
    cm.load_talismans(&dm);

    (*inserted).clone()
}

#[tauri::command]
fn cmd_delete_manual_talisman(
    id: String,
    dm: tauri::State<RwLock<DataManager>>,
    cm: tauri::State<RwLock<CalcDataManager>>,
) -> bool {
    let mut dm = dm.write().unwrap();
    let mut cm = cm.write().unwrap();

    let removed = dm.remove_manual_talisman(&id);
    cm.load_talismans(&dm);

    removed
}

#[tauri::command]
fn cmd_set_manual_talismans(
    talismans: Vec<Talisman>,
    dm: tauri::State<RwLock<DataManager>>,
    cm: tauri::State<RwLock<CalcDataManager>>,
) -> bool {
    let mut dm = dm.write().unwrap();
    let mut cm = cm.write().unwrap();

    dm.clear_manual_talismans();

    for tali in talismans {
        dm.add_manual_talisman(&tali);
    }

    debug!(
        "Manual talismans count: {}",
        dm.get_manual_talismans().len()
    );

    cm.load_talismans(&dm);

    true
}

#[tauri::command]
fn cmd_clear_manual_talismans(
    dm: tauri::State<RwLock<DataManager>>,
    cm: tauri::State<RwLock<CalcDataManager>>,
) -> bool {
    let mut dm = dm.write().unwrap();
    let mut cm = cm.write().unwrap();

    dm.clear_manual_talismans();
    cm.load_talismans(&dm);

    true
}

#[tauri::command]
fn cmd_get_skill_names(dm: tauri::State<RwLock<DataManager>>) -> HashMap<String, Skill> {
    let dm = &dm.read().unwrap();

    return dm
        .get_skills()
        .iter()
        .map(|skill| (skill.id.clone(), skill.clone()))
        .collect();
}

#[tauri::command]
fn cmd_get_armor_names(dm: tauri::State<RwLock<DataManager>>) -> HashMap<String, BaseArmor> {
    return dm
        .read()
        .unwrap()
        .get_armors()
        .iter()
        .map(|(id, armor)| (id.clone(), armor.as_ref().clone()))
        .collect();
}

#[tauri::command]
fn cmd_set_banned_decos(
    deco_ids: HashMap<String, bool>,
    cm: tauri::State<RwLock<CalcDataManager>>,
) -> bool {
    let mut cm = cm.write().unwrap();

    cm.set_banned_decos(
        deco_ids
            .iter()
            .map(|(deco_id, _)| deco_id.clone())
            .collect::<HashSet<_>>(),
    );

    true
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CalculateSkillsetReturn {
    log: String,
    result: CalculateResult,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CalculateAdditionalSkillsReturn {
    log: String,
    equipments_count: usize,
    calc_time: f32,
    skills: HashMap<String, (SkillSlotCount, SkillSlotCount)>,
    slots: Vec<SkillSlotCount>,
}

#[tauri::command]
async fn cmd_calculate_skillset(
    anomaly_filename: String,
    talisman_filename: String,
    sex_type: SexType,
    weapon_slots: Vec<SkillSlotCount>,
    selected_skills: HashMap<String, SkillSlotCount>,
    free_slots: Vec<SkillSlotCount>,
    include_lte_equips: bool,
    dm: tauri::State<'_, RwLock<DataManager>>,
    cm: tauri::State<'_, RwLock<CalcDataManager>>,
) -> Result<CalculateSkillsetReturn, ()> {
    info!("Start calculating...");

    let selected_skills_uid;

    {
        selected_skills_uid = selected_skills
            .iter()
            .map(|(id, level)| (dm.read().unwrap().get_skill_uid(id), *level))
            .collect::<IntMap<_, _>>();
    }

    {
        let mut dm = dm.write().unwrap();
        let mut cm = cm.write().unwrap();

        let anomalies = parse_anomaly(
            anomaly_filename.as_ref(),
            dm.get_armors(),
            dm.get_armor_name_dict(),
            dm.get_skill_name_dict(),
        );

        let talismans = parse_talisman(talisman_filename.as_ref(), dm.get_skill_name_dict());

        dm.set_file_anomalies(anomalies);
        dm.set_file_talismans(talismans);

        cm.load_anomalies(&dm);
        cm.load_talismans(&dm);

        let banned_decos = cm.get_banned_decos();
        dm.set_banned_decos(banned_decos);
    }

    let log;
    let result;

    {
        let dm = dm.read().unwrap();
        let mut cm = cm.write().unwrap();

        let req_skills = Calculator::convert_to_skills_container(&selected_skills_uid);

        cm.refresh_infos(&dm, &req_skills);

        (log, result) = Calculator::calculate(
            weapon_slots,
            selected_skills_uid,
            free_slots,
            sex_type,
            include_lte_equips,
            &dm,
            &cm,
        );
    }

    Ok(CalculateSkillsetReturn { log, result })
}

#[tauri::command]
async fn cmd_calculate_additional_skills(
    anomaly_filename: String,
    talisman_filename: String,
    sex_type: SexType,
    weapon_slots: Vec<SkillSlotCount>,
    selected_skills: HashMap<String, SkillSlotCount>,
    free_slots: Vec<SkillSlotCount>,
    include_lte_equips: bool,
    dm: tauri::State<'_, RwLock<DataManager>>,
    cm: tauri::State<'_, RwLock<CalcDataManager>>,
) -> Result<CalculateAdditionalSkillsReturn, ()> {
    info!("Start calculating...");

    let selected_skills_uid;

    {
        selected_skills_uid = selected_skills
            .iter()
            .map(|(id, level)| (dm.read().unwrap().get_skill_uid(id), *level))
            .collect::<IntMap<_, _>>();
    }

    {
        let mut dm = dm.write().unwrap();
        let mut cm = cm.write().unwrap();

        let anomalies = parse_anomaly(
            anomaly_filename.as_ref(),
            dm.get_armors(),
            dm.get_armor_name_dict(),
            dm.get_skill_name_dict(),
        );

        let talismans = parse_talisman(talisman_filename.as_ref(), dm.get_skill_name_dict());

        dm.set_file_anomalies(anomalies);
        dm.set_file_talismans(talismans);

        cm.load_anomalies(&dm);
        cm.load_talismans(&dm);
    }

    let log;
    let calc_time;
    let equipments_count;
    let skills;
    let slots;

    {
        let dm = dm.read().unwrap();
        let mut cm = cm.write().unwrap();

        let req_skills = Calculator::convert_to_skills_container(&selected_skills_uid);

        cm.refresh_infos(&dm, &req_skills);

        (log, equipments_count, calc_time, skills, slots) = Calculator::calculate_additional_skills(
            weapon_slots,
            selected_skills_uid,
            free_slots,
            sex_type,
            include_lte_equips,
            &dm,
            &cm,
        );
    }

    Ok(CalculateAdditionalSkillsReturn {
        log,
        equipments_count,
        calc_time,
        skills,
        slots,
    })
}

#[tokio::main]
async fn main() {
    env_logger::init();

    let _num_thread = (num_cpus::get() / 2).max(1);
    let num_thread = 1;
    info!("Thread count: {}", num_thread);

    rayon::ThreadPoolBuilder::new()
        .num_threads(num_thread)
        .build_global()
        .unwrap();

    let dm = DataManager::from_sources(
        "./data/armor.json".as_ref(),
        "./data/skill.json".as_ref(),
        "./data/deco.json".as_ref(),
    )
    .await;

    dm.debug_deco_combs();

    let mut cm = CalcDataManager::new(&dm);
    cm.load_base_armors(&dm);

    tauri::Builder::default()
        .manage(RwLock::new(dm))
        .manage(RwLock::new(cm))
        .invoke_handler(tauri::generate_handler![
            cmd_get_file_anomalies,
            cmd_parse_anomaly,
            cmd_clear_file_anomalies,
            cmd_add_manual_anomaly,
            cmd_clear_manual_anomalies,
            cmd_set_manual_anomalies,
            cmd_get_file_talismans,
            cmd_parse_talisman,
            cmd_add_manual_talisman,
            cmd_clear_file_talismans,
            cmd_delete_manual_talisman,
            cmd_set_manual_talismans,
            cmd_clear_manual_talismans,
            cmd_get_skill_names,
            cmd_get_armor_names,
            cmd_set_banned_decos,
            cmd_calculate_skillset,
            cmd_calculate_additional_skills
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
