use std::sync::Arc;

use nalgebra::SVector;

use super::{
    calc_equipment::CalcEquipment,
    constant::{EQUIP_PART_COUNT, MAX_SLOT_LEVEL, SKILLS_COUNT},
};

pub type SkillSlotCount = i8;

pub type EquipmentContainer = Vec<Vec<Arc<CalcEquipment>>>;

pub type SkillLevelVec = SVector<SkillSlotCount, SKILLS_COUNT>;
pub type SkillsTuple = Vec<(usize, SkillSlotCount)>;
pub type SlotsVec = SVector<SkillSlotCount, MAX_SLOT_LEVEL>;
pub type PointsVec = SVector<i32, MAX_SLOT_LEVEL>;

pub type EquipmentsArray<'a> = [&'a Arc<CalcEquipment>; EQUIP_PART_COUNT];
