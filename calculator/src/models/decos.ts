import { FinalDecoInfo } from "../definition/deco_define";

import DecosVec from "../data/deco.json";

export class DecosDataManager {
    private decosBySkill: { [key: string]: FinalDecoInfo[] };

    constructor(private decos: FinalDecoInfo[]) {
        this.decosBySkill = {};

        for (const deco of decos) {
            let id = deco.skillId;

            if (this.decosBySkill[id] === undefined) {
                this.decosBySkill[id] = [];
            }

            // TODO: steadiness case - resolve case with same slot size and different level decoration case
            const removeIndices = [];
            let isInferior = false;

            for (let i = this.decosBySkill[id].length - 1; 0 <= i; --i) {
                const prevDeco = this.decosBySkill[id][i];

                if (prevDeco.slotSize == deco.slotSize) {
                    console.log(`Same slot size case: ${id} - Slot size ${deco.slotSize}, Lv${deco.skillLevel} Lv${prevDeco.skillLevel}`);

                    if (prevDeco.skillLevel < deco.skillLevel) {
                        removeIndices.push(i);
                    } else {
                        isInferior = true;
                    }
                }
            }

            for (const removeIndex of removeIndices) {
                this.decosBySkill[id].splice(removeIndex, 1);
            }

            if (isInferior === false) {
                this.decosBySkill[id].push(deco);
            }
        }

        for (const id in this.decosBySkill) {
            this.decosBySkill[id].sort((deco1, deco2) =>
                deco1.slotSize > deco2.slotSize ? 1 : -1
            );
        }

        console.log(this.decosBySkill);
    }

    getInfo(id: string, index: number) {
        return this.decosBySkill[id][index];
    }

    getAllDecos() {
        return this.decos;
    }

    getAllDecosBySkill() {
        return this.decosBySkill;
    }
}

export const DecosData = new DecosDataManager(DecosVec as FinalDecoInfo[]);
