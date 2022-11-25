import { ArmorFinalSkillInfo, ArmorStatInfo, FinalArmorInfo } from "./armor_define";

export const MAX_EQUIP_SLOTS = 3;
export const MAX_SLOT_LEVEL = 4;

export type Skills = { [key: string]: number };
export type Slots = [number, number, number, number];
export type EquipSlots = [number, number, number];
export type SexType = "" | "male" | "female";

export interface CalculateResult {
    full_equipments: ResultFullEquipments[];
    calc_time: number;
}

export interface ResultFullEquipments {
    sex_type: SexType;
    weapon_slots: EquipSlots;
    armors: { [key: string]: ResultArmor };
    talisman: ResultTalisman;
    deco_combs: ResultDecorationCombination[];
}

export interface ResultArmor {
    base_id: string;
    is_anomaly: boolean;

    skills: Skills;
    base_skills: Skills;
    diff_skills: Skills;

    slots: Slots;
    base_slots: Slots;
    diff_slots: Slots;
}

export interface ResultTalisman {
    skills: Skills;
    slots: Slots;
}

export interface ResultDecorationCombination {
    skill_decos: { [key: string]: number[] };
    slots_sum: Slots;
    leftover_slots_sum: Slots;
    leftover_skills: { [key: string]: number };
}

export interface SearchFavorite {
    name: string;
    sexType: SexType;
    weaponSlots: EquipSlots;
    reqSkills: Skills;
    reqSlots: Slots;
}

export interface ResultFavorite {
    name: string;
    sexType: SexType;
    weaponSlots: EquipSlots;
    armors: { [key: string]: ResultArmor };
    talisman: ResultTalisman;
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
    includeLteEquips: boolean;
}

export interface TalismanInfo {
    id: string;
    skills: { id: string, level: number }[],
    slotSizes: number[]
}