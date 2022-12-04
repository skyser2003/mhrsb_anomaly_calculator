use std::{collections::HashMap, sync::Arc};

use crate::{
    calc::{constant::EQUIP_PART_COUNT, types::EquipmentsArray},
    calc::{skills::SkillsContainer, types::SlotsVec},
    CalcEquipment,
};

#[derive(Clone)]
pub struct FullEquipments<'a> {
    pub weapon_slots_lp: &'a SlotsVec,
    pub equipments: &'a EquipmentsArray<'a>,

    pub all_skills: SkillsContainer,
    pub avail_slots_lp: SlotsVec,
}

impl<'a> FullEquipments<'a> {
    pub fn new(
        weapon_slots_lp: &'a SlotsVec,
        equipments: &'a EquipmentsArray,
    ) -> FullEquipments<'a> {
        let all_skills = FullEquipments::calculate_skills(equipments);
        let avail_slots_lp = FullEquipments::calculate_slots_lp(weapon_slots_lp, equipments);

        FullEquipments {
            weapon_slots_lp,
            equipments,
            all_skills,
            avail_slots_lp,
        }
    }

    pub fn contains_skills(&self, req_skills: &SkillsContainer) -> bool {
        FullEquipments::contains_skills_static(&self.all_skills, req_skills)
    }

    pub fn contains_skills_static(
        equip_skills: &SkillsContainer,
        req_skills: &SkillsContainer,
    ) -> bool {
        let skill_diff = req_skills.get_diff(equip_skills);

        skill_diff.is_empty()
    }

    pub fn calculate_skills(equipments: &EquipmentsArray) -> SkillsContainer {
        let mut skills = SkillsContainer::new();

        for equip in equipments {
            skills.add(equip.skills());
        }

        skills
    }

    pub fn calculate_equipments_slots(equipments: &EquipmentsArray) -> SlotsVec {
        let mut slots = SlotsVec::default();

        for equip in equipments {
            slots += equip.slots();
        }

        slots
    }

    pub fn calculate_equipments_slots_lp(equipments: &EquipmentsArray) -> SlotsVec {
        let mut slots = SlotsVec::default();

        for equip in equipments {
            slots += equip.slots_lp();
        }

        slots
    }

    pub fn calculate_slots(weapon_slots: &SlotsVec, equipments: &EquipmentsArray) -> SlotsVec {
        let mut slots = *weapon_slots;

        for equip in equipments {
            slots += equip.slots();
        }

        slots
    }

    pub fn calculate_slots_lp(
        weapon_slots_lp: &SlotsVec,
        equipments: &EquipmentsArray,
    ) -> SlotsVec {
        let mut sum = *weapon_slots_lp;

        for equip in equipments {
            sum += equip.slots_lp();
        }

        sum
    }

    pub fn save_by_part(equipments: &'a EquipmentsArray) -> HashMap<usize, &'a Arc<CalcEquipment>> {
        let mut equipments_by_part = HashMap::new();

        for &equipment in equipments {
            equipments_by_part.insert(equipment.part(), equipment);
        }

        equipments_by_part
    }

    pub fn subtract_skills(equipments: &'a EquipmentsArray, req_skills: &mut SkillsContainer) {
        for equip in equipments {
            equip.subtract_skills(req_skills);
        }
    }

    pub fn get_full_equip_id(equipments: &EquipmentsArray) -> u128 {
        let mut ret = 0;

        let mut equips_by_part: [Option<&Arc<CalcEquipment>>; EQUIP_PART_COUNT] =
            Default::default();

        for &equip in equipments {
            equips_by_part[equip.part()] = Some(equip);
        }

        for (part, equip) in equips_by_part.iter().enumerate() {
            ret += (equip.unwrap().uid() + 1) as u128 * 10000_u128.pow(part as u32);
        }

        ret
    }

    pub fn get_full_equip_point(equipments: &EquipmentsArray) -> i32 {
        equipments.iter().map(|equip| equip.point()).sum()
    }

    pub fn get_full_equip_point_info(
        weapon_slots: &SlotsVec,
        equipments: &EquipmentsArray,
        req_skills: &SkillsContainer,
    ) -> (SkillsContainer, SlotsVec) {
        let mut skills = FullEquipments::calculate_skills(equipments);
        let slots = FullEquipments::calculate_slots(weapon_slots, equipments);

        for (uid, level) in req_skills.iter() {
            skills.set(uid, skills.get(uid).min(level));
        }

        (skills, slots)
    }

    pub fn parse_full_equip_id(full_equip_id: u128) -> [usize; EQUIP_PART_COUNT] {
        let mut uids = [0; EQUIP_PART_COUNT];

        for part in (0..EQUIP_PART_COUNT).rev() {
            let uid = full_equip_id / 10000;

            uids[part] = (uid - 1) as usize;
        }

        uids
    }
}
