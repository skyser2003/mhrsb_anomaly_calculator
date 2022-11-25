use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::calc::types::SkillSlotCount;

use super::Language;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Skill {
    pub id: String,
    pub max_level: SkillSlotCount,
    pub names: HashMap<Language, String>,
}
