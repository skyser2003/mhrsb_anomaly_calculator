pub mod full_equipments;
mod utils;

pub static MAX_ANSWER_LENGTH: usize = 200;

use calc::calc_data_manager::CalcDataManager;

use calc::calc_point::CalcPoint;
use calc::constant::MAX_SLOT_LEVEL;
use calc::skills::SkillsContainer;
use calc::types::{EquipmentsArray, SkillSlotCount};

use log::{debug, info};
use nohash_hasher::IntMap;

use std::cmp::Reverse;
use std::collections::HashMap;

use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

use data::armor::{ArmorPart, ArmorStat, SexType};
use data::data_manager::DataManager;
use itertools::iproduct;

use serde::Serialize;

pub mod calc;
pub mod data;

mod test;

use crate::calc::calc_equipment::CalcEquipment;

use crate::calc::calc_equips_iterator::CalcEquipmentsIterator;
use crate::calc::calc_vector::CalcVector;
use crate::calc::constant::EQUIP_PART_COUNT;
use crate::calc::deco_combination::DecorationCombination;

use crate::calc::types::SlotsVec;
use crate::data::armor::{BaseArmor, EMPTY_ARMOR_PREFIX};
use crate::data::Language;
use crate::full_equipments::FullEquipments;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CalculateResult {
    full_equipments: Vec<ResultFullEquipments>,
    calc_time: f32,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResultFullEquipments {
    pub sex_type: SexType,
    pub weapon_slots: Vec<SkillSlotCount>,
    pub armors: HashMap<String, ResultArmor>,
    pub talisman: ResultTalisman,
    pub deco_combs: Vec<ResultDecorationCombination>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResultArmor {
    pub base_id: String,
    pub is_anomaly: bool,

    pub skills: HashMap<String, SkillSlotCount>,
    pub base_skills: HashMap<String, SkillSlotCount>,
    pub diff_skills: HashMap<String, SkillSlotCount>,

    pub slots: Vec<SkillSlotCount>,
    pub base_slots: Vec<SkillSlotCount>,
    pub diff_slots: Vec<SkillSlotCount>,

    pub stat: ArmorStat,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResultTalisman {
    pub skills: HashMap<String, SkillSlotCount>,
    pub slots: Vec<SkillSlotCount>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResultDecorationCombination {
    pub skill_decos: HashMap<String, Vec<SkillSlotCount>>,
    pub slots_sum: Vec<SkillSlotCount>,
    pub leftover_slots_sum: Vec<SkillSlotCount>,
    pub leftover_skills: HashMap<String, i8>,
}

fn info(ret_log: &mut String, text: &str) {
    info!("{}", text);
    ret_log.push_str(&format!("{}\n", text));
}

pub fn check_static_conditions(
    dm: &DataManager,
    cm: &CalcDataManager,
    weapon_slots_lp: &SlotsVec,
    equipments: &EquipmentsArray,
    req_slots_lp: &SlotsVec,
    req_skills: &SkillsContainer,
    req_uids: &Vec<usize>,
) -> Option<(SkillsContainer, SlotsVec)> {
    let mut avail_slots_lp = FullEquipments::calculate_slots_lp(weapon_slots_lp, equipments);

    if DecorationCombination::is_possible_static_lp_equip_mut(&mut avail_slots_lp, &[req_slots_lp])
        == false
    {
        return None;
    }

    let avail_slots_points = dm.calc_slot_point_slots_lp(&avail_slots_lp);

    let sub_point_result =
        cm.check_equipment_point(dm, equipments, &req_skills, &req_uids, &avail_slots_points);

    if sub_point_result == None {
        return None;
    }

    let mut multi_req_skills = sub_point_result.unwrap();

    for (uid, level) in multi_req_skills.iter_mut() {
        let decos = dm.get_deco_by_skill_id(uid);
        if *level == 1 || decos.len() == 1 {
            for index in 0..decos[0].slot_size {
                avail_slots_lp[index as usize] -= *level;
            }

            *level = 0;
        }
    }

    CalcVector::promote_subtracted_lp_slots(&mut avail_slots_lp);

    if avail_slots_lp.iter().all(|&count| count >= 0) == false {
        return None;
    }

    let avail_slots_points = dm.calc_slot_point_slots_lp(&avail_slots_lp);
    let multi_req_points = dm.calc_req_skill_point_slots_lp(&multi_req_skills);

    if CalcPoint::is_possible_static(&avail_slots_points, &multi_req_points) == false {
        return None;
    }

    Some((multi_req_skills, avail_slots_lp))
}

pub fn convert_to_skills_container(skills: &IntMap<usize, SkillSlotCount>) -> SkillsContainer {
    let mut ret = SkillsContainer::new();

    for (&uid, &level) in skills {
        ret.set(uid, level);
    }

    ret
}

pub fn calculate_skillset(
    ori_weapon_slots: Vec<SkillSlotCount>,
    selected_skills: IntMap<usize, SkillSlotCount>,
    free_slots: Vec<SkillSlotCount>,
    sex_type: SexType,
    include_lte_equips: bool,
    dm: &DataManager,
    cm: &CalcDataManager,
) -> (String, CalculateResult) {
    let start_time = Instant::now();

    let weapon_slots = CalcEquipment::convert_from_base_slots(&ori_weapon_slots);
    let weapon_slots_lp = CalcVector::convert_to_lp_slots(&weapon_slots);
    let weapon_points = dm.calc_slot_point_slots_lp(&weapon_slots_lp);
    let selected_skills = convert_to_skills_container(&selected_skills);
    let free_slots = SlotsVec::from_vec(free_slots);
    let free_slots_lp = CalcVector::convert_to_lp_slots(&free_slots);
    let req_points = dm.calc_req_point_slots_lp(&selected_skills, &free_slots_lp);
    let req_uids = selected_skills.get_indices();

    let mut ret = String::from("\n");

    let (no_deco_skills, yes_deco_skills) = dm.get_leftover_skills(&selected_skills);
    let has_unique_skill = !no_deco_skills.is_empty();

    info!(
        "Skills with yes deco: {:?}",
        dm.skills_to_ids(&yes_deco_skills)
    );
    info!(
        "Skills with no deco: {:?}",
        dm.skills_to_ids(&no_deco_skills)
    );

    info!("Sex type: {:?}", sex_type);

    let mut multi_skills = SkillsContainer::new();

    for (uid, level) in yes_deco_skills.iter() {
        if level == 1 || dm.get_deco_by_skill_id(uid).len() <= 1 {
            continue;
        }

        multi_skills.set(uid, level);
    }

    info!("Multi skills: {:?}", multi_skills.debug(dm));

    let (all_original_equips, all_equips) = cm.get_all_equipments(&sex_type, include_lte_equips);
    let all_slot_equips = cm.get_slot_equipments();
    let all_deco_equips =
        CalcDataManager::get_equips_with_deco_skill(&all_equips, &yes_deco_skills);
    let empty_equips = cm.get_empty_equips();

    let all_deco_slot_equips =
        CalcDataManager::merge_deco_slot_equips(&all_deco_equips, &all_slot_equips);

    let mut all_deco_slot_equips_flat = all_deco_slot_equips
        .iter()
        .flatten()
        .map(|&equip| equip)
        .collect::<Vec<_>>();

    all_deco_slot_equips_flat.sort_by_cached_key(|equip| Reverse(equip.point()));

    let (possible_candidate_vecs, _) =
        CalcDataManager::get_possible_unique_equips(&all_equips, &no_deco_skills, &empty_equips);

    for (part, part_equips) in all_equips.iter().enumerate() {
        info!(
            "All part equips size: {} - {}",
            ArmorPart::from_usize(part).as_str(),
            part_equips.len()
        );
    }

    let all_cancididate_len = possible_candidate_vecs[0].len()
        * possible_candidate_vecs[1].len()
        * possible_candidate_vecs[2].len()
        * possible_candidate_vecs[3].len()
        * possible_candidate_vecs[4].len()
        * possible_candidate_vecs[5].len();

    info(
        &mut ret,
        &format!(
            "Has unique skill: {}, all candidate armors count: {}, calculation: {:?}",
            has_unique_skill,
            all_cancididate_len,
            start_time.elapsed()
        ),
    );

    let answers = RwLock::new(HashMap::with_capacity(MAX_ANSWER_LENGTH));

    let is_answer_full = || MAX_ANSWER_LENGTH <= answers.read().unwrap().len();

    let mut possible_candidate_flat = possible_candidate_vecs
        .iter()
        .flatten()
        .map(|&equip| equip)
        .collect::<Vec<_>>();

    possible_candidate_flat.sort_by_cached_key(|equip| Reverse(equip.point()));

    let candidate_iter = iproduct!(
        &possible_candidate_vecs[0],
        &possible_candidate_vecs[1],
        &possible_candidate_vecs[2],
        &possible_candidate_vecs[3],
        &possible_candidate_vecs[4],
        &possible_candidate_vecs[5]
    );

    info!(
        "Theoritically possible count: {}, equips count: {}",
        possible_candidate_vecs
            .iter()
            .fold(1, |acum, elem| acum * elem.len()),
        possible_candidate_flat.len()
    );

    let mut candidates = Vec::new();

    candidate_iter.for_each(|(&c0, &c1, &c2, &c3, &c4, &c5)| {
        let candidate = [c0, c1, c2, c3, c4, c5];

        if CalcDataManager::is_le_candidate(&candidate, &candidates) {
            return;
        }

        let mut deco_req_skills = selected_skills.clone();

        FullEquipments::subtract_skills(&candidate, &mut deco_req_skills);

        for (uid, _) in no_deco_skills.iter() {
            if deco_req_skills.contains(uid) {
                return;
            }
        }

        candidates.push(candidate);
    });

    info!("Le removed candidates length: {}", candidates.len());

    candidates.into_iter().all(|possible_candidate_vec| {
        if is_answer_full() {
            return false;
        }

        let mut deco_req_skills = selected_skills.clone();

        FullEquipments::subtract_skills(&possible_candidate_vec, &mut deco_req_skills);

        let mut key_equips = Vec::new();
        let mut key_parts = [false; EQUIP_PART_COUNT];

        for &equipment in possible_candidate_vec.iter() {
            if !equipment.id().starts_with(EMPTY_ARMOR_PREFIX) {
                key_equips.push(equipment);
                key_parts[equipment.part()] = true;
            };
        }

        let (mut parts, ge_equips_map) = CalcDataManager::get_possible_general_part_equips(
            &all_deco_slot_equips_flat,
            &deco_req_skills,
            true,
        );

        parts.retain(|equip| key_parts[equip.part()] == false);

        if !has_unique_skill {
            debug!("General count: {}", parts.len());

            for (i, part) in parts.iter().enumerate() {
                debug!(
                    "Part {} ({}), point: {}, skills: {:?}",
                    i,
                    ArmorPart::from_usize(part.part()).as_str(),
                    part.point(),
                    part.skills().debug(dm)
                );
            }
        }

        // Check for static conditions
        let mut parts_iterator = CalcEquipmentsIterator::new(
            parts,
            &key_equips,
            &req_points,
            &weapon_points,
            &empty_equips,
            dm,
            cm,
            &selected_skills,
        );

        parts_iterator.all(|uids| {
            if is_answer_full() {
                return false;
            }

            let equipments = cm.get_full_equipments(&uids);

            let multi_deco_leftovers = check_static_conditions(
                dm,
                cm,
                &weapon_slots_lp,
                &equipments,
                &free_slots_lp,
                &selected_skills,
                &req_uids,
            );

            if multi_deco_leftovers.is_none() {
                return true;
            }

            let (multi_deco_req_skills, avail_slots_lp) = multi_deco_leftovers.unwrap();

            if !dm.check_possible_deco_combs_lp(&multi_deco_req_skills, &avail_slots_lp) {
                return true;
            }

            debug!(
                "Possible candidates: {:?}\nleft skills: {:?}, slots: {:?}",
                equipments.map(|part| part.id()),
                multi_deco_req_skills.debug(dm),
                CalcVector::convert_from_lp_slots(&avail_slots_lp)
            );

            let mut all_le_uids = Vec::new();

            equipments.iter().for_each(|equip| {
                if !has_unique_skill || !key_parts[equip.part()] {
                    if let Some(le_uids) = ge_equips_map.get(&equip.uid()) {
                        for &uid in le_uids {
                            all_le_uids.push(uid);
                        }

                        return;
                    }
                }

                all_le_uids.push(equip.uid());
            });

            let mut all_le_equips = all_le_uids
                .iter()
                .map(|&uid| cm.get_by_uid(uid))
                .collect::<Vec<_>>();
            all_le_equips.sort_by_cached_key(|equip| Reverse(equip.point()));

            debug!("Le equips count: {}", all_le_equips.len());

            let mut le_iterator = CalcEquipmentsIterator::new(
                all_le_equips,
                &key_equips,
                &req_points,
                &weapon_points,
                &empty_equips,
                dm,
                cm,
                &selected_skills,
            );

            le_iterator.all(|le_uids| {
                if is_answer_full() {
                    return false;
                }

                let le_equips = cm.get_full_equipments(&le_uids);

                let multi_deco_leftovers = check_static_conditions(
                    dm,
                    cm,
                    &weapon_slots_lp,
                    &le_equips,
                    &free_slots_lp,
                    &selected_skills,
                    &req_uids,
                );

                if multi_deco_leftovers.is_none() {
                    return true;
                }

                let (multi_deco_req_skills, avail_slots_lp) = multi_deco_leftovers.unwrap();

                if !dm.check_possible_deco_combs_lp(&multi_deco_req_skills, &avail_slots_lp) {
                    return true;
                }

                {
                    let mut answers = answers.write().unwrap();

                    if MAX_ANSWER_LENGTH <= answers.len() {
                        return false;
                    }

                    let local_answers = calculate_full_equip(
                        dm,
                        &all_original_equips,
                        &selected_skills,
                        &free_slots_lp,
                        &yes_deco_skills,
                        &weapon_slots_lp,
                        &le_equips,
                        &sex_type,
                    );

                    for local_answer in local_answers {
                        let local_equips_id = FullEquipments::get_full_equip_id(&local_answer.0);

                        if answers.contains_key(&local_equips_id) {
                            continue;
                        }

                        if MAX_ANSWER_LENGTH <= answers.len() {
                            return false;
                        }

                        answers.insert(local_equips_id, local_answer);
                    }
                }

                true
            })
        })
    });

    let elapsed_sort = start_time.elapsed();
    let answers = answers.read().unwrap();

    info(
        &mut ret,
        &format!(
            "Answers count: {}, all_loop_cases sorting elapsed: {:?}",
            answers.len(),
            elapsed_sort
        ),
    );

    let elapsed_final = start_time.elapsed();

    let mut all_answers_length = 0;

    for (_, (_, deco_combs, _)) in answers.iter() {
        for _ in deco_combs.iter() {
            all_answers_length += 1;
        }
    }

    info(
        &mut ret,
        &format!(
            "calculate_skillset elapsed: {:?},\nanswers length: {} -> {}",
            elapsed_final,
            answers.len(),
            all_answers_length
        ),
    );
    info!("{}", ret);

    let calculate_result = generate_result(
        dm,
        &sex_type,
        &ori_weapon_slots,
        &weapon_slots_lp,
        &free_slots_lp,
        &answers
            .iter()
            .map(|(_, equips)| equips.clone())
            .collect::<Vec<_>>(),
        start_time.elapsed(),
    );

    (ret, calculate_result)
}

pub fn generate_result(
    dm: &DataManager,
    sex_type: &SexType,
    ori_weapon_slots: &Vec<SkillSlotCount>,
    weapon_slots_lp: &SlotsVec,
    req_slots_lp: &SlotsVec,
    answers: &[(
        EquipmentsArray,
        Vec<DecorationCombination>,
        Vec<SkillsContainer>,
    )],
    elapsed: Duration,
) -> CalculateResult {
    let mut full_equipments = answers
        .iter()
        .map(|(equipments, deco_combs, all_leftover_skills)| {
            let result_armors = equipments
                .iter()
                .filter_map(|equip| {
                    if equip.part() == ArmorPart::Talisman.as_usize() {
                        return None;
                    }

                    let armor = equip.as_armor();

                    let skills = equip
                        .skills()
                        .iter()
                        .filter_map(|(uid, level)| {
                            if level == 0 {
                                None
                            } else {
                                Some((dm.get_skill(uid).id.clone(), level))
                            }
                        })
                        .collect::<HashMap<_, _>>();

                    let mut diff_skills = HashMap::new();
                    let base_skills;

                    let mut diff_slots = Vec::new();
                    let base_slots;

                    let is_anomaly = armor.is_anomaly();

                    if is_anomaly {
                        let original_armor = dm.get_armor(armor.base_id());
                        let anomaly_armor = dm.get_anomaly_armor(armor.id()).unwrap();

                        base_skills = original_armor
                            .skills
                            .iter()
                            .map(|(id, info)| (id.clone(), info.level))
                            .collect();

                        diff_skills = anomaly_armor
                            .skill_diffs
                            .iter()
                            .map(|(id, info)| (id.clone(), info.level))
                            .collect();

                        base_slots = CalcEquipment::convert_from_base_slots(&original_armor.slots)
                            .data
                            .0[0]
                            .to_vec();

                        diff_slots = anomaly_armor.slot_diffs.clone();
                    } else {
                        base_skills = skills.clone();
                        base_slots = equip.slots().data.0[0].to_vec();
                    }

                    let result_armor = ResultArmor {
                        base_id: armor.base_id().clone(),
                        is_anomaly,
                        skills,
                        base_skills,
                        diff_skills,
                        slots: equip.slots().data.0[0].to_vec(),
                        base_slots,
                        diff_slots,
                        stat: equip.stats(),
                    };

                    Some((
                        ArmorPart::from_usize(equip.part()).as_str().to_string(),
                        result_armor,
                    ))
                })
                .collect::<HashMap<String, ResultArmor>>();

            let avail_slots_lp =
                FullEquipments::calculate_slots_lp(weapon_slots_lp, equipments) - req_slots_lp;

            let mut result_deco_combs = deco_combs
                .iter()
                .zip(all_leftover_skills)
                .map(|(deco_comb, leftover_skills)| {
                    let mut slots_diff = avail_slots_lp - deco_comb.sum_lp;

                    CalcVector::promote_subtracted_lp_slots_to_slots(&mut slots_diff);

                    let mut ret_leftover_skills = HashMap::new();

                    for (uid, level) in leftover_skills.iter() {
                        let skill_id = dm.get_skill(uid).id.clone();

                        ret_leftover_skills.insert(skill_id, level);
                    }

                    let ret = ResultDecorationCombination {
                        skill_decos: deco_comb
                            .get_skill_decos()
                            .iter()
                            .filter_map(|(&uid, levels)| {
                                if levels.is_empty() {
                                    None
                                } else {
                                    let decos = dm.get_deco_by_skill_id(uid);

                                    let mut deco_counts = Vec::new();

                                    for deco in decos {
                                        let count = levels[deco.slot_size as usize - 1];
                                        deco_counts.push(count);
                                    }

                                    Some((dm.get_skill(uid).id.clone(), deco_counts))
                                }
                            })
                            .collect(),
                        slots_sum: CalcVector::convert_from_lp_slots(&deco_comb.sum_lp).data.0[0]
                            .to_vec(),
                        leftover_slots_sum: slots_diff.data.0[0].to_vec(),
                        leftover_skills: ret_leftover_skills,
                    };

                    ret
                })
                .collect::<Vec<ResultDecorationCombination>>();

            for index in 0..MAX_SLOT_LEVEL {
                result_deco_combs
                    .sort_by_cached_key(|comb| Reverse(comb.leftover_slots_sum[index]));
            }

            result_deco_combs.sort_by_cached_key(|comb| {
                Reverse(
                    comb.leftover_slots_sum
                        .iter()
                        .map(|&val| val as i32)
                        .sum::<i32>(),
                )
            });

            let equips_by_part = FullEquipments::save_by_part(equipments);
            let equip = equips_by_part[&ArmorPart::Talisman.as_usize()];

            let result_tali = ResultTalisman {
                skills: equip
                    .skills()
                    .iter()
                    .filter_map(|(uid, level)| {
                        if level == 0 {
                            None
                        } else {
                            Some((dm.get_skill(uid).id.clone(), level))
                        }
                    })
                    .collect(),
                slots: equip.slots().data.0[0].to_vec(),
            };

            ResultFullEquipments {
                sex_type: sex_type.clone(),
                weapon_slots: ori_weapon_slots.clone(),
                armors: result_armors,
                deco_combs: result_deco_combs,
                talisman: result_tali,
            }
        })
        .collect::<Vec<ResultFullEquipments>>();

    for index in 0..MAX_SLOT_LEVEL {
        full_equipments.sort_by_cached_key(|equips| {
            let count = if equips.deco_combs.len() == 0 {
                0
            } else {
                equips.deco_combs[0].leftover_slots_sum[index]
            };

            Reverse(count)
        });
    }

    full_equipments.sort_by_cached_key(|equips| {
        let leftover_slots_sum = if equips.deco_combs.len() == 0 {
            0
        } else {
            equips.deco_combs[0]
                .leftover_slots_sum
                .iter()
                .map(|&val| val as i32)
                .sum::<i32>()
        };

        Reverse(leftover_slots_sum)
    });

    CalculateResult {
        full_equipments,
        calc_time: elapsed.as_secs_f32(),
    }
}

pub fn calculate_full_equip<'a>(
    dm: &DataManager,
    all_original_equips: &'a [HashMap<String, &Arc<CalcEquipment>>],
    req_skills: &SkillsContainer,
    req_slots_lp: &SlotsVec,
    yes_deco_skills: &SkillsContainer,
    weapon_slots_lp: &SlotsVec,
    equipments: &EquipmentsArray,
    sex_type: &SexType,
) -> Vec<(
    EquipmentsArray<'a>,
    Vec<DecorationCombination>,
    Vec<SkillsContainer>,
)> {
    let avail_slots_lp =
        FullEquipments::calculate_slots_lp(weapon_slots_lp, equipments) - req_slots_lp;
    let mut yes_deco_skills = yes_deco_skills.clone();

    FullEquipments::subtract_skills(equipments, &mut yes_deco_skills);
    yes_deco_skills.clear_zeros();

    let all_possible_deco_combs = dm.get_full_possible_deco_combs(&yes_deco_skills);
    let mut possible_deco_combs = Vec::new();

    for comb in all_possible_deco_combs.iter() {
        if comb.is_possible_lp(&avail_slots_lp) {
            possible_deco_combs.push(comb.clone());
        }
    }

    if !yes_deco_skills.is_empty() && possible_deco_combs.is_empty() {
        return Vec::new();
    }

    let all_skills = FullEquipments::calculate_skills(equipments);

    debug!("Initial slots: {:?}", avail_slots_lp);
    debug!("Skill ids: {:?}", dm.skills_to_ids(&all_skills));

    for local_answer in possible_deco_combs.iter() {
        debug!("Local answer: {:?}", local_answer);
    }

    debug!(
        "Possible slot combinations: {:?} {:?}",
        possible_deco_combs
            .iter()
            .map(|comb| comb
                .combs_per_skill_lp
                .iter()
                .filter_map(|(&uid, &levels)| {
                    if levels.is_empty() {
                        None
                    } else {
                        Some((&dm.get_skill(uid).id, levels))
                    }
                })
                .collect())
            .collect::<Vec<HashMap<&String, SlotsVec>>>(),
        possible_deco_combs
            .iter()
            .map(|comb| CalcVector::convert_from_lp_slots(&comb.sum_lp))
            .collect::<Vec<SlotsVec>>()
    );

    let equips_by_part = FullEquipments::save_by_part(equipments);

    let helm = &equips_by_part[&ArmorPart::Helm.as_usize()];
    let torso = &equips_by_part[&ArmorPart::Torso.as_usize()];
    let arm = &equips_by_part[&ArmorPart::Arm.as_usize()];
    let waist = &equips_by_part[&ArmorPart::Waist.as_usize()];
    let feet = &equips_by_part[&ArmorPart::Feet.as_usize()];
    let tali = &equips_by_part[&ArmorPart::Talisman.as_usize()];

    debug!(
        "Armors ids: ({}), ({}), ({}), ({}), ({}), ({})",
        helm.id(),
        torso.id(),
        arm.id(),
        waist.id(),
        feet.id(),
        tali.id(),
    );

    debug!(
        "Armors names: ({}), ({}), ({}), ({}), ({})",
        helm.as_armor().name(&Language::Korean),
        torso.as_armor().name(&Language::Korean),
        arm.as_armor().name(&Language::Korean),
        waist.as_armor().name(&Language::Korean),
        feet.as_armor().name(&Language::Korean),
    );

    let mut real_armors = Vec::new();
    let mut all_real_armors_case = 1;

    for equipment in equipments {
        let equip_id = equipment.id();
        let part = equipment.part();

        let is_slot_equip = BaseArmor::is_slot_armor(equip_id);
        let is_empty_equip = equip_id.starts_with(EMPTY_ARMOR_PREFIX);

        if is_slot_equip {
            let mut part_real_equips = Vec::new();

            if part == ArmorPart::Talisman.as_usize() {
                let talis_by_slot = dm.get_talismans_by_slot(equip_id);

                for base_tali in talis_by_slot {
                    let box_tali = all_original_equips[part][base_tali.id()];
                    part_real_equips.push(box_tali);
                }
            } else if is_empty_equip {
                part_real_equips.append(
                    &mut all_original_equips[part]
                        .iter()
                        .map(|(_, &equip)| equip)
                        .collect::<Vec<_>>(),
                );
            } else {
                let armors_by_slot = dm.get_armors_by_slot(part, sex_type, equip_id);

                for base_armor in armors_by_slot {
                    let box_armor = all_original_equips[part][base_armor.id()];

                    part_real_equips.push(box_armor);
                }
            }

            all_real_armors_case *= part_real_equips.len();
            real_armors.push(part_real_equips);
        } else {
            real_armors.push(vec![all_original_equips[part][equipment.id()]]);
        }
    }

    let mut answers_equip = Vec::with_capacity(all_real_armors_case);

    for (&a0, &a1, &a2, &a3, &a4, &a5) in iproduct!(
        &real_armors[0],
        &real_armors[1],
        &real_armors[2],
        &real_armors[3],
        &real_armors[4],
        &real_armors[5]
    ) {
        let equipments = [a0, a1, a2, a3, a4, a5];

        answers_equip.push(equipments);
    }

    let mut local_answers = Vec::with_capacity(answers_equip.len());

    for equipments in answers_equip.into_iter() {
        let mut all_leftover_skills = Vec::with_capacity(possible_deco_combs.len());

        for comb in possible_deco_combs.iter() {
            let mut leftover_skills = SkillsContainer::new();

            for equip in &equipments {
                leftover_skills.add(equip.skills());
            }

            let deco_skills = dm.get_skills_from_decos(&comb.get_skill_decos());
            leftover_skills.add(&deco_skills);

            leftover_skills.sub(req_skills);

            all_leftover_skills.push(leftover_skills);
        }

        local_answers.push((equipments, possible_deco_combs.clone(), all_leftover_skills));
    }

    local_answers
}
