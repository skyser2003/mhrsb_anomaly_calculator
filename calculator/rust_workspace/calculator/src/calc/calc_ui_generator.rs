#[derive(Default)]
pub struct CalcEquipmentUid {
    uid: usize,
}

impl CalcEquipmentUid {
    pub fn flush(&mut self) {
        self.uid = 0;
    }

    pub fn next_uid(&mut self) -> usize {
        let uid = self.uid;
        self.uid += 1;

        uid
    }
}
