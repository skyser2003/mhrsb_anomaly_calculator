import { MAX_SLOT_LEVEL, Slots } from "../definition/calculate_result";

export class SlotsDataManager {
    static convertToBase(slots: Slots) {
        let ret = [] as number[];

        for (let i = MAX_SLOT_LEVEL - 1; 0 <= i; --i) {
            for (let j = 0; j < slots[i]; ++j) {
                ret.push(i + 1);
            }
        }

        for (let i = ret.length; i < 3; ++i) {
            ret.push(0);
        }

        if (3 < ret.length) {
            console.error(
                `Slots length is ${ret.length} (${slots}), this is invalid`
            );
        }

        return ret;
    }
}
