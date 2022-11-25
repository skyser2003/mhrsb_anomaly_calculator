use super::types::{CalcInternalVec, PointsVec, SlotsVec};

#[derive(Clone)]
pub struct CalcVector {
    vec: CalcInternalVec,
}

impl CalcVector {
    pub fn convert_to_lp_slots(slots: &SlotsVec) -> SlotsVec {
        let mut ret = *slots;

        Self::convert_to_lp_slots_mut(&mut ret);

        ret
    }
    pub fn convert_to_lp_points(slots: &PointsVec) -> PointsVec {
        let mut ret = *slots;

        Self::convert_to_lp_points_mut(&mut ret);

        ret
    }

    pub fn convert_from_lp_slots(slots: &SlotsVec) -> SlotsVec {
        let mut ret = *slots;

        Self::convert_from_lp_slots_mut(&mut ret);

        ret
    }

    pub fn convert_to_lp_slots_mut(slots: &mut SlotsVec) {
        for index in (1..slots.len()).rev() {
            slots[index - 1] += slots[index];
        }
    }

    pub fn convert_to_lp_points_mut(slots: &mut PointsVec) {
        for index in (1..slots.len()).rev() {
            slots[index - 1] += slots[index];
        }
    }

    pub fn convert_from_lp_slots_mut(slots: &mut SlotsVec) {
        for index in 0..slots.len() - 1 {
            slots[index] -= slots[index + 1];
        }
    }

    pub fn promote_subtracted_lp_slots(slots: &mut SlotsVec) {
        let mut promote = 0;

        for index in 0..slots.len() - 1 {
            slots[index] += promote;
            let real_slot = slots[index] - slots[index + 1];

            if real_slot < 0 {
                promote = real_slot;
            } else {
                promote = 0;
            }
        }

        let len = slots.len();

        slots[len - 1] += promote;
    }

    pub fn promote_subtracted_lp_slots_to_slots(slots: &mut SlotsVec) {
        let mut promote = 0;

        for index in 0..slots.len() - 1 {
            let real_slot = slots[index] + promote - slots[index + 1];

            if real_slot < 0 {
                promote = real_slot;
                slots[index] = 0;
            } else {
                promote = 0;
                slots[index] = real_slot;
            }
        }

        let len = slots.len();

        slots[len - 1] += promote;
    }
}
