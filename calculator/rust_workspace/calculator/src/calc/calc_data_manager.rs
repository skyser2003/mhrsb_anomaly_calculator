use std::{
    cmp::Reverse,
    collections::{HashMap, HashSet},
    sync::Arc,
};

use log::{debug, info};
use nohash_hasher::IntMap;

use crate::data::{
    armor::{ArmorPart, SexType, EMPTY_ARMOR_PREFIX, SLOT_ARMOR_PREFIX},
    data_manager::DataManager,
};

use super::{
    calc_equipment::CalcEquipment,
    calc_point::CalcPoint,
    calc_ui_generator::CalcEquipmentUid,
    constant::{EQUIP_PART_COUNT, MAX_SLOT_LEVEL},
    full_equipments::FullEquipments,
    skills::SkillsContainer,
    types::{EquipmentsArray, PointsVec},
};

type AllRealEquipments<'a> = Vec<HashMap<String, &'a Arc<CalcEquipment>>>;

pub struct CalcDataManager {
    uid_gen: CalcEquipmentUid,

    all_base_armors: Vec<HashMap<String, Arc<CalcEquipment>>>,
    all_anomaly_armors: Vec<HashMap<String, Arc<CalcEquipment>>>,
    all_talismans: HashMap<String, Arc<CalcEquipment>>,

    all_equips: Vec<Arc<CalcEquipment>>,

    slot_armors: Vec<HashMap<String, Arc<CalcEquipment>>>,
    slot_talismans: HashMap<String, Arc<CalcEquipment>>,

    empty_equips: [Arc<CalcEquipment>; EQUIP_PART_COUNT],
}

impl CalcDataManager {
    pub fn new(dm: &DataManager) -> Self {
        let mut uid_gen = CalcEquipmentUid::default();

        let armor_parts_len = ArmorPart::get_all_armor().len();

        let mut all_base_armors = Vec::with_capacity(armor_parts_len);
        let mut all_anomaly_armors = Vec::with_capacity(armor_parts_len);

        let mut slot_armors = Vec::with_capacity(armor_parts_len);

        for _ in 0..armor_parts_len {
            all_base_armors.push(HashMap::new());
            all_anomaly_armors.push(HashMap::new());

            slot_armors.push(HashMap::new());
        }

        let mut empty_equips_opt: [Option<Arc<CalcEquipment>>; EQUIP_PART_COUNT] =
            Default::default();

        for base_armor in dm.empty_armors() {
            let calc_armor =
                CalcEquipment::new_original_armor(uid_gen.next_uid(), base_armor.clone(), dm);

            empty_equips_opt[base_armor.part.as_usize()] = Some(Arc::new(calc_armor));
        }

        let empty_tali = CalcEquipment::new_talisman(
            uid_gen.next_uid(),
            Arc::new(dm.empty_talisman().clone()),
            dm,
        );

        empty_equips_opt[ArmorPart::Talisman.as_usize()] = Some(Arc::new(empty_tali));

        let empty_equips = empty_equips_opt.map(|opt| opt.unwrap());

        Self {
            uid_gen,

            all_base_armors,
            all_anomaly_armors,
            all_talismans: HashMap::new(),

            all_equips: Vec::new(),

            slot_armors,
            slot_talismans: HashMap::new(),

            empty_equips,
        }
    }

    pub fn load_base_armors(&mut self, dm: &DataManager) {
        self.uid_gen.flush();

        for parts in self.all_base_armors.iter_mut() {
            parts.clear();
        }

        for part in ArmorPart::get_all_armor() {
            let base_armors = dm.get_parts(part.as_usize());

            for &base_armor in &base_armors {
                let anomaly_base = dm.get_anomaly_armor(base_armor.id());

                if anomaly_base.is_some() {
                    continue;
                }

                let calc_armor = Arc::new(CalcEquipment::new_original_armor(
                    self.uid_gen.next_uid(),
                    base_armor.clone(),
                    dm,
                ));

                self.all_base_armors[part.as_usize()].insert(calc_armor.id().clone(), calc_armor);
            }
        }

        self.load_slot_armors(dm);
    }

    pub fn load_anomalies(&mut self, dm: &DataManager) {
        for parts in self.all_anomaly_armors.iter_mut() {
            parts.clear();
        }

        let anomaly_armors = dm.get_anomaly_armors();

        for (id, anomaly_armor) in anomaly_armors.iter() {
            let calc_armor = Arc::new(CalcEquipment::new_anomaly_armor(
                self.uid_gen.next_uid(),
                anomaly_armor.clone(),
                dm,
            ));

            let part = calc_armor.part();

            self.all_anomaly_armors[part].insert(id.clone(), calc_armor);
        }

        self.load_slot_armors(dm);
    }

    pub fn load_talismans(&mut self, dm: &DataManager) {
        self.all_talismans.clear();

        for tali in dm.get_talismans() {
            let calc_tali = CalcEquipment::new_talisman(self.uid_gen.next_uid(), tali.clone(), dm);

            self.all_talismans
                .insert(calc_tali.id().clone(), Arc::new(calc_tali));
        }

        self.load_slot_talismans(dm);
    }

    pub fn load_slot_armors(&mut self, dm: &DataManager) {
        for parts in self.slot_armors.iter_mut() {
            parts.clear();
        }

        let all_slot_only_armors = dm.get_slot_only_armors();

        for (part, parts) in self.slot_armors.iter_mut().enumerate() {
            let slot_only_armors = &all_slot_only_armors[part];

            for armor in slot_only_armors.values() {
                let calc_armor =
                    CalcEquipment::new_original_armor(self.uid_gen.next_uid(), armor.clone(), dm);

                parts.insert(calc_armor.id().clone(), Arc::new(calc_armor));
            }
        }
    }

    pub fn load_slot_talismans(&mut self, dm: &DataManager) {
        self.slot_talismans.clear();

        let slot_only_talis = dm.get_slot_only_talismans();

        for tali in slot_only_talis.values() {
            let calc_tali = CalcEquipment::new_talisman(self.uid_gen.next_uid(), tali.clone(), dm);

            self.slot_talismans
                .insert(calc_tali.id().clone(), Arc::new(calc_tali));
        }
    }

    pub fn get_all_equipments(
        &self,
        sex_type: &SexType,
        include_lte_equips: bool,
    ) -> (AllRealEquipments, Vec<Vec<&Arc<CalcEquipment>>>) {
        let equip_parts_len = ArmorPart::get_all_equip().len();

        let mut all_real_equips = Vec::with_capacity(equip_parts_len);
        let mut all_calc_equips = Vec::with_capacity(equip_parts_len);

        for part in 0..ArmorPart::get_all_armor().len() {
            let max_capacity =
                self.all_base_armors[part].len() + self.all_anomaly_armors[part].len();

            all_real_equips.push(HashMap::with_capacity(max_capacity));
            all_calc_equips.push(Vec::with_capacity(max_capacity));
        }

        for (part, part_equips) in self.all_base_armors.iter().enumerate() {
            let real_part_equips = all_real_equips.get_mut(part).unwrap();
            let calc_part_equips = all_calc_equips.get_mut(part).unwrap();

            for equip in part_equips.values() {
                if !DataManager::is_valid_sextype(equip.as_armor().sex_type(), sex_type) {
                    continue;
                }

                real_part_equips.insert(equip.id().clone(), equip);
                calc_part_equips.push(equip);
            }
        }

        for (part, part_equips) in self.all_anomaly_armors.iter().enumerate() {
            let real_part_equips = all_real_equips.get_mut(part).unwrap();
            let calc_part_equips = all_calc_equips.get_mut(part).unwrap();

            for equip in part_equips.values() {
                if !DataManager::is_valid_sextype(equip.as_armor().sex_type(), sex_type) {
                    continue;
                }

                real_part_equips.insert(equip.id().clone(), equip);
                calc_part_equips.push(equip);
            }
        }

        all_real_equips.push(
            self.all_talismans
                .iter()
                .map(|(id, equip)| (id.clone(), equip))
                .collect(),
        );

        for equip in self.get_empty_equips() {
            all_real_equips[equip.part()].insert(equip.id().clone(), equip);
        }

        let talis_vec = self
            .all_talismans
            .iter()
            .map(|(_, tali)| tali)
            .collect::<Vec<_>>();

        all_calc_equips.push(talis_vec);

        for (part, part_equips) in all_calc_equips.iter_mut().enumerate() {
            if part == ArmorPart::Talisman.as_usize() || !include_lte_equips {
                *part_equips = Self::remove_le_equipments(part_equips.clone(), None);
            }

            Self::remove_duplicate_equipments(part_equips);

            CalcEquipment::sort_by_points(part_equips);
        }

        (all_real_equips, all_calc_equips)
    }

    pub fn get_slot_equipments(&self) -> Vec<Vec<&Arc<CalcEquipment>>> {
        let mut ret = Vec::with_capacity(ArmorPart::get_all_equip().len());

        for _ in 0..ret.capacity() {
            ret.push(Vec::new());
        }

        for equips in &self.slot_armors {
            for equip in equips.values() {
                ret[equip.part()].push(equip);
            }
        }

        for tali in self.slot_talismans.values() {
            ret[ArmorPart::Talisman.as_usize()].push(tali);
        }

        ret
    }

    pub fn get_by_uid(&self, uid: usize) -> &Arc<CalcEquipment> {
        &self.all_equips[uid]
    }

    pub fn get_full_equipments(&self, uids: &[usize; EQUIP_PART_COUNT]) -> EquipmentsArray {
        uids.map(|uid| self.get_by_uid(uid))
    }

    pub fn get_full_equipments_clone(
        &self,
        uids: &[usize; EQUIP_PART_COUNT],
    ) -> [CalcEquipment; EQUIP_PART_COUNT] {
        uids.map(|uid| {
            let equip = self.get_by_uid(uid).clone();
            (*equip).clone()
        })
    }

    pub fn get_empty_equips(&self) -> EquipmentsArray {
        let mut ret_opt: [Option<&Arc<CalcEquipment>>; EQUIP_PART_COUNT] = Default::default();

        for equip in &self.empty_equips {
            ret_opt[equip.part()] = Some(equip);
        }

        ret_opt.map(|opt| opt.unwrap())
    }

    pub fn refresh_infos(&mut self, dm: &DataManager, req_skills: &SkillsContainer) {
        self.uid_gen.flush();
        self.all_equips.clear();

        // Armors
        for equips in self.all_base_armors.iter_mut() {
            for equip in equips.values_mut() {
                Self::update_equipment(
                    equip,
                    &mut self.uid_gen,
                    &mut self.all_equips,
                    dm,
                    req_skills,
                );
            }
        }

        for equips in self.all_anomaly_armors.iter_mut() {
            for equip in equips.values_mut() {
                Self::update_equipment(
                    equip,
                    &mut self.uid_gen,
                    &mut self.all_equips,
                    dm,
                    req_skills,
                );
            }
        }

        for equips in self.slot_armors.iter_mut() {
            for equip in equips.values_mut() {
                Self::update_equipment(
                    equip,
                    &mut self.uid_gen,
                    &mut self.all_equips,
                    dm,
                    req_skills,
                );
            }
        }

        // Talismans
        for equip in self.all_talismans.values_mut() {
            Self::update_equipment(
                equip,
                &mut self.uid_gen,
                &mut self.all_equips,
                dm,
                req_skills,
            );
        }

        for equip in self.slot_talismans.values_mut() {
            Self::update_equipment(
                equip,
                &mut self.uid_gen,
                &mut self.all_equips,
                dm,
                req_skills,
            );
        }

        // Equipments
        for equip in self.empty_equips.iter_mut() {
            Self::update_equipment(
                equip,
                &mut self.uid_gen,
                &mut self.all_equips,
                dm,
                req_skills,
            );
        }
    }

    pub fn check_equipment_point(
        &self,
        dm: &DataManager,
        equipments: &EquipmentsArray,
        req_skills: &SkillsContainer,
        req_uids: &Vec<usize>,
        sub_slots_points: &PointsVec,
    ) -> Option<SkillsContainer> {
        let mut sub_req_skills = req_skills.clone();

        for equip in equipments.iter() {
            sub_req_skills.sub(equip.skills());
        }

        let new_req_points = dm.calc_req_skill_point_slots_lp_by_uids(&sub_req_skills, req_uids);

        if !CalcPoint::is_possible_static(sub_slots_points, &new_req_points) {
            return None;
        }

        Some(sub_req_skills)
    }

    fn update_equipment(
        equip: &mut Arc<CalcEquipment>,
        uid_gen: &mut CalcEquipmentUid,
        all_equips: &mut Vec<Arc<CalcEquipment>>,
        dm: &DataManager,
        req_skills: &SkillsContainer,
    ) {
        let equip_inner = Arc::get_mut(equip).unwrap();
        equip_inner.set_uid(uid_gen.next_uid());
        equip_inner.set_point(dm, req_skills);

        all_equips.push(equip.clone());
    }

    pub fn remove_le_equipments<'a>(
        equipments: Vec<&'a Arc<CalcEquipment>>,
        req_skills: Option<&SkillsContainer>,
    ) -> Vec<&'a Arc<CalcEquipment>> {
        let mut left_equipments = Vec::with_capacity(equipments.len());

        for &equip1 in equipments.iter() {
            let mut is_le = false;

            for &equip2 in equipments.iter() {
                if equip1.part() != equip2.part() || equip1.uid() == equip2.uid() {
                    continue;
                }

                if equip1.skills() == equip2.skills()
                    && equip1.slots() == equip2.slots()
                    && equip1.stats() == equip2.stats()
                {
                    continue;
                }

                if equip1.is_le(equip2, false, req_skills) {
                    is_le = true;
                    break;
                }
            }

            if !is_le {
                left_equipments.push(equip1);
            }
        }

        left_equipments
    }

    pub fn remove_duplicate_equipments(equipments: &mut Vec<&Arc<CalcEquipment>>) {
        let mut dup_ids = Vec::new();

        for (index1, equip1) in equipments.iter().enumerate().rev() {
            for (index2, equip2) in equipments.iter().enumerate() {
                if index1 == index2 {
                    break;
                }

                if equip1.part() == equip2.part()
                    && equip1.skills() == equip2.skills()
                    && equip1.slots() == equip2.slots()
                    && equip1.stats() == equip2.stats()
                {
                    let mut remove_index = index1;

                    if equip1.is_armor()
                        && equip2.is_armor()
                        && !equip1.as_armor().is_anomaly()
                        && equip2.as_armor().is_anomaly()
                    {
                        remove_index = index2;
                    }

                    dup_ids.push(remove_index);
                    break;
                }
            }
        }

        for dup_id in dup_ids {
            equipments.swap_remove(dup_id);
        }
    }

    pub fn get_ge_equipments<'a>(
        equipments: Vec<&'a Arc<CalcEquipment>>,
        req_skills: Option<&SkillsContainer>,
    ) -> (Vec<&'a Arc<CalcEquipment>>, IntMap<usize, Vec<usize>>) {
        let mut left_equipments = Vec::with_capacity(equipments.len());
        let mut ge_equips_map = IntMap::<usize, Vec<usize>>::default();

        for (index, &equip) in equipments.iter().enumerate() {
            let le_vec = vec![index];

            ge_equips_map.insert(equip.uid(), le_vec);
        }

        for (index1, &equip1) in equipments.iter().enumerate() {
            let mut is_le = false;

            for &equip2 in equipments.iter() {
                if equip1.part() != equip2.part() || equip1.uid() == equip2.uid() {
                    continue;
                }

                if equip1.is_le(equip2, false, req_skills) {
                    is_le = true;

                    let existing_set = ge_equips_map.get_mut(&equip2.uid()).unwrap();
                    existing_set.push(index1);
                }
            }

            if !is_le {
                left_equipments.push(equip1);
            }
        }

        for (_, le_uids) in ge_equips_map.iter_mut() {
            for slot_index in (0..MAX_SLOT_LEVEL).rev() {
                le_uids.sort_by_cached_key(|&uid| Reverse(equipments[uid].points()[slot_index]));
            }
        }

        let ge_equips_map = ge_equips_map
            .iter()
            .map(|(&ge_uid, le_indices)| {
                let le_uids = le_indices
                    .iter()
                    .map(|&index| equipments[index].uid())
                    .collect::<Vec<_>>();

                (ge_uid, le_uids)
            })
            .collect::<IntMap<_, _>>();

        (left_equipments, ge_equips_map)
    }

    pub fn remove_le_candidates<'a>(
        all_equipments: &'a Vec<EquipmentsArray>,
    ) -> Vec<EquipmentsArray<'a>> {
        let mut le_equip_ids = HashSet::new();

        for equips1 in all_equipments {
            if Self::is_le_candidate(equips1, all_equipments) {
                le_equip_ids.insert(FullEquipments::get_full_equip_id(equips1));
            }
        }

        let mut ret = all_equipments.clone();

        ret.retain(|equips| !le_equip_ids.contains(&FullEquipments::get_full_equip_id(equips)));

        ret
    }

    pub fn is_le_candidate(
        target_cand: &EquipmentsArray,
        all_candidates: &Vec<EquipmentsArray>,
    ) -> bool {
        let target_equips_id = FullEquipments::get_full_equip_id(target_cand);

        for source_cand in all_candidates {
            if target_equips_id == FullEquipments::get_full_equip_id(source_cand) {
                continue;
            }

            let mut is_le = true;

            for part in ArmorPart::get_all_equip() {
                let part = part.as_usize();

                let source_equip = source_cand[part];

                if source_equip.id().starts_with(EMPTY_ARMOR_PREFIX) {
                    continue;
                }

                let target_equip = target_cand[part];

                if target_equip.uid() != source_equip.uid() {
                    is_le = false;
                    break;
                }
            }

            if is_le {
                return true;
            }
        }

        false
    }

    pub fn get_equips_with_deco_skill<'a>(
        all_calc_equips: &Vec<Vec<&'a Arc<CalcEquipment>>>,
        yes_deco_skills: &SkillsContainer,
    ) -> Vec<Vec<&'a Arc<CalcEquipment>>> {
        let mut ret = Vec::with_capacity(ArmorPart::get_all_equip().len());

        for part_equips in all_calc_equips {
            let mut skill_part_equips = Vec::new();

            for &equip in part_equips {
                for (uid, _) in equip.skills().iter() {
                    if yes_deco_skills.contains(uid) {
                        skill_part_equips.push(equip);
                        break;
                    }
                }
            }

            ret.push(skill_part_equips);
        }

        ret
    }

    pub fn merge_deco_slot_equips<'a>(
        deco_equips: &[Vec<&'a Arc<CalcEquipment>>],
        slot_equips: &[Vec<&'a Arc<CalcEquipment>>],
    ) -> Vec<Vec<&'a Arc<CalcEquipment>>> {
        let mut ret = deco_equips.to_owned();

        for (part, equips) in slot_equips.iter().enumerate() {
            let part_equips = ret.get_mut(part).unwrap();

            for &equip in equips {
                part_equips.push(equip);
            }

            CalcEquipment::sort_by_points(part_equips);
        }

        ret
    }

    pub fn merge_deco_slot_unique_equips<'a>(
        all_deco_slot_equips: &[Vec<&'a Arc<CalcEquipment>>],
        unique_equips: &[&'a Arc<CalcEquipment>],
    ) -> Vec<&'a Arc<CalcEquipment>> {
        let mut ret = all_deco_slot_equips
            .iter()
            .flatten()
            .copied()
            .collect::<Vec<_>>();

        ret.append(&mut unique_equips.to_owned());

        CalcEquipment::sort_by_points(&mut ret);

        ret
    }

    pub fn get_possible_unique_equips<'a>(
        all_equips: &Vec<Vec<&'a Arc<CalcEquipment>>>,
        no_deco_skills: &SkillsContainer,
        empty_equips: &'a EquipmentsArray,
    ) -> (
        Vec<Vec<&'a Arc<CalcEquipment>>>,
        Vec<&'a Arc<CalcEquipment>>,
    ) {
        let mut all_unique_equips = Vec::with_capacity(all_equips.len());
        let mut all_unique_equips_flat = Vec::new();

        for (part, equips) in all_equips.iter().enumerate() {
            let mut part_unique_equips = equips
                .iter()
                .filter_map(|&equip| {
                    let equip_skills = equip.skills();

                    for (uid, _) in no_deco_skills.iter() {
                        if equip_skills.contains(uid) {
                            all_unique_equips_flat.push(equip);
                            return Some(equip);
                        }
                    }

                    None
                })
                .collect::<Vec<_>>();

            part_unique_equips.push(empty_equips[part]);
            all_unique_equips.insert(part, part_unique_equips);

            info!(
                "{} part unique equipments count: {}",
                ArmorPart::from_usize(part).as_str(),
                all_unique_equips[part].len()
            );
        }

        CalcEquipment::sort_by_points(&mut all_unique_equips_flat);

        (all_unique_equips, all_unique_equips_flat)
    }

    pub fn get_possible_general_equips<'a>(
        all_deco_slot_equips: &[Vec<&'a Arc<CalcEquipment>>],
        yes_deco_skills: &SkillsContainer,
        empty_equips: &'a EquipmentsArray,
    ) -> (
        Vec<Vec<&'a Arc<CalcEquipment>>>,
        Vec<&'a Arc<CalcEquipment>>,
    ) {
        let mut flat = Vec::new();

        let mut parts = all_deco_slot_equips
            .iter()
            .enumerate()
            .map(|(part, deco_slot_part_equips)| {
                let (mut part_equips, _) = Self::get_possible_general_part_equips(
                    deco_slot_part_equips,
                    yes_deco_skills,
                    true,
                );

                part_equips.retain(|equip| {
                    let diff = yes_deco_skills.get_diff(equip.skills());

                    for (uid, _) in yes_deco_skills.iter() {
                        if diff.get(uid) <= 0 {
                            return true;
                        }
                    }

                    false
                });

                part_equips.push(empty_equips[part]);

                CalcEquipment::sort_by_points(&mut part_equips);

                info!(
                    "{} part general equipments count: {}",
                    ArmorPart::from_usize(part).as_str(),
                    part_equips.len()
                );

                flat.append(&mut part_equips.clone());

                part_equips
            })
            .collect::<Vec<_>>();

        parts.sort_by_cached_key(|equips| equips.len());

        CalcEquipment::sort_by_points(&mut flat);

        (parts, flat)
    }

    pub fn get_possible_general_part_equips<'a>(
        part_equips: &[&'a Arc<CalcEquipment>],
        yes_deco_skills: &SkillsContainer,
        remove_le_equips: bool,
    ) -> (Vec<&'a Arc<CalcEquipment>>, IntMap<usize, Vec<usize>>) {
        let mut ret = part_equips.to_owned();

        ret.retain(|equip| {
            if equip.id().starts_with(SLOT_ARMOR_PREFIX) {
                return true;
            }

            for (uid, _) in equip.skills().iter() {
                if yes_deco_skills.contains(uid) {
                    return true;
                }
            }

            false
        });

        let ge_equips_map;

        if remove_le_equips {
            (ret, ge_equips_map) = CalcDataManager::get_ge_equipments(ret, Some(yes_deco_skills));
        } else {
            ge_equips_map = IntMap::default();
        }

        (ret, ge_equips_map)
    }
}
