use std::cmp::Reverse;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use ::num::integer::lcm;
use log::{debug, info};
use nohash_hasher::IntMap;

use crate::calc::calc_vector::CalcVector;
use crate::calc::constant::EQUIP_PART_COUNT;
use crate::calc::deco_combination::{DecorationCombination, DecorationCombinations};
use crate::calc::skills::SkillsContainer;
use crate::calc::types::{PointsVec, SkillSlotCount, SlotsVec};

use super::armor::{
    AnomalyArmor, ArmorPart, ArmorSkill, ArmorStat, BaseArmor, SexType, SkillIdLevel, Talisman,
    ANOMALY_ARMOR_PREFIX,
};
use super::deco::Decoration;
use super::skill::Skill;

type SlotOnlyArmors = Vec<HashMap<String, Arc<BaseArmor>>>;
type ArmorsBySlot = Vec<HashMap<String, Vec<Arc<BaseArmor>>>>;

pub struct DataManager {
    armors: HashMap<String, Arc<BaseArmor>>,
    skills: Vec<Skill>,
    decos: HashMap<String, Decoration>,

    skill_id_map: HashMap<String, usize>,
    empty_skill_levels: Vec<SkillSlotCount>,

    decos_by_skill: Vec<Vec<Decoration>>,
    deco_by_level: Vec<Vec<Decoration>>,
    skills_point: Vec<Vec<i32>>,
    point_lcm: i32,
    single_deco_skills: Vec<Option<Decoration>>,
    deco_combinations: DecorationCombinations,

    slot_only_armors: SlotOnlyArmors,
    armors_by_slot: ArmorsBySlot,
    empty_armors: [Arc<BaseArmor>; EQUIP_PART_COUNT - 1],
    all_anomaly_armors: HashMap<String, Arc<AnomalyArmor>>,
    file_anomaly_armors: HashMap<String, Arc<AnomalyArmor>>,
    manual_anomaly_armors: HashMap<String, Arc<AnomalyArmor>>,

    bases_by_part: HashMap<usize, Vec<Arc<BaseArmor>>>,
    anomalies_by_part: HashMap<usize, Vec<Arc<BaseArmor>>>,

    all_talismans: Vec<Arc<Talisman>>,
    file_talismans: Vec<Arc<Talisman>>,
    manual_talismans: Vec<Arc<Talisman>>,

    slot_only_talismans: HashMap<String, Arc<Talisman>>,
    talismans_by_slot: HashMap<String, Vec<Arc<Talisman>>>,
    empty_talisman: Arc<Talisman>,

    armor_name_dict: HashMap<String, String>,
    skill_name_dict: HashMap<String, String>,
}

impl DataManager {
    pub async fn from_sources(
        armors_filename: &Path,
        skills_filename: &Path,
        decos_filename: &Path,
    ) -> DataManager {
        use crate::utils::parse::parse_data;

        let armor_task = async {
            parse_data::<BaseArmor, _>(armors_filename)
                .into_iter()
                .map(|armor| (armor.id().clone(), armor))
                .collect::<HashMap<_, _>>()
        };
        let skill_task = async {
            parse_data::<Skill, _>(skills_filename)
                .into_iter()
                .map(|skill| (skill.id.clone(), skill))
                .collect::<HashMap<_, _>>()
        };
        let deco_task = async {
            parse_data::<Decoration, _>(decos_filename)
                .into_iter()
                .map(|deco| (deco.id.clone(), deco))
                .collect::<HashMap<_, _>>()
        };
        let (armors, skills, decos) = futures_util::join!(armor_task, skill_task, deco_task);

        Self::new(armors, skills, decos)
    }

    pub fn new(
        armors: HashMap<String, BaseArmor>,
        skills: HashMap<String, Skill>,
        decos: HashMap<String, Decoration>,
    ) -> Self {
        let armors = armors
            .into_iter()
            .map(|(id, armor)| (id, Arc::new(armor)))
            .collect::<HashMap<_, _>>();

        let mut armor_name_dict = HashMap::<String, String>::new();
        let mut skill_name_dict = HashMap::<String, String>::new();

        let skills = skills
            .into_iter()
            .map(|(_, skill)| skill)
            .collect::<Vec<Skill>>();

        let skill_id_map = skills
            .iter()
            .enumerate()
            .map(|(index, skill)| (skill.id.clone(), index))
            .collect::<HashMap<String, usize>>();

        let empty_skill_levels = vec![0; skills.len()];

        for pair in &armors {
            let armor = pair.1;

            for lang_name in &armor.names {
                let name = lang_name.1;

                armor_name_dict.insert(name.to_string(), armor.id().clone());
            }
        }

        for skill in &skills {
            for lang_name in &skill.names {
                let name = lang_name.1;

                skill_name_dict.insert(name.to_string(), skill.id.to_string());
            }
        }

        let mut decos_by_skill = Vec::new();
        let mut single_deco_skills = Vec::new();
        let mut deco_by_level = Vec::new();
        let mut skills_point = Vec::new();

        for _ in &skills {
            decos_by_skill.push(Vec::new());
            single_deco_skills.push(None);
            deco_by_level.push(Vec::new());
            skills_point.push(Vec::new());
        }

        for deco in decos.values() {
            let skill_id = &deco.skill_id;
            let uid = skill_id_map[skill_id];

            let decos_vec = &mut decos_by_skill[uid];

            decos_vec.push(deco.clone());
        }

        for decos in decos_by_skill.iter_mut() {
            if decos.len() <= 1 {
                continue;
            }

            decos.sort_by_key(|a| a.skill_level);
        }

        let mut point_lcm = 1_i32;

        for decos in decos_by_skill.iter() {
            if decos.is_empty() {
                continue;
            }

            for deco in decos {
                let point = deco.skill_level;

                point_lcm = lcm(point_lcm, point as i32);
            }
        }

        for decos in decos_by_skill.iter() {
            if decos.is_empty() {
                continue;
            }

            let uid = skill_id_map[&decos[0].skill_id];

            let mut prev_level = 0;

            for deco in decos {
                let point = deco.skill_level;

                for _ in prev_level..deco.skill_level {
                    skills_point[uid].push(point_lcm / point as i32);
                    deco_by_level[uid].push(deco.clone());
                }

                prev_level = deco.skill_level;
            }

            let last_deco = decos.last().unwrap();

            for _ in last_deco.skill_level..99 {
                skills_point[uid].push(point_lcm / last_deco.skill_level as i32);
                deco_by_level[uid].push(last_deco.clone());
            }
        }

        for (uid, decos) in decos_by_skill.iter().enumerate() {
            if decos.len() == 1 {
                single_deco_skills[uid] = Some(decos[0].clone());
            }
        }

        let deco_combinations = DecorationCombinations::new(&decos_by_skill, &skills);

        let (slot_only_armors, armors_by_slot) = Self::extract_slot_armors(&armors);

        let mut empty_parts = [0; EQUIP_PART_COUNT - 1];
        for part in ArmorPart::get_all_armor() {
            let part = part.as_usize();
            empty_parts[part] = part;
        }

        let empty_armors =
            empty_parts.map(|part| Arc::new(BaseArmor::create_empty(ArmorPart::from_usize(part))));

        let mut bases_by_part = HashMap::new();

        bases_by_part.insert(ArmorPart::Helm.as_usize(), Vec::new());
        bases_by_part.insert(ArmorPart::Torso.as_usize(), Vec::new());
        bases_by_part.insert(ArmorPart::Arm.as_usize(), Vec::new());
        bases_by_part.insert(ArmorPart::Waist.as_usize(), Vec::new());
        bases_by_part.insert(ArmorPart::Feet.as_usize(), Vec::new());

        let anomalies_by_part = bases_by_part.clone();

        for armor in armors.values() {
            let part = &armor.part;
            bases_by_part
                .get_mut(&part.as_usize())
                .unwrap()
                .push(armor.clone());
        }

        DataManager {
            armors,
            skills,
            decos,
            skill_id_map,
            empty_skill_levels,
            decos_by_skill,
            single_deco_skills,
            deco_by_level,
            skills_point,
            point_lcm,
            deco_combinations,
            slot_only_armors,
            armors_by_slot,
            empty_armors,
            armor_name_dict,
            skill_name_dict,
            bases_by_part,
            anomalies_by_part,
            empty_talisman: Arc::new(Talisman::create_empty()),
            all_anomaly_armors: Default::default(),
            file_anomaly_armors: Default::default(),
            manual_anomaly_armors: Default::default(),
            all_talismans: Default::default(),
            file_talismans: Default::default(),
            manual_talismans: Default::default(),
            slot_only_talismans: Default::default(),
            talismans_by_slot: Default::default(),
        }
    }

    pub fn debug_deco_combs(&self) {
        self.deco_combinations.debug(self);
    }

    fn extract_slot_armors(
        armors: &HashMap<String, Arc<BaseArmor>>,
    ) -> (SlotOnlyArmors, ArmorsBySlot) {
        let mut slot_only_armors = Vec::new();
        let mut armors_by_slot = Vec::new();

        for _part in ArmorPart::get_all_armor() {
            slot_only_armors.push(HashMap::new());
            armors_by_slot.push(HashMap::new());
        }

        for armor in armors.values() {
            let slot_armor_id = BaseArmor::get_slot_armor_id(&armor.slots);

            let part_slot_only_armors = slot_only_armors.get_mut(armor.part.as_usize()).unwrap();
            if !part_slot_only_armors.contains_key(&slot_armor_id) {
                part_slot_only_armors.insert(
                    slot_armor_id.clone(),
                    Arc::new(BaseArmor::get_slot_armor(
                        armor.part.clone(),
                        slot_armor_id.clone(),
                    )),
                );
            }

            let part_slot_armors = armors_by_slot.get_mut(armor.part.as_usize()).unwrap();

            let existing = part_slot_armors.get_mut(&slot_armor_id);
            let slot_armors;

            if let Some(existing_armors) = existing {
                slot_armors = existing_armors;
            } else {
                part_slot_armors.insert(slot_armor_id.clone(), Vec::new());
                slot_armors = part_slot_armors.get_mut(&slot_armor_id).unwrap();
            }

            slot_armors.push(armor.clone());
        }

        (slot_only_armors, armors_by_slot)
    }

    pub fn refresh_anomalies(&mut self) {
        self.all_anomaly_armors.clear();

        for (id, equip) in self.file_anomaly_armors.iter() {
            self.all_anomaly_armors.insert(id.clone(), equip.clone());
        }

        for (id, equip) in self.manual_anomaly_armors.iter() {
            self.all_anomaly_armors.insert(id.clone(), equip.clone());
        }

        for part_armors in self.anomalies_by_part.values_mut() {
            part_armors.clear();
        }

        for anomaly in self.all_anomaly_armors.values() {
            let part = &anomaly.original.part;

            self.anomalies_by_part
                .get_mut(&part.as_usize())
                .unwrap()
                .push(Arc::new(anomaly.affected.clone()));
        }

        let mut all_armors = HashMap::new();

        for (id, armor) in &self.armors {
            all_armors.insert(id.clone(), armor.clone());
        }

        for (id, armor) in &self.all_anomaly_armors {
            all_armors.insert(id.clone(), Arc::new(armor.affected.clone()));
        }

        (self.slot_only_armors, self.armors_by_slot) = Self::extract_slot_armors(&all_armors);
    }

    pub fn set_file_anomalies(&mut self, anomalies: Vec<AnomalyArmor>) {
        let mut anomalies = anomalies;

        for (index, armor) in anomalies.iter_mut().enumerate() {
            armor.affected.id = format!(
                "{}_file_{}_{}",
                ANOMALY_ARMOR_PREFIX,
                index,
                armor.original.id()
            );
        }

        self.file_anomaly_armors = anomalies
            .into_iter()
            .map(|anomaly| (anomaly.affected.id.clone(), Arc::new(anomaly)))
            .collect::<HashMap<_, _>>();

        self.refresh_anomalies();
    }

    pub fn add_manual_anomaly(&mut self, mut anomaly: AnomalyArmor) -> Arc<AnomalyArmor> {
        anomaly.affected.id = format!(
            "{}_manual_{}_{}",
            ANOMALY_ARMOR_PREFIX,
            self.file_anomaly_armors.len(),
            anomaly.original.id()
        );

        let insert_value = Arc::new(anomaly);

        self.manual_anomaly_armors
            .insert(insert_value.affected.id.clone(), insert_value.clone());

        self.refresh_anomalies();

        insert_value
    }

    pub fn remove_manual_anomaly(&mut self, id: &String) {
        let removed = self.manual_anomaly_armors.remove(id);

        if removed.is_some() {
            self.refresh_anomalies();
        }
    }

    pub fn clear_file_anomalies(&mut self) {
        self.file_anomaly_armors.clear();
        self.refresh_anomalies();
    }

    pub fn clear_manual_anomalies(&mut self) {
        self.manual_talismans.clear();
        self.refresh_anomalies();
    }

    pub fn refresh_talismans(&mut self) {
        self.all_talismans.clear();
        self.slot_only_talismans.clear();
        self.talismans_by_slot.clear();

        for tali in self.file_talismans.iter() {
            self.all_talismans.push(tali.clone());
        }

        for tali in self.manual_talismans.iter() {
            self.all_talismans.push(tali.clone());
        }

        for tali in &self.all_talismans {
            let slot_tali_id = BaseArmor::get_slot_armor_id(&tali.slot_sizes);

            if !self.slot_only_talismans.contains_key(&slot_tali_id) {
                let slot_talisman = Talisman::get_slot_talisman(slot_tali_id.clone());

                self.slot_only_talismans
                    .insert(slot_tali_id.clone(), Arc::new(slot_talisman));

                self.talismans_by_slot
                    .insert(slot_tali_id.clone(), Vec::new());
            }

            self.talismans_by_slot
                .get_mut(&slot_tali_id)
                .unwrap()
                .push(tali.clone());
        }
    }

    pub fn set_file_talismans(&mut self, talismans: Vec<Talisman>) {
        self.file_talismans = talismans.into_iter().map(Arc::new).collect();

        self.refresh_talismans();
    }

    pub fn add_manual_talisman(&mut self, talisman: &Talisman) -> Arc<Talisman> {
        let manual_id = format!("talisman_manual_{}", self.manual_talismans.len());

        let mut real_skills = Vec::new();

        for info in talisman.skills.iter() {
            if info.id.is_empty() || info.level == 0 {
                continue;
            }

            real_skills.push(info.clone());
        }

        let mut real_slots = talisman.slot_sizes.clone();

        real_slots.sort_by_key(|&count| Reverse(count));

        let real_talisman = Talisman::new(manual_id, real_skills, real_slots);
        let arc = Arc::new(real_talisman);

        self.manual_talismans.push(arc.clone());

        debug!("Count: {}", self.manual_talismans.len());

        self.refresh_talismans();

        arc
    }

    pub fn remove_manual_talisman(&mut self, id: &String) -> bool {
        let mut remove_index = -1;

        for (index, tali) in self.manual_talismans.iter().enumerate() {
            if tali.id() == id {
                remove_index = index as i32;
                break;
            }
        }

        if remove_index == -1 {
            return false;
        }

        self.manual_talismans.remove(remove_index as usize);

        self.refresh_talismans();

        true
    }

    pub fn clear_file_talismans(&mut self) {
        self.file_talismans.clear();
        self.refresh_talismans();
    }

    pub fn clear_manual_talismans(&mut self) {
        self.manual_talismans.clear();
        self.refresh_talismans();
    }

    pub fn get_manual_talismans(&self) -> &Vec<Arc<Talisman>> {
        &self.manual_talismans
    }

    pub fn create_anomaly(
        &self,
        original_id: &str,
        skill_diffs_vec: &Vec<SkillIdLevel>,
        slot_diffs: &[SkillSlotCount],
        stat_diff: &ArmorStat,
    ) -> AnomalyArmor {
        let original = self.get_armor(&original_id.to_string());

        let mut skill_diffs = HashMap::with_capacity(skill_diffs_vec.len());

        for info in skill_diffs_vec.iter() {
            if info.id.is_empty() || info.level == 0 {
                continue;
            }

            skill_diffs.insert(info.id.clone(), ArmorSkill { level: info.level });
        }

        AnomalyArmor::new(
            original,
            stat_diff.clone(),
            slot_diffs.to_owned(),
            skill_diffs,
        )
    }

    pub fn get_empty_skill_levels(&self) -> Vec<SkillSlotCount> {
        self.empty_skill_levels.clone()
    }

    pub fn get_parts(&self, part: usize) -> Vec<&Arc<BaseArmor>> {
        let mut ret = Vec::new();

        for part_armors in self.bases_by_part.get(&part).unwrap() {
            ret.push(part_armors);
        }

        for part_anomaly in self.anomalies_by_part.get(&part).unwrap() {
            ret.push(part_anomaly);
        }

        ret
    }

    pub fn get_parts_clone(&self, part: usize) -> Vec<Arc<BaseArmor>> {
        let mut ret = Vec::new();

        for part_armors in self.bases_by_part.get(&part).unwrap() {
            ret.push(part_armors.clone());
        }

        for part_anomaly in self.anomalies_by_part.get(&part).unwrap() {
            ret.push(part_anomaly.clone());
        }

        ret
    }

    pub fn get_anomaly_armor(&self, armor_id: &String) -> Option<&Arc<AnomalyArmor>> {
        if !BaseArmor::is_anomaly_armor(armor_id) {
            return None;
        }

        self.all_anomaly_armors.get(armor_id)
    }

    pub fn get_anomaly_armors(&self) -> &HashMap<String, Arc<AnomalyArmor>> {
        &self.all_anomaly_armors
    }

    pub fn get_file_anomaly_armors(&self) -> &HashMap<String, Arc<AnomalyArmor>> {
        &self.file_anomaly_armors
    }

    pub fn get_file_talismans(&self) -> &Vec<Arc<Talisman>> {
        &self.file_talismans
    }

    pub fn get_skill_uid(&self, skill_id: &str) -> usize {
        self.skill_id_map[skill_id]
    }

    pub fn get_deco_by_skill_id(&self, skill_uid: usize) -> &Vec<Decoration> {
        &self.decos_by_skill[skill_uid]
    }

    pub fn get_deco_size_by_level(&self, uid: usize, level: SkillSlotCount) -> &Decoration {
        let level_index = level as usize - 1;

        &self.deco_by_level[uid][level_index]
    }

    pub fn get_skill_point(&self, uid: usize, level: SkillSlotCount) -> i32 {
        let level_index = level as usize - 1;

        self.skills_point[uid][level_index]
    }

    pub fn has_decoration(&self, skill_uid: usize) -> bool {
        !self.decos_by_skill[skill_uid].is_empty()
    }

    pub fn get_leftover_skills(
        &self,
        req_skills: &SkillsContainer,
    ) -> (SkillsContainer, SkillsContainer) {
        let mut yes_deco_skills = SkillsContainer::new();
        let mut no_deco_skills = SkillsContainer::new();

        for (skill_uid, level) in req_skills.iter() {
            let decos = self.get_deco_by_skill_id(skill_uid);

            if !decos.is_empty() {
                yes_deco_skills.set(skill_uid, level);
            } else {
                no_deco_skills.set(skill_uid, level);
            }
        }

        (no_deco_skills, yes_deco_skills)
    }

    pub fn get_skills_by_deco(
        &self,
        req_skills: &SkillsContainer,
    ) -> (SkillsContainer, SkillsContainer, SkillsContainer) {
        let mut no_deco_skills = SkillsContainer::new();
        let mut single_deco_skills = SkillsContainer::new();
        let mut multi_deco_skills = SkillsContainer::new();

        for (skill_uid, level) in req_skills.iter() {
            let decos = self.get_deco_by_skill_id(skill_uid);

            if decos.is_empty() {
                no_deco_skills.set(skill_uid, level);
            } else if decos.len() == 1 {
                single_deco_skills.set(skill_uid, level);
            } else {
                multi_deco_skills.set(skill_uid, level);
            }
        }

        (no_deco_skills, single_deco_skills, multi_deco_skills)
    }

    pub fn get_single_deco_skills(
        &self,
        req_skills: &SkillsContainer,
    ) -> (Vec<usize>, Vec<(SkillSlotCount, SkillSlotCount)>) {
        let mut ids = Vec::with_capacity(req_skills.len());
        let mut single_deco_skills = Vec::with_capacity(req_skills.len());

        for (skill_uid, _) in req_skills.iter() {
            let deco = &self.single_deco_skills[skill_uid];

            if deco.is_some() {
                ids.push(skill_uid);
                single_deco_skills
                    .push((deco.as_ref().unwrap().slot_size, req_skills.get(skill_uid)));
            }
        }

        (ids, single_deco_skills)
    }

    pub fn get_single_decos(&self, uids: Vec<usize>) -> Vec<(usize, &Decoration)> {
        uids.iter()
            .filter_map(|&uid| {
                let deco = &self.single_deco_skills[uid];

                if deco.is_none() {
                    None
                } else {
                    Some((uid, self.single_deco_skills[uid].as_ref().unwrap()))
                }
            })
            .collect()
    }

    pub fn skills_to_ids(&self, skills: &SkillsContainer) -> HashMap<&String, SkillSlotCount> {
        skills
            .iter()
            .filter_map(|(uid, level)| {
                if level == 0 {
                    return None;
                }

                Some((&self.skills[uid].id, level))
            })
            .collect()
    }

    pub fn get_skills(&self) -> &Vec<Skill> {
        &self.skills
    }

    pub fn get_skill_name_dict(&self) -> &HashMap<String, String> {
        &self.skill_name_dict
    }

    pub fn get_armor_name_dict(&self) -> &HashMap<String, String> {
        &self.armor_name_dict
    }

    pub fn get_armors(&self) -> &HashMap<String, Arc<BaseArmor>> {
        &self.armors
    }

    pub fn get_talismans(&self) -> &Vec<Arc<Talisman>> {
        &self.all_talismans
    }

    pub fn get_armors_by_slot(
        &self,
        part: usize,
        sex_type: &SexType,
        equip_id: &String,
    ) -> Vec<&Arc<BaseArmor>> {
        self.armors_by_slot[part][equip_id]
            .iter()
            .filter(|armor| Self::is_valid_sextype(&armor.sex_type, sex_type))
            .collect()
    }

    pub fn get_talismans_by_slot(&self, equip_id: &String) -> &Vec<Arc<Talisman>> {
        &self.talismans_by_slot[equip_id]
    }

    pub fn get_slot_only_armors(&self) -> &[HashMap<String, Arc<BaseArmor>>] {
        &self.slot_only_armors
    }

    pub fn get_slot_only_talismans(&self) -> &HashMap<String, Arc<Talisman>> {
        &self.slot_only_talismans
    }

    pub fn get_skill(&self, uid: usize) -> &Skill {
        &self.skills[uid]
    }

    pub fn get_armor(&self, equip_id: &String) -> &BaseArmor {
        &self.armors[equip_id]
    }

    pub fn get_deco_combs(&self, skill_uid: usize) -> &Vec<Vec<SlotsVec>> {
        self.deco_combinations.get(skill_uid)
    }

    pub fn get_possible_deco_combs_lp(
        &self,
        req_skills: &SkillsContainer,
        avail_slots_lp: &SlotsVec,
    ) -> Vec<SlotsVec> {
        self.deco_combinations
            .get_possible_combs_lp(req_skills, avail_slots_lp)
    }

    pub fn get_possible_deco_combs_sum(&self, req_skills: &SkillsContainer) -> SlotsVec {
        self.deco_combinations
            .get_possible_deco_combs_sum(req_skills)
    }

    pub fn get_full_possible_deco_combs(
        &self,
        req_skills: &SkillsContainer,
    ) -> Arc<Vec<DecorationCombination>> {
        self.deco_combinations.get_full_possible_combs(req_skills)
    }

    pub fn check_possible_deco_combs_lp(
        &self,
        req_skills: &SkillsContainer,
        avail_slots_lp: &SlotsVec,
    ) -> bool {
        if req_skills.is_empty() {
            return true;
        }

        let min_slots_sum = self.get_possible_deco_combs_sum(req_skills);

        if !DecorationCombination::is_possible_static_lp(avail_slots_lp, &min_slots_sum) {
            return false;
        }

        let comb_lps = self.get_possible_deco_combs_lp(req_skills, avail_slots_lp);

        !comb_lps.is_empty()
    }

    pub fn has_deco_possible_combs_lp(
        &self,
        req_skills: &SkillsContainer,
        armor_slots_lp: &SlotsVec,
    ) -> bool {
        self.deco_combinations
            .has_possible_combs_lp(req_skills, armor_slots_lp)
    }

    pub fn empty_armors(&self) -> &[Arc<BaseArmor>; EQUIP_PART_COUNT - 1] {
        &self.empty_armors
    }

    pub fn empty_talisman(&self) -> &Talisman {
        &self.empty_talisman
    }

    pub fn calc_equip_point(
        &self,
        skills: &SkillsContainer,
        slots: &SlotsVec,
        req_skills: &SkillsContainer,
    ) -> i32 {
        self.calc_equip_skill_point(skills, req_skills) + self.calc_slot_point(slots)
    }

    pub fn calc_equip_point_slots_lp(
        &self,
        skills: &SkillsContainer,
        slots_lp: &SlotsVec,
        req_skills: &SkillsContainer,
    ) -> PointsVec {
        let mut equip_points_lp = self.calc_equip_skill_point_slots_lp(skills, req_skills);
        self.calc_slot_point_slots_lp_mut(slots_lp, &mut equip_points_lp);

        equip_points_lp
    }

    pub fn calc_req_point_slots_lp(
        &self,
        skills: &SkillsContainer,
        slots_lp: &SlotsVec,
    ) -> PointsVec {
        let mut equip_points_lp = self.calc_req_skill_point_slots_lp(skills);
        self.calc_slot_point_slots_lp_mut(slots_lp, &mut equip_points_lp);

        equip_points_lp
    }

    pub fn calc_equip_skill_point(
        &self,
        skills: &SkillsContainer,
        req_skills: &SkillsContainer,
    ) -> i32 {
        let mut points = 0;

        for (uid, level) in skills.iter() {
            let req_level = req_skills.get(uid);

            if req_level == 0 {
                continue;
            }

            if self.skills_point[uid].is_empty() {
                continue;
            }

            let point = self.get_skill_point(uid, req_level);

            points += level.min(req_level) as i32 * point;
        }

        points
    }

    pub fn calc_req_skill_point(&self, skills: &SkillsContainer) -> i32 {
        let mut points = 0;

        for (uid, level) in skills.iter() {
            if self.skills_point[uid].is_empty() {
                continue;
            }

            let point = self.get_skill_point(uid, level);

            points += level as i32 * point;
        }

        points
    }

    pub fn calc_equip_skill_point_slots_lp(
        &self,
        skills: &SkillsContainer,
        req_skills: &SkillsContainer,
    ) -> PointsVec {
        let mut points = PointsVec::default();

        for (uid, level) in skills.iter() {
            let req_level = req_skills.get(uid);

            if req_level == 0 {
                continue;
            }

            if self.skills_point[uid].is_empty() {
                continue;
            }

            let deco = &self.get_deco_by_skill_id(uid)[0];
            let point = self.get_skill_point(uid, req_level);

            let slot_index = deco.slot_size as usize - 1;

            points[slot_index] += level.min(req_level) as i32 * point;
        }

        CalcVector::convert_to_lp_points_mut(&mut points);

        points
    }

    pub fn calc_req_skill_point_slots_lp(&self, req_skills: &SkillsContainer) -> PointsVec {
        let mut points = PointsVec::default();

        for (uid, req_level) in req_skills.iter() {
            let skills_point = &self.skills_point[uid];

            if skills_point.is_empty() {
                continue;
            }

            let deco = &self.get_deco_by_skill_id(uid)[0];
            let point = skills_point[req_level as usize - 1];

            let slot_index = deco.slot_size as usize - 1;

            points[slot_index] += req_level as i32 * point;
        }

        CalcVector::convert_to_lp_points_mut(&mut points);

        points
    }

    pub fn calc_req_skill_point_slots_lp_by_uids(
        &self,
        req_skills: &SkillsContainer,
        req_uids: &Vec<usize>,
    ) -> PointsVec {
        let mut points = PointsVec::default();

        for &uid in req_uids {
            let req_level = req_skills.get(uid);

            if req_level == 0 {
                continue;
            }

            let skills_point = &self.skills_point[uid];

            if skills_point.is_empty() {
                continue;
            }

            let deco = &self.get_deco_by_skill_id(uid)[0];
            let point = skills_point[req_level as usize - 1];

            let slot_index = deco.slot_size as usize - 1;

            points[slot_index] += req_level as i32 * point;
        }

        CalcVector::convert_to_lp_points_mut(&mut points);

        points
    }

    pub fn calc_slot_point(&self, slots: &SlotsVec) -> i32 {
        let mut points = 0;

        for &count in slots {
            points += self.point_lcm * count as i32;
        }

        points
    }

    pub fn calc_slot_point_lp(&self, slots: &SlotsVec) -> i32 {
        self.point_lcm * slots[0] as i32
    }

    pub fn calc_slot_point_slots_lp(&self, slots_lp: &SlotsVec) -> PointsVec {
        let mut points = PointsVec::default();

        for (index, &value) in slots_lp.iter().enumerate() {
            points[index] = value as i32 * self.point_lcm;
        }

        points
    }

    pub fn calc_slot_point_slots_lp_mut(&self, slots_lp: &SlotsVec, result: &mut PointsVec) {
        for (index, &value) in slots_lp.iter().enumerate() {
            result[index] += value as i32 * self.point_lcm;
        }
    }

    pub fn get_skills_from_decos(&self, comb: &IntMap<usize, SlotsVec>) -> SkillsContainer {
        let mut ret = SkillsContainer::new();

        for (&uid, &slots) in comb.iter() {
            let decos = self.get_deco_by_skill_id(uid);

            for deco in decos {
                let slot_index = deco.slot_size as usize - 1;

                ret.add_level(uid, deco.skill_level * slots[slot_index]);
            }
        }

        ret
    }

    pub fn is_valid_sextype(armor_sex_type: &SexType, sex_type: &SexType) -> bool {
        armor_sex_type == &SexType::All || armor_sex_type == sex_type
    }
}
