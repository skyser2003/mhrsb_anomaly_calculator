use std::{cmp::Reverse, sync::Arc};

use crate::data::{
    armor::{AnomalyArmor, ArmorPart, ArmorStat, BaseArmor, SexType, Talisman},
    data_manager::DataManager,
    Language,
};

use super::{
    calc_vector::CalcVector,
    constant::MAX_SLOT_LEVEL,
    deco_combination::DecorationCombination,
    skills::SkillsContainer,
    types::{PointsVec, SkillSlotCount, SlotsVec},
};

#[derive(Clone)]
pub struct CalcArmorInfo {
    current: Arc<BaseArmor>,
    base: Arc<BaseArmor>,

    sex_type: SexType,
    rarity: i8,
}

#[derive(Clone)]
pub struct CalcTalismanInfo {
    base: Arc<Talisman>,
}

#[derive(Clone)]
pub struct CalcEquipment {
    armor: Option<CalcArmorInfo>,
    talisman: Option<CalcTalismanInfo>,

    uid: usize,
    id: String,
    part: usize,

    skills: SkillsContainer,
    slots: SlotsVec,
    slots_lp: SlotsVec,

    points: PointsVec,
}

impl CalcEquipment {
    pub fn new_original_armor(uid: usize, base_armor: Arc<BaseArmor>, dm: &DataManager) -> Self {
        let armor_info = CalcArmorInfo {
            current: base_armor.clone(),
            base: base_armor.clone(),

            sex_type: base_armor.sex_type.clone(),
            rarity: base_armor.rarity,
        };

        let skills = Self::convert_from_base_skills_armor(dm, &armor_info.current);
        let slots = Self::convert_from_base_slots(&base_armor.slots);
        let slots_lp = CalcVector::convert_to_lp_slots(&slots);

        Self {
            armor: Some(armor_info),
            talisman: None,

            uid,
            id: base_armor.id().clone(),
            part: base_armor.part.as_usize(),

            skills,
            slots,
            slots_lp,
            points: PointsVec::default(),
        }
    }

    pub fn new_anomaly_armor(
        uid: usize,
        anomaly_armor: Arc<AnomalyArmor>,
        dm: &DataManager,
    ) -> Self {
        let current = anomaly_armor.affected.clone();

        let armor_info = CalcArmorInfo {
            sex_type: current.sex_type.clone(),
            rarity: current.rarity,

            current: Arc::new(current),
            base: Arc::new(anomaly_armor.original.clone()),
        };

        let skills = Self::convert_from_base_skills_armor(dm, &armor_info.current);
        let slots = Self::convert_from_base_slots(&armor_info.current.slots);
        let slots_lp = CalcVector::convert_to_lp_slots(&slots);

        let id = armor_info.current.id().clone();
        let part = armor_info.current.part.as_usize();

        Self {
            armor: Some(armor_info),
            talisman: None,

            uid,
            id,
            part,

            skills,
            slots,
            slots_lp,

            points: PointsVec::default(),
        }
    }

    pub fn new_talisman(uid: usize, talisman: Arc<Talisman>, dm: &DataManager) -> Self {
        let tali_info = CalcTalismanInfo { base: talisman };

        let skills = Self::convert_from_base_skills_talisman(dm, &tali_info.base);
        let slots = Self::convert_from_base_slots(&tali_info.base.slot_sizes);
        let slots_lp = CalcVector::convert_to_lp_slots(&slots);

        let id = tali_info.base.id().clone();

        Self {
            armor: None,
            talisman: Some(tali_info),

            uid,
            id,
            part: ArmorPart::Talisman.as_usize(),

            skills,
            slots,
            slots_lp,

            points: PointsVec::default(),
        }
    }

    pub fn uid(&self) -> usize {
        self.uid
    }

    pub fn set_uid(&mut self, uid: usize) {
        self.uid = uid;
    }

    pub fn id(&self) -> &String {
        &self.id
    }

    pub fn stats(&self) -> ArmorStat {
        if self.is_armor() {
            self.as_armor().base.stat.clone()
        } else {
            ArmorStat::new_empty()
        }
    }

    pub fn skills(&self) -> &SkillsContainer {
        &self.skills
    }

    pub fn mut_skills(&mut self) -> &mut SkillsContainer {
        &mut self.skills
    }

    pub fn slots(&self) -> &SlotsVec {
        &self.slots
    }

    pub fn slots_lp(&self) -> &SlotsVec {
        &self.slots_lp
    }

    pub fn mut_slots(&mut self) -> &mut SlotsVec {
        &mut self.slots
    }

    pub fn part(&self) -> usize {
        self.part
    }

    pub fn point(&self) -> i32 {
        self.points[0]
    }

    pub fn points(&self) -> &PointsVec {
        &self.points
    }

    pub fn set_point(&mut self, dm: &DataManager, req_skills: &SkillsContainer) {
        self.points = dm.calc_equip_point_slots_lp(&self.skills, &self.slots_lp, req_skills);
    }

    pub fn is_armor(&self) -> bool {
        self.armor.is_some()
    }

    pub fn is_talisman(&self) -> bool {
        self.talisman.is_some()
    }

    pub fn as_armor(&self) -> &CalcArmorInfo {
        self.armor.as_ref().unwrap()
    }

    pub fn as_talisman(&self) -> &CalcTalismanInfo {
        self.talisman.as_ref().unwrap()
    }

    pub fn clone_self(&mut self) -> Self {
        self.clone()
    }

    pub fn is_le(
        &self,
        other: &CalcEquipment,
        include_stat: bool,
        req_skills: Option<&SkillsContainer>,
    ) -> bool {
        // Slot
        let self_slots_lp = self.slots_lp();
        let other_slots_lp = other.slots_lp();

        let slots_possible =
            DecorationCombination::is_possible_static_lp(other_slots_lp, self_slots_lp);

        if !slots_possible {
            return false;
        }

        // Skill
        let base_self_skills = self.skills();
        let base_other_skills = other.skills();

        let is_skill_le = match req_skills {
            Some(req_skills) => {
                let mut is_le = true;

                for (uid, _) in req_skills.iter() {
                    if base_self_skills.get(uid) > base_other_skills.get(uid) {
                        is_le = false;
                        break;
                    }
                }

                is_le
            }
            None => {
                let diff = base_self_skills.get_diff(base_other_skills);

                diff.is_empty()
            }
        };

        if !is_skill_le {
            return false;
        }

        // Stat
        if include_stat
            && self.part != ArmorPart::Talisman.as_usize()
            && self.stats() > other.stats()
        {
            return false;
        }

        true
    }

    pub fn subtract_skills(&self, req_skills: &mut SkillsContainer) {
        req_skills.sub(self.skills())
    }

    pub fn subtract_skills_mut(&mut self, req_skills: &mut SkillsContainer) {
        req_skills.sub_mut(&mut self.skills);
    }

    pub fn contains_skill(&self, skill_uid: usize) -> bool {
        self.skills().contains(skill_uid)
    }

    pub fn convert_from_base_skills_armor(
        dm: &DataManager,
        base_armor: &Arc<BaseArmor>,
    ) -> SkillsContainer {
        let mut skills = SkillsContainer::from(dm.get_empty_skill_levels());

        for (skill_id, armor_skill) in &base_armor.skills {
            let uid = dm.get_skill_uid(skill_id); // TODO: use serde deserialize to embed uid into BaseArmor

            skills.set(uid, armor_skill.level);
        }

        skills
    }

    pub fn convert_from_base_skills_talisman(
        dm: &DataManager,
        talisman: &Arc<Talisman>,
    ) -> SkillsContainer {
        let mut skills = SkillsContainer::from(dm.get_empty_skill_levels());

        for skill_info in &talisman.skills {
            let skill_id = &skill_info.id;
            let level = skill_info.level;
            let uid = dm.get_skill_uid(skill_id); // TODO: use serde deserialize to embed uid into Talisman

            skills.set(uid, level);
        }

        skills
    }

    pub fn convert_from_base_slots(base_slots: &Vec<SkillSlotCount>) -> SlotsVec {
        let mut ret = SlotsVec::default();

        for slot_size in base_slots {
            if *slot_size == 0 {
                continue;
            }

            ret[*slot_size as usize - 1] += 1;
        }

        ret
    }

    pub fn sort_by_points(equipments: &mut Vec<&Arc<CalcEquipment>>) {
        for slot_index in (0..MAX_SLOT_LEVEL).rev() {
            equipments.sort_by_cached_key(|equip| Reverse(equip.points()[slot_index]));
        }
    }
}

impl PartialEq for CalcEquipment {
    fn eq(&self, other: &Self) -> bool {
        self.slots() == other.slots() && self.skills() == other.skills()
    }
}

impl CalcArmorInfo {
    pub fn id(&self) -> &String {
        self.current.id()
    }

    pub fn base_id(&self) -> &String {
        self.base.id()
    }

    pub fn rarity(&self) -> SkillSlotCount {
        self.rarity
    }

    pub fn sex_type(&self) -> &SexType {
        &self.sex_type
    }

    pub fn is_anomaly(&self) -> bool {
        BaseArmor::is_anomaly_armor(self.current.id())
    }

    pub fn name(&self, lang: &Language) -> String {
        let existing = self.current.names.get(lang);

        match existing {
            Some(name) => name.clone(),
            None => "SYSTEM_PART".to_string(), // TODO: empty armor, slot only armor
        }
    }
}
