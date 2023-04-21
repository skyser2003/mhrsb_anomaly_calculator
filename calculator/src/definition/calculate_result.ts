import { ArmorFinalSkillInfo, ArmorStatInfo, FinalArmorInfo, getDefaultStat } from "./armor_define";

export const MAX_EQUIP_SLOTS = 3;
export const MAX_SLOT_LEVEL = 4;

export type Skills = { [key: string]: number };
export type Slots = [number, number, number, number];
export type EquipSlots = [number, number, number];
export type SexType = "" | "male" | "female";

export interface CalculateResult {
    fullEquipments: ResultFullEquipments[];
    calcTime: number;
}

export interface FullEquipmentsCommon {
    sexType: SexType;
    weaponSlots: EquipSlots;
    armors: { [key: string]: ResultArmor };
    talisman: ResultTalisman;
}
export interface ResultFullEquipments extends FullEquipmentsCommon {
    decoCombs: ResultDecorationCombination[];
    commonLeftoverSkills: Skills;
}

export interface ResultArmor {
    baseId: string;
    isAnomaly: boolean;

    skills: Skills;
    baseSkills: Skills;
    diffSkills: Skills;

    slots: Slots;
    baseSlots: Slots;
    diffSlots: Slots;

    stat: ArmorStatInfo;
}

export interface ResultTalisman {
    skills: Skills;
    slots: Slots;
}

export interface ResultDecorationCombination {
    skillDecos: { [key: string]: number[] };
    slotsSum: Slots;
    leftoverSlotsSum: Slots;
    leftoverSkills: Skills;
}

export interface SearchFavorite {
    name: string;
    sexType: SexType;
    weaponSlots: EquipSlots;
    reqSkills: Skills;
    reqSlots: Slots;
}

export interface ResultFavorite extends FullEquipmentsCommon {
    name: string;
    decoComb: ResultDecorationCombination;
}

export interface AnomalyArmorInfo {
    original: FinalArmorInfo,
    affected: FinalArmorInfo;
    statDiff: ArmorStatInfo,
    slotDiffs: number[],
    skillDiffs: { [key: string]: ArmorFinalSkillInfo },
}

export interface CalcChoices {
    sexType: SexType;
    weaponSlots: EquipSlots;
    selectedSkills: Skills;
    freeSlots: Slots;
}

export interface TalismanInfo {
    id: string;
    skills: { id: string, level: number }[],
    slotSizes: number[]
}

export function getTotalStat(armors: { [key: string]: ResultArmor }) {
    const stat = getDefaultStat();

    if (armors !== undefined) {
        for (const equipId in armors) {
            const equip = armors[equipId];
            const equipStat = equip.stat;

            if (equipStat === undefined) {
                continue;
            }

            stat.defense += equipStat.defense;
            stat.fireRes += equipStat.fireRes;
            stat.waterRes += equipStat.waterRes;
            stat.iceRes += equipStat.iceRes;
            stat.elecRes += equipStat.elecRes;
            stat.dragonRes += equipStat.dragonRes;
        }
    }

    return stat;
}