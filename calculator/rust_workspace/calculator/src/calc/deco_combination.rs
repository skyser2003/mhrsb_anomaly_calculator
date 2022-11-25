use std::{
    cell::RefCell,
    collections::HashMap,
    mem::swap,
    rc::Rc,
    sync::{Arc, RwLock},
};

use itertools::izip;

use log::debug;
use nohash_hasher::{BuildNoHashHasher, IntMap};

use crate::{
    calc::skills::SkillsContainer,
    data::{data_manager::DataManager, deco::Decoration, skill::Skill},
};

use super::{
    calc_vector::CalcVector,
    constant::{MAX_SLOT_LEVEL, SKILLS_COUNT},
    types::SlotsVec,
    types::{SkillSlotCount, SkillsTuple},
};

#[derive(Default)]
pub struct DecorationCombinations {
    pub combinations: Vec<Vec<Vec<SlotsVec>>>,
    pub combinations_lp: Vec<Vec<Vec<SlotsVec>>>,
    pub combinations_lp_mins: Vec<Vec<SlotsVec>>,
    pub caches: RwLock<HashMap<SkillsTuple, Arc<Vec<SlotsVec>>>>,
    pub sum_caches: RwLock<HashMap<SkillsTuple, Arc<SlotsVec>>>,
    pub full_caches: RwLock<HashMap<SkillsTuple, Arc<Vec<DecorationCombination>>>>,
    pub empty_full_cache: Arc<Vec<DecorationCombination>>,
}

#[derive(Clone, Debug)]
pub struct DecorationCombination {
    pub combs_per_skill_lp: IntMap<usize, SlotsVec>,
    pub sum_lp: SlotsVec,
}

impl DecorationCombinations {
    pub fn new(decos_by_skill: &[Vec<Decoration>], skills: &[Skill]) -> DecorationCombinations {
        let mut temp_combs = IntMap::default();

        for (uid, decos) in decos_by_skill.iter().enumerate() {
            if decos.is_empty() {
                continue;
            }

            let skill = &skills[uid];
            let max_level = skill.max_level;

            if decos.len() == 1 {
                let mut skill_combs = Vec::new();

                let deco_skill_level = decos[0].skill_level;

                for req_level in 1..max_level + 1 {
                    let mut minimum_deco_count = req_level / deco_skill_level + 1;

                    if req_level % deco_skill_level == 0 {
                        minimum_deco_count -= 1;
                    }

                    skill_combs.push(vec![vec![minimum_deco_count]]);
                }

                temp_combs.insert(uid, skill_combs);
            } else {
                temp_combs.insert(uid, vec![]);

                let mut max_deco_counts = Vec::new();
                let mut init_case = Vec::new();

                for deco in decos {
                    let mut max_required = max_level / deco.skill_level + 1;

                    if max_level % deco.skill_level == 0 {
                        max_required -= 1;
                    }

                    max_deco_counts.push(max_required);
                    init_case.push(0);
                }

                for req_level in 1..max_level + 1 {
                    let mut skill_temp_combs = Vec::new();
                    let mut skill_done_combs = Vec::new();

                    for (slot_size_index, max_deco_count) in max_deco_counts.iter().enumerate() {
                        let deco = &decos[slot_size_index];

                        skill_temp_combs.push(init_case.clone());
                        let deco_temp_combs = skill_temp_combs.clone();

                        for temp_comb in &deco_temp_combs {
                            for count in (1..max_deco_count + 1).rev() {
                                let mut cur_level_sum: SkillSlotCount = temp_comb
                                    .iter()
                                    .enumerate()
                                    .map(|(prev_level_index, count)| {
                                        (prev_level_index + 1) as SkillSlotCount * count
                                    })
                                    .sum();

                                cur_level_sum += count * deco.skill_level;

                                let mut next_temp_comb = temp_comb.clone();
                                next_temp_comb[slot_size_index] = count;

                                if req_level <= cur_level_sum {
                                    let mut has_better_slot_answer = false;

                                    for lower_deco in decos.iter().take(slot_size_index) {
                                        let mut lower_level_sum = temp_comb.iter().sum();
                                        lower_level_sum += count * lower_deco.skill_level;

                                        if req_level <= lower_level_sum {
                                            has_better_slot_answer = true;
                                            break;
                                        }
                                    }

                                    if !has_better_slot_answer {
                                        skill_done_combs.push(next_temp_comb);
                                    }
                                } else {
                                    skill_temp_combs.push(next_temp_comb);
                                }
                            }
                        }
                    }

                    temp_combs.get_mut(&uid).unwrap().push(skill_done_combs);
                }
            }
        }

        for (_, combs) in temp_combs.iter_mut() {
            for deco_size_combs in combs {
                let mut remove_comb_indices = Vec::new();

                'remove_loop: for index1 in 0..deco_size_combs.len() - 1 {
                    //  TODO: Bug?
                    let deco_comb1 = &deco_size_combs[index1];

                    for deco_comb2 in deco_size_combs.iter().skip(index1 + 1) {
                        let mut is_inferior = true;

                        for (count1, count2) in izip!(deco_comb1, deco_comb2) {
                            if count1 < count2 {
                                is_inferior = false;
                                break;
                            }
                        }

                        if is_inferior {
                            remove_comb_indices.push(index1);
                            continue 'remove_loop;
                        }
                    }
                }

                for remove_index in remove_comb_indices.iter().rev() {
                    deco_size_combs.remove(*remove_index);
                }
            }
        }

        let existing_combinations = temp_combs
            .iter_mut()
            .map(|(&skill_uid, combs_per_skill)| {
                let mut ret = Vec::new();

                let decos = &decos_by_skill[skill_uid];

                for combs_per_level in combs_per_skill {
                    let mut converted_level_combs = Vec::new();

                    for comb in combs_per_level {
                        let mut converted = SlotsVec::default();

                        for (deco_index, slot_count) in comb.iter().enumerate() {
                            let deco = &decos[deco_index];
                            let slot_size = deco.slot_size;

                            converted[(slot_size - 1) as usize] = *slot_count;
                        }

                        converted_level_combs.push(converted);
                    }

                    ret.push(converted_level_combs);
                }

                let mut left_combs = Vec::new();

                for level_combs in ret {
                    let mut left_level_combs = Vec::new();

                    for (index1, comb1) in level_combs.iter().enumerate() {
                        let comb_lp1 = CalcVector::convert_to_lp_slots(&comb1);

                        let mut is_inferior = false;

                        for (index2, comb2) in level_combs.iter().enumerate() {
                            if index1 == index2 {
                                continue;
                            }

                            let comb_lp2 = CalcVector::convert_to_lp_slots(&comb2);

                            if DecorationCombination::is_possible_static_lp(&comb_lp1, &comb_lp2) {
                                is_inferior = true;
                                break;
                            }
                        }

                        if !is_inferior {
                            left_level_combs.push(comb_lp1);
                        }
                    }

                    for slot_size_index in (0..MAX_SLOT_LEVEL).rev() {
                        left_level_combs.sort_by_cached_key(|comb| comb[slot_size_index]);
                    }

                    left_combs.push(left_level_combs);
                }

                (skill_uid, left_combs)
            })
            .collect::<HashMap<usize, Vec<Vec<SlotsVec>>>>();

        let mut combinations_lp = Vec::with_capacity(SKILLS_COUNT);
        let mut combinations_lp_mins = Vec::with_capacity(SKILLS_COUNT);

        combinations_lp.resize(SKILLS_COUNT, Vec::new());
        combinations_lp_mins.resize(SKILLS_COUNT, Vec::new());

        for (uid, slot_combs) in existing_combinations.into_iter() {
            let mut min_vals = Vec::new();

            for level_combs in &slot_combs {
                let mut min_val =
                    SlotsVec::from_column_slice(&[SkillSlotCount::MAX; MAX_SLOT_LEVEL]);

                for comb in level_combs {
                    for (index, &val) in comb.iter().enumerate() {
                        min_val[index] = min_val[index].min(val);
                    }
                }

                min_vals.push(min_val);
            }

            combinations_lp[uid] = slot_combs;
            combinations_lp_mins[uid] = min_vals;
        }

        let combinations = combinations_lp
            .iter()
            .map(|slot_combs| {
                slot_combs
                    .iter()
                    .map(|combs| {
                        combs
                            .iter()
                            .map(|comb_lp| CalcVector::convert_from_lp_slots(comb_lp))
                            .collect()
                    })
                    .collect()
            })
            .collect::<Vec<_>>();

        Self {
            combinations,
            combinations_lp,
            combinations_lp_mins,
            caches: RwLock::new(HashMap::new()),
            sum_caches: RwLock::new(HashMap::new()),
            full_caches: RwLock::new(HashMap::new()),
            empty_full_cache: Arc::new(vec![DecorationCombination {
                sum_lp: SlotsVec::default(),
                combs_per_skill_lp: IntMap::default(),
            }]),
        }
    }

    pub fn debug(&self, dm: &DataManager) {
        for (uid, combs) in self.combinations.iter().enumerate() {
            for (level_index, level_combs) in combs.iter().enumerate() {
                let level = level_index + 1;

                debug!(
                    "{} - Lv{} - {:?}",
                    dm.get_skill(uid).id.clone(),
                    level,
                    level_combs
                );
            }
        }
    }

    pub fn get(&self, skill_uid: usize) -> &Vec<Vec<SlotsVec>> {
        &self.combinations[skill_uid]
    }

    pub fn get_possible_combs_lp(
        &self,
        req_skills: &SkillsContainer,
        avail_slots_lp: &SlotsVec,
    ) -> Vec<SlotsVec> {
        let all_combs_count = self.get_total_combs_count(req_skills);

        let mut all_possible_combs1 = Vec::with_capacity(all_combs_count);
        let mut all_possible_combs2 = Vec::with_capacity(all_combs_count);

        all_possible_combs1.resize(all_combs_count, SlotsVec::default());
        all_possible_combs2.resize(all_combs_count, SlotsVec::default());

        let mut all_combs1 = &mut all_possible_combs1;
        let mut all_combs2 = &mut all_possible_combs2;

        let mut combs1_len = 1;
        let mut combs2_len = 0;

        all_combs1[0].fill(0);

        for (uid, level) in req_skills.iter() {
            let combs = &self.combinations_lp[uid][(level - 1) as usize];

            'combs_loop: for combs1_index in 0..combs1_len {
                let mut prev_index = MAX_SLOT_LEVEL;
                let mut prev_val = SkillSlotCount::MAX;

                'existing_loop: for comb in combs {
                    if prev_index != MAX_SLOT_LEVEL && comb[prev_index] == prev_val {
                        continue;
                    }

                    let new_comb = all_combs2.get_mut(combs2_len).unwrap();

                    *new_comb = all_combs1[combs1_index];
                    *new_comb += comb;

                    if avail_slots_lp[0] < new_comb[0] {
                        continue 'combs_loop;
                    }

                    for index in 1..MAX_SLOT_LEVEL {
                        if avail_slots_lp[index] < new_comb[index] {
                            prev_index = index;
                            prev_val = comb[index - 1];

                            continue 'existing_loop;
                        }
                    }

                    combs2_len += 1;
                }
            }

            swap(&mut all_combs1, &mut all_combs2);
            swap(&mut combs1_len, &mut combs2_len);

            combs2_len = 0;
        }

        all_combs1.truncate(combs1_len);

        all_combs1.to_vec()
    }

    pub fn get_possible_deco_combs_sum(&self, req_skills: &SkillsContainer) -> SlotsVec {
        let mut mins = SlotsVec::default();

        for (uid, level) in req_skills.iter() {
            let min_val = &self.combinations_lp_mins[uid][(level - 1) as usize];

            mins += min_val;
        }

        mins
    }

    pub fn get_full_possible_combs(
        &self,
        req_skills: &SkillsContainer,
    ) -> Arc<Vec<DecorationCombination>> {
        if req_skills.is_empty() {
            return self.empty_full_cache.clone();
        }

        {
            let caches = self.full_caches.read().unwrap();
            let cache = caches.get(&req_skills.get_list());

            if let Some(cache) = cache {
                return cache.clone();
            }
        }

        let all_combs_count = self.get_total_combs_count(req_skills);
        let mut all_possible_combs = Vec::with_capacity(all_combs_count);

        let (req_list, mut level_indices) = self.get_iter_init_data(req_skills);
        let all_skill_combs = Rc::new(RefCell::new(HashMap::with_capacity_and_hasher(
            level_indices.len(),
            BuildNoHashHasher::default(),
        )));

        let mut all_combs_lp = Vec::new();

        for (uid, level) in req_skills.iter() {
            let combs = &self.combinations_lp[uid][(level - 1) as usize];

            all_combs_lp.push(combs);
        }

        loop {
            let deco_comb = self.get_next_deco_comb(
                &req_list,
                &all_combs_lp,
                &level_indices,
                all_skill_combs.clone(),
            );

            all_possible_combs.push(deco_comb);

            if self.proceed_next_iter(&all_combs_lp, &mut level_indices) == false {
                break;
            }
        }

        {
            let mut caches = self.full_caches.write().unwrap();
            let insert_value = Arc::new(all_possible_combs);
            caches.insert(req_skills.get_list(), insert_value.clone());

            insert_value
        }
    }

    pub fn has_possible_combs_lp(
        &self,
        req_skills: &SkillsContainer,
        armor_slots_lp: &SlotsVec,
    ) -> bool {
        self.iter_possible_combs_lp(req_skills, |sum_lp| {
            DecorationCombination::is_possible_static_lp(armor_slots_lp, sum_lp)
        })
    }

    pub fn iter_possible_combs_lp<F>(&self, req_skills: &SkillsContainer, mut f: F) -> bool
    where
        F: FnMut(&SlotsVec) -> bool,
    {
        let skills_count = req_skills.len();

        let mut level_indices = vec![0; skills_count];

        let mut all_combs_lp = Vec::with_capacity(skills_count);

        for (uid, level) in req_skills.iter() {
            let combs = &self.combinations_lp[uid][(level - 1) as usize];

            all_combs_lp.push(combs);
        }

        let mut sum_lp = SlotsVec::default();

        loop {
            self.get_next_deco_comb_lp(&all_combs_lp, &level_indices, &mut sum_lp);

            if f(&sum_lp) {
                return true;
            }

            if self.proceed_next_iter(&all_combs_lp, &mut level_indices) == false {
                break;
            }
        }

        false
    }

    fn get_iter_init_data<'a>(&self, req_skills: &'a SkillsContainer) -> (SkillsTuple, Vec<usize>) {
        let req_list = req_skills.get_list();
        let level_indices = vec![0; req_list.len()];

        (req_list, level_indices)
    }

    fn get_next_deco_comb(
        &self,
        req_list: &SkillsTuple,
        all_combs_lp: &Vec<&Vec<SlotsVec>>,
        level_indices: &[usize],
        all_skill_combs: Rc<RefCell<IntMap<usize, SlotsVec>>>,
    ) -> DecorationCombination {
        let mut slot_combs_lp = SlotsVec::default();

        let mut all_skill_combs = all_skill_combs.as_ref().borrow_mut();

        for (&inside_level_index, req_info, combs) in izip!(level_indices, req_list, all_combs_lp) {
            let skill_uid = req_info.0;

            let skill_comb = &combs[inside_level_index];

            all_skill_combs.insert(skill_uid, *skill_comb);

            slot_combs_lp += skill_comb;
        }

        DecorationCombination {
            combs_per_skill_lp: all_skill_combs.clone(),
            sum_lp: slot_combs_lp,
        }
    }

    fn get_next_deco_comb_lp(
        &self,
        all_combs_lp: &Vec<&Vec<SlotsVec>>,
        level_indices: &[usize],
        slot_combs_lp: &mut SlotsVec,
    ) {
        slot_combs_lp.fill(0);

        for (&inside_level_index, combs) in level_indices.iter().zip(all_combs_lp) {
            let skill_comb = &combs[inside_level_index];

            *slot_combs_lp += skill_comb;
        }
    }

    fn proceed_next_iter(
        &self,
        all_combs: &Vec<&Vec<SlotsVec>>,
        level_indices: &mut Vec<usize>,
    ) -> bool {
        let mut promote = 0;

        for (index, &combs) in all_combs.iter().enumerate() {
            level_indices[index] += 1;

            if level_indices[index] == combs.len() {
                level_indices[index] = 0;
                promote = 1;
            } else {
                promote = 0;
                break;
            }
        }

        promote == 0
    }

    fn get_total_combs_count(&self, req_skills: &SkillsContainer) -> usize {
        req_skills.iter().fold(1, |acum, (uid, level)| {
            acum * self.combinations[uid][(level - 1) as usize].len()
        })
    }

    pub fn compare(
        slots1: &Vec<SkillSlotCount>,
        slots2: &Vec<SkillSlotCount>,
    ) -> std::cmp::Ordering {
        for (slot1, slot2) in izip!(slots1, slots2) {
            if slot1 == slot2 {
                continue;
            }

            return slot1.cmp(slot2);
        }

        std::cmp::Ordering::Equal
    }
}

impl DecorationCombination {
    pub fn get_skill_decos(&self) -> IntMap<usize, SlotsVec> {
        let mut ret = IntMap::default();

        for (&uid, slots_lp) in self.combs_per_skill_lp.iter() {
            let slots = CalcVector::convert_from_lp_slots(slots_lp);

            ret.insert(uid, slots);
        }

        ret
    }

    pub fn is_possible_lp(&self, equip_slots_lp: &SlotsVec) -> bool {
        Self::is_possible_static_lp(equip_slots_lp, &self.sum_lp)
    }

    pub fn is_possible_static_lp(free_slots_lp: &SlotsVec, req_slots_lp: &SlotsVec) -> bool {
        for index in 0..MAX_SLOT_LEVEL {
            if free_slots_lp[index] < req_slots_lp[index] {
                return false;
            }
        }

        true
    }

    pub fn is_possible_static_lp_equip_mut(
        equip_slots_lp: &mut SlotsVec,
        all_req_slots_lp: &[&SlotsVec],
    ) -> bool {
        for req_slots in all_req_slots_lp {
            *equip_slots_lp -= *req_slots;
        }

        let result = equip_slots_lp.iter().all(|&count| count >= 0);

        if result {
            CalcVector::promote_subtracted_lp_slots(equip_slots_lp);
        }

        result
    }
}
