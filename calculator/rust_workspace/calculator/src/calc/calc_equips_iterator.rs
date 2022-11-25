use std::sync::Arc;

use itertools::Itertools;
use log::debug;
use nohash_hasher::{IntMap, IntSet};

use crate::{data::data_manager::DataManager, full_equipments::FullEquipments};

use super::{
    calc_data_manager::CalcDataManager,
    calc_equipment::CalcEquipment,
    calc_point::CalcPoint,
    constant::{EQUIP_PART_COUNT, MAX_SLOT_LEVEL},
    skills::SkillsContainer,
    types::{EquipmentsArray, PointsVec},
};

pub struct CalcEquipmentsIterator<'a> {
    req_points: &'a PointsVec,
    weapon_points: &'a PointsVec,
    left_points: PointsVec,

    is_initialized: bool,
    all_equips: Vec<&'a Arc<CalcEquipment>>,

    all_parts: Vec<usize>,
    all_parts_sum: usize,
    part_to_part_id: [usize; EQUIP_PART_COUNT],
    part_equips: Vec<Vec<&'a Arc<CalcEquipment>>>,
    equip_id_to_part_id: Vec<IntMap<usize, usize>>,
    part_id_to_equip_id: Vec<Vec<usize>>,
    next_equip_id: Vec<Vec<usize>>,
    probe_ids: Vec<(usize, usize)>,

    full_equip_ids: [usize; EQUIP_PART_COUNT],

    dm: &'a DataManager,
    cm: &'a CalcDataManager,
    req_skills: &'a SkillsContainer,
}

impl<'a> CalcEquipmentsIterator<'a> {
    pub fn new(
        all_equips: Vec<&'a Arc<CalcEquipment>>,
        key_equips: &Vec<&'a Arc<CalcEquipment>>,
        req_points: &'a PointsVec,
        weapon_points: &'a PointsVec,
        empty_equips: &'a EquipmentsArray,

        dm: &'a DataManager,
        cm: &'a CalcDataManager,
        req_skills: &'a SkillsContainer,
    ) -> Self {
        let key_equips_points = key_equips
            .iter()
            .map(|equip| equip.points())
            .sum::<PointsVec>();

        let left_points = req_points - key_equips_points - weapon_points;

        let mut all_parts = IntSet::default();

        for equip in all_equips.iter() {
            all_parts.insert(equip.part());
        }

        let mut all_parts = all_parts.iter().map(|&part| part).collect_vec();
        all_parts.sort();

        let mut part_to_part_id = [EQUIP_PART_COUNT; EQUIP_PART_COUNT];

        for (part_id, &part) in all_parts.iter().enumerate() {
            part_to_part_id[part] = part_id;
        }

        let mut part_equips = Vec::new();
        let mut equip_id_to_part_id = Vec::new();
        let mut part_id_to_equip_id = Vec::new();
        let mut next_equip_id = Vec::new();

        for _ in &all_parts {
            part_equips.push(Vec::new());
            equip_id_to_part_id.push(IntMap::default());
            part_id_to_equip_id.push(Vec::new());
            next_equip_id.push(Vec::new());
        }

        for (equip_id, &equip) in all_equips.iter().enumerate() {
            let part_id = part_to_part_id[equip.part()];

            part_equips[part_id].push(equip);

            equip_id_to_part_id[part_id].insert(equip_id, part_equips[part_id].len() - 1);
            part_id_to_equip_id[part_id].push(equip_id);
        }

        for (part_id, equip_ids) in part_id_to_equip_id.iter().enumerate() {
            let mut prev_equip_id = 0;

            for &equip_id in equip_ids {
                for _ in prev_equip_id..equip_id {
                    next_equip_id[part_id].push(equip_id);
                }

                prev_equip_id = equip_id;
            }

            for _ in prev_equip_id..all_equips.len() {
                next_equip_id[part_id].push(all_equips.len());
            }
        }

        let mut full_equip_ids = [usize::MAX; EQUIP_PART_COUNT];

        for equip in empty_equips {
            full_equip_ids[equip.part()] = equip.uid();
        }

        for &equip in key_equips.iter() {
            full_equip_ids[equip.part()] = equip.uid();
        }

        Self {
            req_points,
            weapon_points,
            left_points,

            is_initialized: false,
            all_equips,

            all_parts_sum: all_parts.iter().sum::<usize>(),
            all_parts,
            part_to_part_id,
            part_equips,
            equip_id_to_part_id,
            part_id_to_equip_id,
            next_equip_id,
            probe_ids: Vec::new(),

            full_equip_ids,

            dm,
            cm,
            req_skills,
        }
    }

    pub fn proceed(&mut self) -> bool {
        if !self.is_initialized {
            self.init_first_case();

            if self.has_enough_point() {
                return true;
            }
        }

        'point_loop: loop {
            'probe_loop: for part_id in (0..self.probe_ids.len()).rev() {
                let (part, equip_id) = self.probe_ids[part_id];
                let part_equips_id = self.part_to_part_id[part];

                if self.part_equips[part_equips_id].len()
                    <= self.equip_id_to_part_id[part_equips_id][&equip_id] + 1
                {
                    continue;
                }

                let mut existing_parts = [false; EQUIP_PART_COUNT];
                let mut left_parts = self.all_parts_sum;

                for i in 0..part_id {
                    let (part, _) = self.probe_ids[i];

                    existing_parts[part] = true;
                    left_parts -= part;
                }

                for next_part_id in part_id..self.probe_ids.len() - 1 {
                    let equip_len_diff = self.all_parts.len() - next_part_id - 1;

                    let mut assigned = false;

                    for next_equip_id in equip_id + 1..self.all_equips.len() - equip_len_diff {
                        let equip = self.all_equips[next_equip_id];
                        let part = equip.part();

                        if existing_parts[part] == true {
                            continue;
                        }

                        existing_parts[part] = true;
                        left_parts -= part;

                        self.probe_ids[next_part_id] = (part, next_equip_id);

                        assigned = true;
                        break;
                    }

                    if assigned == false {
                        continue 'probe_loop;
                    }
                }

                let last_part = left_parts;
                let last_part_id = self.part_to_part_id[last_part];

                let next_part_equip_ids = &self.next_equip_id[last_part_id];
                let next_equip_id = next_part_equip_ids[equip_id];
                *self.probe_ids.last_mut().unwrap() = (last_part, next_equip_id);

                'point_check_loop: loop {
                    self.refresh_equipments();

                    let points = self.get_probe_points();

                    if points[0] < self.left_points[0] {
                        continue 'probe_loop;
                    }

                    for slot_index in 1..MAX_SLOT_LEVEL {
                        if points[slot_index] < self.left_points[slot_index] {
                            let last_equip_id = self.probe_ids.last().unwrap().1;

                            let next_equip_id = self.next_equip_id[last_part_id][last_equip_id];

                            if next_equip_id == self.all_equips.len() {
                                self.promote();
                                continue 'point_loop;
                            } else {
                                (*self.probe_ids.last_mut().unwrap()).1 = next_equip_id;
                                continue 'point_check_loop;
                            }
                        }
                    }

                    break;
                }

                return true;
            }

            return false;
        }
    }

    pub fn promote(&mut self) {
        let last = self.probe_ids.last_mut().unwrap();
        let part = last.0;
        let part_id = self.part_to_part_id[part];

        (*last).1 = *self.part_id_to_equip_id[part_id].last().unwrap();
    }

    pub fn refresh_equipments(&mut self) {
        for &(_, equip_id) in self.probe_ids.iter() {
            let equip = self.all_equips[equip_id];
            self.full_equip_ids[equip.part()] = equip.uid();
        }
    }

    pub fn get_indices(&self) -> &Vec<(usize, usize)> {
        &self.probe_ids
    }

    fn get_probe_points(&self) -> PointsVec {
        let mut points = PointsVec::default();

        for &(_, equip_id) in self.probe_ids.iter() {
            points += self.all_equips[equip_id].points();
        }

        points
    }

    fn has_enough_point(&self) -> bool {
        let equipments = self.cm.get_full_equipments(&self.full_equip_ids);

        let skills = FullEquipments::calculate_skills(&equipments);
        let slots_lp = FullEquipments::calculate_equipments_slots_lp(&equipments);

        let equip_points = self
            .dm
            .calc_equip_point_slots_lp(&skills, &slots_lp, self.req_skills)
            + self.weapon_points;

        CalcPoint::is_possible_static(&equip_points, self.req_points)
    }

    fn init_first_case(&mut self) {
        let mut unexisting_parts = IntSet::default();

        for &part in self.all_parts.iter() {
            unexisting_parts.insert(part);
        }

        let mut part_id = 0;

        for equip_id in 0..self.all_equips.len() {
            let part = self.all_equips[equip_id].part();

            if unexisting_parts.contains(&part) == false {
                continue;
            }

            unexisting_parts.remove(&part);

            self.probe_ids.push((part, equip_id));
            part_id += 1;

            if part_id == self.all_parts.len() {
                break;
            }
        }

        self.refresh_equipments();

        self.is_initialized = true;
    }
}

impl<'a> Iterator for CalcEquipmentsIterator<'a> {
    type Item = [usize; EQUIP_PART_COUNT];

    fn next(&mut self) -> Option<Self::Item> {
        let succeeded = self.proceed();

        if succeeded {
            Some(self.full_equip_ids)
        } else {
            None
        }
    }
}
