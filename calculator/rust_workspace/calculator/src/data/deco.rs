use std::collections::HashMap;

use serde::Deserialize;

use crate::calc::types::SkillSlotCount;

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Decoration {
    pub id: String,
    pub names: HashMap<String, String>,
    pub skill_id: String,
    pub skill_level: SkillSlotCount,
    pub slot_size: SkillSlotCount,
}
