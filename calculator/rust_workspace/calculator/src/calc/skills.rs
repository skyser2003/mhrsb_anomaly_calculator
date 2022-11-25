use itertools::Itertools;

use crate::{calc::constant::SKILLS_COUNT, data::data_manager::DataManager};

use super::types::{SkillLevelVec, SkillSlotCount, SkillsTuple};

#[derive(Clone, Debug)]
pub struct SkillsContainer {
    vec: SkillLevelVec,
}

pub struct SkillsContainerIterator<'a> {
    index: usize,
    container: &'a SkillsContainer,
}

pub struct SkillsContainerIteratorMutable<'a> {
    index: usize,
    container: &'a mut SkillsContainer,
}

impl SkillsContainer {
    pub fn new() -> Self {
        let vec = SkillLevelVec::from_vec(vec![0; SKILLS_COUNT]);

        Self::from_vec(vec)
    }

    pub fn from(vec: Vec<SkillSlotCount>) -> Self {
        Self::from_vec(SkillLevelVec::from_vec(vec))
    }

    pub fn from_vec(vec: SkillLevelVec) -> Self {
        Self { vec }
    }

    pub fn iter(&self) -> SkillsContainerIterator {
        SkillsContainerIterator {
            index: 0,
            container: self,
        }
    }

    pub fn iter_mut(&mut self) -> SkillsContainerIteratorMutable {
        SkillsContainerIteratorMutable {
            index: 0,
            container: self,
        }
    }

    pub fn contains(&self, uid: usize) -> bool {
        0 < self.vec[uid]
    }

    pub fn get(&self, uid: usize) -> SkillSlotCount {
        self.vec[uid].max(0)
    }

    pub fn set(&mut self, uid: usize, value: SkillSlotCount) {
        self.vec[uid] = value;
    }

    pub fn add(&mut self, other: &SkillsContainer) {
        self.vec += other.vec;
    }

    pub fn sub(&mut self, other: &SkillsContainer) {
        self.vec -= other.vec;
    }

    pub fn sub_mut(&mut self, other: &mut SkillsContainer) {
        self.vec -= other.vec;
        other.vec = -self.vec;
    }

    pub fn get_diff(&self, other: &SkillsContainer) -> Self {
        Self {
            vec: self.vec - other.vec,
        }
    }

    pub fn add_level(&mut self, uid: usize, level: SkillSlotCount) {
        self.vec[uid] += level;
    }

    pub fn clear_zeros(&mut self) {
        for i in 0..SKILLS_COUNT {
            self.vec[i] = self.vec[i].max(0);
        }
    }

    pub fn get_indices(&self) -> Vec<usize> {
        let mut vec = Vec::new();

        for (uid, &level) in self.vec.iter().enumerate() {
            if 0 < level {
                vec.push(uid);
            }
        }

        vec
    }

    pub fn len(&self) -> usize {
        let mut length = 0;

        for _ in self.iter() {
            length += 1;
        }

        length
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn debug<'a>(&self, dm: &'a DataManager) -> Vec<(&'a String, SkillSlotCount)> {
        self.iter()
            .map(|(uid, level)| (&dm.get_skill(uid).id, level))
            .collect_vec()
    }

    pub fn len_static(vec: &SkillLevelVec) -> usize {
        let mut length = 0;

        for &value in vec {
            if 0 < value {
                length += 1
            }
        }

        length
    }

    pub fn get_list(&self) -> SkillsTuple {
        let mut ret = Vec::new();

        for (uid, level) in self.iter() {
            ret.push((uid, level));
        }

        ret
    }
}

impl<'a> Iterator for SkillsContainerIterator<'a> {
    type Item = (usize, SkillSlotCount);

    fn next(&mut self) -> Option<Self::Item> {
        let length = self.container.vec.len();

        while self.index != length {
            let ret_index = self.index;
            let val = self.container.vec[self.index];

            self.index += 1;

            if val <= 0 {
                continue;
            } else {
                return Some((ret_index, val));
            }
        }

        self.index = 0;

        None
    }
}

impl<'a> Iterator for SkillsContainerIteratorMutable<'a> {
    type Item = (usize, &'a mut SkillSlotCount);

    fn next(&mut self) -> Option<Self::Item> {
        let length = self.container.vec.len();

        let vec_mut = self.container.vec.as_mut_ptr();

        while self.index != length {
            let ret_index = self.index;

            unsafe {
                let val = vec_mut.add(ret_index);
                self.index += 1;

                if *val <= 0 {
                    continue;
                } else {
                    return Some((ret_index, &mut *val));
                }
            }
        }

        self.index = 0;

        None
    }
}

impl PartialEq for SkillsContainer {
    fn eq(&self, other: &Self) -> bool {
        self.vec == other.vec
    }
}

impl Default for SkillsContainer {
    fn default() -> Self {
        Self::new()
    }
}
