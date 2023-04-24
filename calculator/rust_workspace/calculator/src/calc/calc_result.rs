use std::{cmp::Reverse, collections::HashMap, time::Duration};

use serde::Serialize;

use crate::data::{
    armor::{ArmorPart, ArmorStat, SexType},
    data_manager::DataManager,
};

use super::{
    calc_equipment::CalcEquipment,
    calc_vector::CalcVector,
    constant::MAX_SLOT_LEVEL,
    deco_combination::DecorationCombination,
    full_equipments::FullEquipments,
    skills::SkillsContainer,
    types::{EquipmentsArray, SkillSlotCount, SlotsVec},
};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CalculateResult {
    pub full_equipments: Vec<ResultFullEquipments>,
    pub calc_time: f32,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResultFullEquipments {
    pub sex_type: SexType,
    pub total_raw_slots: Vec<SkillSlotCount>,
    pub weapon_slots: Vec<SkillSlotCount>,
    pub armors: HashMap<String, ResultArmor>,
    pub talisman: ResultTalisman,
    pub deco_combs: Vec<ResultDecorationCombination>,
    pub common_leftover_skills: HashMap<String, i8>,
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

pub struct CalcResultGenerator {}

impl CalcResultGenerator {
    pub fn generate(
        dm: &DataManager,
        sex_type: &SexType,
        ori_weapon_slots: &[SkillSlotCount],
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

                            base_slots =
                                CalcEquipment::convert_from_base_slots(&original_armor.slots)
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

                let total_raw_slots_lp =
                    FullEquipments::calculate_slots_lp(weapon_slots_lp, equipments);

                let total_raw_slots = CalcVector::convert_from_lp_slots(&total_raw_slots_lp);

                let avail_slots_lp = total_raw_slots_lp - req_slots_lp;

                let common_leftover_skills =
                    SkillsContainer::get_have_in_common_skills(&all_leftover_skills);

                let mut result_deco_combs = deco_combs
                    .iter()
                    .zip(all_leftover_skills)
                    .map(|(deco_comb, leftover_skills)| {
                        let mut slots_diff = avail_slots_lp - deco_comb.sum_lp;

                        CalcVector::promote_subtracted_lp_slots_to_slots(&mut slots_diff);

                        let mut ret_leftover_skills = HashMap::new();

                        for (uid, level) in leftover_skills.iter() {
                            let common_level = common_leftover_skills.get(uid);

                            if common_level == 0 {
                                continue;
                            }

                            let diff_level = level - common_level;

                            if diff_level == 0 {
                                continue;
                            }

                            let skill_id = dm.get_skill(uid).id.clone();

                            ret_leftover_skills.insert(skill_id, diff_level);
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
                            slots_sum: CalcVector::convert_from_lp_slots(&deco_comb.sum_lp).data.0
                                [0]
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

                let common_leftover_skills = common_leftover_skills
                    .iter()
                    .map(|(uid, level)| (dm.get_skill(uid).id.clone(), level))
                    .collect();

                ResultFullEquipments {
                    sex_type: sex_type.clone(),
                    total_raw_slots: total_raw_slots.data.0[0].to_vec(),
                    weapon_slots: ori_weapon_slots.to_owned(),
                    armors: result_armors,
                    deco_combs: result_deco_combs,
                    common_leftover_skills,
                    talisman: result_tali,
                }
            })
            .collect::<Vec<ResultFullEquipments>>();

        for index in 0..MAX_SLOT_LEVEL {
            full_equipments.sort_by_cached_key(|equips| {
                let count = if equips.deco_combs.is_empty() {
                    0
                } else {
                    equips.deco_combs[0].leftover_slots_sum[index]
                };

                Reverse(count)
            });
        }

        full_equipments.sort_by_cached_key(|equips| {
            let leftover_slots_sum = if equips.deco_combs.is_empty() {
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
}
