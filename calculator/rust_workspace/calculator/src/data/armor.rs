use std::{
    cmp::{Ordering, Reverse},
    collections::HashMap,
};

use lazy_static::lazy_static;
use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::calc::{constant::MAX_SLOT_LEVEL, types::SkillSlotCount};

use super::Language;

pub static EMPTY_ARMOR_PREFIX: &str = "__empty";
pub static SLOT_ARMOR_PREFIX: &str = "__slot";
pub static ANOMALY_ARMOR_PREFIX: &str = "__anomaly";

lazy_static! {
    pub static ref SLOT_ARMOR_REGEX: Regex =
        Regex::new(&format!("{}_{}", SLOT_ARMOR_PREFIX, r"(\d+)-(\d+)-(\d+)")).unwrap();
}

#[derive(Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum ArmorPart {
    Helm,
    Torso,
    Arm,
    Waist,
    Feet,
    Talisman, // TODO Separate json ArmorPart data from calc ArmorPart data
}

impl ArmorPart {
    pub fn as_str(&self) -> &'static str {
        match self {
            ArmorPart::Helm => "helm",
            ArmorPart::Torso => "torso",
            ArmorPart::Arm => "arm",
            ArmorPart::Waist => "waist",
            ArmorPart::Feet => "feet",
            ArmorPart::Talisman => "talisman",
        }
    }

    pub fn as_usize(&self) -> usize {
        match self {
            ArmorPart::Helm => 0,
            ArmorPart::Torso => 1,
            ArmorPart::Arm => 2,
            ArmorPart::Waist => 3,
            ArmorPart::Feet => 4,
            ArmorPart::Talisman => 5,
        }
    }

    pub fn from_usize(part: usize) -> ArmorPart {
        match part {
            0 => ArmorPart::Helm,
            1 => ArmorPart::Torso,
            2 => ArmorPart::Arm,
            3 => ArmorPart::Waist,
            4 => ArmorPart::Feet,
            5 => ArmorPart::Talisman,
            _ => panic!("Invalid ArmorPart uid: {}", part),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum SexType {
    All,
    Male,
    Female,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ArmorStat {
    pub defense: i16,
    pub fire_res: i16,
    pub water_res: i16,
    pub ice_res: i16,
    pub elec_res: i16,
    pub dragon_res: i16,
}

impl ArmorStat {
    pub fn new_empty() -> Self {
        Self {
            defense: 0,
            fire_res: 0,
            water_res: 0,
            ice_res: 0,
            elec_res: 0,
            dragon_res: 0,
        }
    }

    fn less(&self, other: &Self) -> bool {
        self.defense < other.defense
            && self.fire_res < other.fire_res
            && self.water_res < other.water_res
            && self.ice_res < other.ice_res
            && self.elec_res < other.elec_res
            && self.dragon_res < other.dragon_res
    }

    fn less_or_equal(&self, other: &Self) -> bool {
        self.defense <= other.defense
            && self.fire_res <= other.fire_res
            && self.water_res <= other.water_res
            && self.ice_res <= other.ice_res
            && self.elec_res <= other.elec_res
            && self.dragon_res <= other.dragon_res
    }
}

impl PartialOrd for ArmorStat {
    fn ge(&self, other: &Self) -> bool {
        other.less_or_equal(self)
    }

    fn gt(&self, other: &Self) -> bool {
        other.less(self)
    }

    fn le(&self, other: &Self) -> bool {
        self.less_or_equal(other)
    }

    fn lt(&self, other: &Self) -> bool {
        self.less(other)
    }

    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        if self == other {
            Some(Ordering::Equal)
        } else if self.less(other) {
            Some(Ordering::Less)
        } else if other.less(self) {
            Some(Ordering::Greater)
        } else {
            None
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ArmorSkill {
    pub level: SkillSlotCount,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BaseArmor {
    pub(super) id: String,
    pub part: ArmorPart,

    pub sex_type: SexType,

    pub names: HashMap<Language, String>,
    pub rarity: SkillSlotCount,
    pub stat: ArmorStat,
    pub skills: HashMap<String, ArmorSkill>,
    pub slots: Vec<SkillSlotCount>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AnomalyArmor {
    pub original: BaseArmor,
    pub affected: BaseArmor,

    pub stat_diff: ArmorStat,

    pub slot_diffs: Vec<SkillSlotCount>,

    pub skill_diffs: HashMap<String, ArmorSkill>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillIdLevel {
    pub id: String,
    pub level: SkillSlotCount,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Talisman {
    id: String,
    pub skills: Vec<SkillIdLevel>,
    pub slot_sizes: Vec<SkillSlotCount>,
}

impl ArmorPart {
    pub fn get_all_armor() -> Vec<Self> {
        vec![Self::Helm, Self::Torso, Self::Arm, Self::Waist, Self::Feet]
    }

    pub fn get_all_equip() -> Vec<Self> {
        vec![
            Self::Helm,
            Self::Torso,
            Self::Arm,
            Self::Waist,
            Self::Feet,
            Self::Talisman,
        ]
    }
}

impl BaseArmor {
    pub fn create_empty(part: ArmorPart) -> BaseArmor {
        Self {
            id: format!("{}-{}", EMPTY_ARMOR_PREFIX, part.as_str()),
            names: HashMap::new(),
            part,
            rarity: 10,
            sex_type: SexType::All,
            skills: HashMap::new(),
            slots: vec![0, 0, 0],
            stat: ArmorStat::new_empty(),
        }
    }

    pub fn id(&self) -> &String {
        &self.id
    }

    pub fn get_slot_armor_id(slots: &[SkillSlotCount]) -> String {
        format!(
            "{}_{}-{}-{}",
            SLOT_ARMOR_PREFIX, slots[0], slots[1], slots[2]
        )
    }

    pub fn parse_slot_armor_id(slot_armor_id: &str) -> Vec<SkillSlotCount> {
        let mut ret = Vec::new();

        for cap in SLOT_ARMOR_REGEX.captures_iter(slot_armor_id) {
            ret.push(
                cap.get(1)
                    .unwrap()
                    .as_str()
                    .parse::<SkillSlotCount>()
                    .unwrap(),
            );
            ret.push(
                cap.get(2)
                    .unwrap()
                    .as_str()
                    .parse::<SkillSlotCount>()
                    .unwrap(),
            );
            ret.push(
                cap.get(3)
                    .unwrap()
                    .as_str()
                    .parse::<SkillSlotCount>()
                    .unwrap(),
            );
        }

        ret
    }

    pub fn is_slot_armor(armor_id: &str) -> bool {
        armor_id.starts_with(SLOT_ARMOR_PREFIX)
    }

    pub fn is_anomaly_armor(armor_id: &str) -> bool {
        armor_id.starts_with(ANOMALY_ARMOR_PREFIX)
    }

    pub fn get_slot_armor(part: ArmorPart, slot_armor_id: String) -> BaseArmor {
        Self {
            id: slot_armor_id.clone(),
            names: HashMap::new(),
            part,
            rarity: 10,
            sex_type: SexType::All,
            skills: HashMap::new(),
            slots: BaseArmor::parse_slot_armor_id(&slot_armor_id),
            stat: ArmorStat::new_empty(),
        }
    }
}

impl AnomalyArmor {
    pub fn new(
        original: &BaseArmor,
        stat_diff: ArmorStat,
        mut slot_diffs: Vec<SkillSlotCount>,
        skill_diffs: HashMap<String, ArmorSkill>,
    ) -> AnomalyArmor {
        let mut affected = original.clone();

        affected.stat.defense += stat_diff.defense;
        affected.stat.fire_res += stat_diff.fire_res;
        affected.stat.water_res += stat_diff.water_res;
        affected.stat.ice_res += stat_diff.ice_res;
        affected.stat.elec_res += stat_diff.elec_res;
        affected.stat.dragon_res += stat_diff.dragon_res;

        let mut real_skill_diffs = HashMap::new();

        for (id, skill_info) in &skill_diffs {
            if skill_info.level == 0 {
                continue;
            }

            if skill_info.level < 0 && !original.skills.contains_key(id) {
                continue;
            }

            let diff_level = skill_info.level;

            let existing_skill = affected.skills.get(id);
            let base_level = match existing_skill {
                Some(existing_level) => existing_level.level,
                None => 0,
            };

            let new_value = (diff_level + base_level).max(0);

            let real_diff_level = new_value - base_level;

            real_skill_diffs.insert(
                id.clone(),
                ArmorSkill {
                    level: real_diff_level,
                },
            );

            if new_value == 0 {
                affected.skills.remove(id);
            } else {
                affected
                    .skills
                    .insert(id.clone(), ArmorSkill { level: new_value });
            }
        }

        for (index, slot_diff) in slot_diffs.iter().enumerate() {
            affected.slots[index] += slot_diff;
            affected.slots[index] = affected.slots[index].min(MAX_SLOT_LEVEL as i8);
        }

        affected.slots.sort_by_key(|&slot_size| Reverse(slot_size));

        for (index, (aff_slot, ori_slot)) in affected.slots.iter().zip(&original.slots).enumerate()
        {
            slot_diffs[index] = aff_slot - ori_slot;
        }

        AnomalyArmor {
            original: original.clone(),
            affected,
            stat_diff,
            slot_diffs,
            skill_diffs: real_skill_diffs,
        }
    }
}

impl Talisman {
    pub fn new(id: String, skills: Vec<SkillIdLevel>, slot_sizes: Vec<SkillSlotCount>) -> Self {
        Self {
            id,
            skills,
            slot_sizes,
        }
    }

    pub fn create_empty() -> Self {
        Self {
            id: format!("{}-talisman", EMPTY_ARMOR_PREFIX),
            ..Default::default()
        }
    }

    pub fn get_slot_talisman(slot_tali_id: String) -> Self {
        Self {
            id: slot_tali_id.clone(),
            slot_sizes: BaseArmor::parse_slot_armor_id(&slot_tali_id),
            skills: Default::default(),
        }
    }

    pub fn id(&self) -> &String {
        &self.id
    }

    pub fn set_id(&mut self, id: &str) {
        self.id = id.to_string();
    }
}
