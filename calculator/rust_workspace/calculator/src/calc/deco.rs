use crate::data::deco::Decoration;

use super::{calc_vector::CalcVector, skills::SkillsContainer, types::SlotsVec};

pub struct CalcDeco<'a> {
    pub base: &'a Decoration,
}

impl<'a> CalcDeco<'a> {
    pub fn new(base: &'a Decoration) -> Self {
        Self { base }
    }

    pub fn convert_to_slots(
        req_skills: &SkillsContainer,
        single_slot_decos: &Vec<(usize, &Decoration)>,
    ) -> SlotsVec {
        let mut slots = SlotsVec::default();

        for &(uid, deco) in single_slot_decos {
            let level = req_skills.get(uid);

            if level == 0 {
                continue;
            }

            let slot_size_index = deco.slot_size as usize - 1;
            slots[slot_size_index] += level;
        }

        slots
    }

    pub fn convert_to_slots_lp(
        req_skills: &SkillsContainer,
        single_slot_decos: &Vec<(usize, &Decoration)>,
    ) -> SlotsVec {
        let mut slots = Self::convert_to_slots(req_skills, single_slot_decos);

        CalcVector::convert_to_lp_slots_mut(&mut slots);

        slots
    }
}
