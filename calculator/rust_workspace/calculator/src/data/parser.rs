use csv::StringRecord;
use log::info;
use std::fs::File;
use std::io::BufReader;
use std::sync::Arc;
use std::{collections::HashMap, path::Path};

use crate::data::armor::{AnomalyArmor, ArmorSkill, ArmorStat, BaseArmor, SkillIdLevel, Talisman};

fn to_i32(record: &StringRecord, index: usize) -> i32 {
    record[index].parse().unwrap()
}

fn to_i16(record: &StringRecord, index: usize) -> i16 {
    record[index].parse().unwrap()
}
fn to_i8(record: &StringRecord, index: usize) -> i8 {
    record[index].parse().unwrap()
}

pub fn parse_anomaly(
    filename: &Path,
    armors: &HashMap<String, Arc<BaseArmor>>,
    armor_name_dict: &HashMap<String, String>,
    skill_name_dict: &HashMap<String, String>,
) -> Vec<AnomalyArmor> {
    let file = File::open(filename);

    match file {
        Ok(file) => {
            let reader = BufReader::new(file);

            let mut csv_reader = csv::ReaderBuilder::new()
                .has_headers(false)
                .from_reader(reader);

            let mut records = Vec::new();

            for result in csv_reader.records() {
                let record = result.unwrap();

                records.push(record);
            }

            let mut anomaly_armors = Vec::new();

            for record in records {
                let armor_name = &record[0];

                let defense = to_i16(&record, 1);

                let fire_res = to_i16(&record, 2);
                let water_res = to_i16(&record, 3);
                let elec_res = to_i16(&record, 4);
                let ice_res = to_i16(&record, 5);
                let dragon_res = to_i16(&record, 6);

                let slot_size1 = to_i8(&record, 7);
                let slot_size2 = to_i8(&record, 8);
                let slot_size3 = to_i8(&record, 9);

                let slot_sizes = vec![slot_size1, slot_size2, slot_size3];

                let stat = ArmorStat {
                    defense,
                    fire_res,
                    water_res,
                    elec_res,
                    ice_res,
                    dragon_res,
                };

                let mut anomaly_skills = HashMap::new();

                for i in (10..record.len()).step_by(2) {
                    let skill_name = &record[i];

                    if skill_name.is_empty() {
                        continue;
                    }

                    let skill_level = to_i8(&record, i + 1);

                    let skill_id = skill_name_dict.get(skill_name).unwrap();

                    let anomaly_skill = ArmorSkill { level: skill_level };

                    anomaly_skills.insert(skill_id.to_string(), anomaly_skill);
                }

                let armor_id = armor_name_dict.get(armor_name).unwrap();
                let armor_info = armors.get(armor_id).unwrap();

                let anomaly_armor = AnomalyArmor::new(armor_info, stat, slot_sizes, anomaly_skills);

                anomaly_armors.push(anomaly_armor);
            }

            info!("Anomaly parsed - count : {}", anomaly_armors.len());

            anomaly_armors
        }
        Err(_) => Vec::new(),
    }
}

pub fn parse_talisman(filename: &Path, skill_name_dict: &HashMap<String, String>) -> Vec<Talisman> {
    let file = File::open(filename);

    match file {
        Ok(file) => {
            let reader = BufReader::new(file);

            let mut csv_reader = csv::ReaderBuilder::new()
                .has_headers(false)
                .from_reader(reader);

            let mut records = Vec::new();

            for result in csv_reader.records() {
                let record = result.unwrap();

                records.push(record);
            }

            let mut talismans = Vec::new();

            for (index, record) in records.iter().enumerate() {
                let skill_name1 = &record[0];
                let skill_level1 = to_i8(record, 1);
                let skill_name2 = &record[2];
                let skill_level2 = to_i8(record, 3);

                let slot_size1 = to_i8(record, 4);
                let slot_size2 = to_i8(record, 5);
                let slot_size3 = to_i8(record, 6);

                let slot_sizes = vec![slot_size1, slot_size2, slot_size3];

                let mut talisman_skills = Vec::new();

                if !skill_name1.is_empty() {
                    let skill_id = skill_name_dict.get(skill_name1).unwrap();

                    talisman_skills.push(SkillIdLevel {
                        id: skill_id.to_string(),
                        level: skill_level1,
                    });
                }

                if !skill_name2.is_empty() {
                    let skill_id = skill_name_dict.get(skill_name2).unwrap();

                    talisman_skills.push(SkillIdLevel {
                        id: skill_id.to_string(),
                        level: skill_level2,
                    });
                }

                let talisman = Talisman::new(
                    format!("talisman_file_{}", index),
                    talisman_skills,
                    slot_sizes,
                );

                talismans.push(talisman);
            }

            info!("Talisman parsed - count : {}", talismans.len());

            talismans
        }
        Err(_) => Vec::new(),
    }
}
