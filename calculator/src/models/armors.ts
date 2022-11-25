import { FinalArmorInfo } from "../definition/armor_define";

import RawArmorsVec from "../data/armor.json";

const ArmorsVec = RawArmorsVec as unknown as FinalArmorInfo[];

const armorParts = {
    "helm": true,
    "torso": true,
    "arm": true,
    "waist": true,
    "feet": true,
} as { [key: string]: boolean };

class ArmorsDataManager {
    private armorsMap: { [key: string]: FinalArmorInfo };

    constructor(private armorsVec: FinalArmorInfo[]) {
        this.armorsMap = {};

        for (const armor of ArmorsVec) {
            this.armorsMap[armor.id] = armor;
        }
    }

    getArmor(id: string) {
        return this.armorsMap[id];
    }

    getName(id: string, lang: string) {
        return this.armorsMap[id].names[lang];
    }

    isArmorPart(value: string) {
        return armorParts[value] !== undefined;
    }
}

export const ArmorsData = new ArmorsDataManager(ArmorsVec);
