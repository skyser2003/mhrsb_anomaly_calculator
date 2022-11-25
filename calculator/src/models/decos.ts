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

            this.decosBySkill[id].push(deco);
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
}

export const DecosData = new DecosDataManager(DecosVec as FinalDecoInfo[]);
