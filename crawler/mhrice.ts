import path from "path";
import fs from "fs";

import { FinalSkillInfo } from "./definition/skill_define.js";
import { FinalDecoInfo } from "./definition/deco_define.js";
import {
    ArmorFinalSkillInfo,
    ArmorStatInfo,
    FinalArmorInfo,
} from "./definition/armor_define.js";

import { makeId } from "./util.js";

enum ContentLangIndex {
    ja = 0,
    en = 1,
    fr = 2,
    it = 3,
    de = 4,
    es = 5,
    ru = 6,
    pl = 7,
    pt = 10,
    ko = 11,
    zh_Hant = 12,
    zh_Hans = 13,
    ar = 21,
}

const LangCodeMap = new Map<ContentLangIndex, string>([
    [ContentLangIndex.ja, "ja"],
    [ContentLangIndex.en, "en"],
    [ContentLangIndex.fr, "fr"],
    [ContentLangIndex.it, "it"],
    [ContentLangIndex.de, "de"],
    [ContentLangIndex.es, "es"],
    [ContentLangIndex.ru, "ru"],
    [ContentLangIndex.pl, "pl"],
    [ContentLangIndex.pt, "pt"],
    [ContentLangIndex.ko, "ko"],
    [ContentLangIndex.zh_Hant, "zh-Hant"],
    [ContentLangIndex.zh_Hans, "zh-Hans"],
    [ContentLangIndex.ar, "ar"],
]);

interface Decoration {
    id: { Deco?: number; MrDeco?: number } | "None";
    sort_id: number;
    rare: number;
    icon_color: number;
    base_price: number;
    decoration_lv: number;
    skill_id_list: { Skill?: number; MrSkill?: number }[];
    skill_lv_list: number[];
}

interface EquipSkill {
    id: { Skill?: number; MrSkill?: number };
    max_level: number;
    icon_color: number;
    worth_point_list: number;
}

interface PlayerSkillNameMsg {
    name: string;
    guid: string;
    hash: number;
    attributes: any[];
    content: string[];
}

interface PlayerSkillDetailMsg {
    name: string;
    guid: string;
    hash: number;
    attiributes: any[];
    content: string[];
}

interface Armor {
    pl_armor_id: {
        Head?: number;
        Chest?: number;
        Arm?: number;
        Waist?: number;
        Leg?: number;
    };

    is_valid: boolean;
    series: number;
    sort_id: number;
    model_id: 30;
    rare: number;
    value: number;
    buy_value: number;
    sexual_equipable: string;
    symbol_color1: boolean;
    symbol_color2: boolean;
    def_val: number;
    fire_reg_val: number;
    water_reg_val: number;
    ice_reg_val: number;
    thunder_reg_val: number;
    dragon_reg_val: number;
    buildup_table: number;
    buff_formula: number;
    decorations_num_list: number[];
    skill_list: ({ Skill?: number; MrSkill?: number } | "None")[];
    skill_lv_list: number[];
    id_after_ex_change: string;
    custom_talbe_no: number;
    custom_cost: number;
}

export function parseMhrIce() {
    const filename = path.join("original_data", "mhrice.json");

    const jsonContents = JSON.parse(fs.readFileSync(filename).toString()) as {
        [key: string]: any;
    };

    const decos = jsonContents["decorations"]["param"] as Decoration[];
    const decoNames = jsonContents["decorations_name_msg"][
        "entries"
    ] as PlayerSkillDetailMsg[];
    const decoMrNames = jsonContents["decorations_name_msg_mr"][
        "entries"
    ] as PlayerSkillDetailMsg[];

    const skills = jsonContents["equip_skill"]["param"] as EquipSkill[];
    const skillNames = jsonContents["player_skill_name_msg"][
        "entries"
    ] as PlayerSkillNameMsg[];
    const skillMrNames = jsonContents["player_skill_name_msg_mr"][
        "entries"
    ] as PlayerSkillNameMsg[];
    const skillLvTexts = jsonContents["player_skill_detail_msg"][
        "entries"
    ] as PlayerSkillDetailMsg[];
    const skillLvMrTexts = jsonContents["player_skill_detail_msg_mr"][
        "entries"
    ] as PlayerSkillDetailMsg[];
    const skillExpTexts = jsonContents["player_skill_explain_msg"][
        "entries"
    ] as PlayerSkillDetailMsg[];
    const skillExpMrTexts = jsonContents["player_skill_explain_msg_mr"][
        "entries"
    ] as PlayerSkillDetailMsg[];

    const [skillInfos, hrSkillIds, mrSkillIds] = parseSkills(
        skillNames,
        skillMrNames,
        skillLvTexts,
        skillLvMrTexts,
        skillExpTexts,
        skillExpMrTexts
    );

    const decoInfos = parseDecos(
        decos,
        hrSkillIds,
        mrSkillIds,
        decoNames,
        decoMrNames
    );

    for (const key in jsonContents) {
        // console.log(key);
    }

    const armors = jsonContents["armor"]["param"] as Armor[];

    const headNames = jsonContents["armor_head_name_msg"][
        "entries"
    ] as PlayerSkillDetailMsg[];
    const chestNames = jsonContents["armor_chest_name_msg"][
        "entries"
    ] as PlayerSkillDetailMsg[];
    const armNames = jsonContents["armor_arm_name_msg"][
        "entries"
    ] as PlayerSkillDetailMsg[];
    const waistNames = jsonContents["armor_waist_name_msg"][
        "entries"
    ] as PlayerSkillDetailMsg[];
    const legNames = jsonContents["armor_leg_name_msg"][
        "entries"
    ] as PlayerSkillDetailMsg[];

    const headMrNames = jsonContents["armor_head_name_msg_mr"][
        "entries"
    ] as PlayerSkillDetailMsg[];
    const chestMrNames = jsonContents["armor_chest_name_msg_mr"][
        "entries"
    ] as PlayerSkillDetailMsg[];
    const armMrNames = jsonContents["armor_arm_name_msg_mr"][
        "entries"
    ] as PlayerSkillDetailMsg[];
    const waistMrNames = jsonContents["armor_waist_name_msg_mr"][
        "entries"
    ] as PlayerSkillDetailMsg[];
    const legMrNames = jsonContents["armor_leg_name_msg_mr"][
        "entries"
    ] as PlayerSkillDetailMsg[];

    const armorNames = {
        head: headNames.concat(headMrNames),
        chest: chestNames.concat(chestMrNames),
        arm: armNames.concat(armMrNames),
        waist: waistNames.concat(waistMrNames),
        leg: legNames.concat(legMrNames),
    };

    const armorInfos = parseArmor(armors, armorNames, hrSkillIds, mrSkillIds);

    fs.writeFileSync(
        path.join("data", "armor.json"),
        JSON.stringify(armorInfos, null, 4)
    );
    fs.writeFileSync(
        path.join("data", "skill.json"),
        JSON.stringify(skillInfos, null, 4)
    );
    fs.writeFileSync(
        path.join("data", "deco.json"),
        JSON.stringify(decoInfos, null, 4)
    );
}

function isInvalidContent(info: PlayerSkillDetailMsg) {
    return info.content[0].includes("#Rejected#") || info.content[0] === "";
}

const decoNameRegex = new RegExp(/^Decorations_(\d+)_Name$/);

function parseDecos(
    decos: Decoration[],
    hrSkillIds: { [key: number]: string },
    mrSkillIds: { [key: number]: string },
    decoNames: PlayerSkillDetailMsg[],
    decoMrNames: PlayerSkillDetailMsg[]
) {
    let decoIndex = 0;
    let mrDecoIndex = 0;

    const hrDecos = {} as {
        [key: number]: {
            id: number;
            isMrSkill: boolean;
            slotSize: number;
            skillId: number;
            skillLevel: number;
        };
    };

    const mrDecos = {} as typeof hrDecos;

    for (const deco of decos) {
        if (deco.id === "None") {
            continue;
        }

        let isMr = false;

        if (deco.id.Deco !== undefined) {
            ++decoIndex;
        } else if (deco.id.MrDeco !== undefined) {
            isMr = true;
            ++mrDecoIndex;
        }

        const decoId = deco.id.Deco ?? deco.id.MrDeco;

        if (decoId === undefined) {
            continue;
        }

        const skillIdInfo = deco.skill_id_list[0];
        const skillId = skillIdInfo.Skill ?? skillIdInfo.MrSkill;

        if (skillId === undefined) {
            continue;
        }

        const decoInfo = {
            id: decoId,
            isMrSkill: skillIdInfo.Skill === undefined ? true : false,
            slotSize: deco.decoration_lv,
            skillId: skillId,
            skillLevel: deco.skill_lv_list[0],
        };

        if (deco.id.MrDeco !== undefined) {
            mrDecos[decoId] = decoInfo;
        } else if (deco.id.Deco !== undefined) {
            hrDecos[decoId] = decoInfo;
        }

        // console.log(
        //     isMr,
        //     decoId,
        //     skillId,
        //     `${decoIndex} ${mrDecoIndex}`,
        //     hrSkillIds[skillId],
        //     mrSkillIds[skillId],
        //     deco.decoration_lv,
        //     deco.skill_lv_list[0]
        // );
    }

    const ret = [] as FinalDecoInfo[];

    const getFinalResult = (realNames: typeof decoNames) => {
        for (const info of realNames) {
            const match = info.name.match(decoNameRegex);
            if (match === null) {
                continue;
            }

            if (isInvalidContent(info)) {
                continue;
            }

            let decoIndex = parseInt(match[1]);
            let isMr = false;

            if (200 <= decoIndex) {
                decoIndex -= 200;
                isMr = true;
            }

            let decoInfo;

            if (isMr) {
                decoInfo = mrDecos[decoIndex];
            } else {
                decoInfo = hrDecos[decoIndex];
            }

            if (decoInfo === undefined) {
                console.log(isMr, decoIndex, info.content[ContentLangIndex.ko]);
                continue;
            }

            // console.log(isMr, decoIndex, skillIds[decoInfo.skillId], decoInfo);

            let resultId = makeId(info.content[ContentLangIndex.en]);

            const names = {} as { [key: string]: string };

            for (const [index, langCode] of LangCodeMap) {
                names[langCode] = info.content[index];
            }

            let skillId;

            if (decoInfo.isMrSkill === true) {
                skillId = mrSkillIds[decoInfo.skillId];
            } else {
                skillId = hrSkillIds[decoInfo.skillId];
            }

            let result = {
                id: resultId,
                names,
                skillId,
                skillLevel: decoInfo.skillLevel,
                slotSize: decoInfo.slotSize,
            } as FinalDecoInfo;

            ret.push(result);
        }
    };

    getFinalResult(decoNames);
    getFinalResult(decoMrNames);

    return ret;
}

const skillIdRegex = new RegExp(/^PlayerSkill_(\d+)_Name$/);
const skillLevelRegex = new RegExp(/^PlayerSkill_(\d+)_(\d+)_Detail.*/);
const skillExpRegex = new RegExp(/^PlayerSkill_(\d+)_Explain.*/);

function parseSkills(
    skillNames: PlayerSkillNameMsg[],
    skillMrNames: PlayerSkillNameMsg[],
    skillLvTexts: PlayerSkillDetailMsg[],
    skillLvMrTexts: PlayerSkillDetailMsg[],
    skillExpTexts: PlayerSkillDetailMsg[],
    skillExpMrTexts: PlayerSkillDetailMsg[]
) {
    const hr_names = {} as { [key: string]: string[] };
    const hr_texts = {} as { [key: string]: string[] };

    const mr_names = {} as typeof hr_names;
    const mr_texts = {} as typeof hr_texts;

    const max_levels = {} as { [key: string]: number };

    const getNames = (
        realNames: typeof hr_names,
        rawNames: typeof skillNames
    ) => {
        for (const info of rawNames) {
            const match = info.name.match(skillIdRegex);
            if (match === null) {
                continue;
            }

            if (info.content[0] === "") {
                continue;
            }

            const id = match[1];

            realNames[id] = info.content;
        }
    };

    const getLevels = (
        realLevels: typeof max_levels,
        rawLevels: typeof skillLvTexts
    ) => {
        for (const info of rawLevels) {
            const match = info.name.match(skillLevelRegex);
            if (match === null) {
                continue;
            }

            if (isInvalidContent(info)) {
                continue;
            }

            const id = match[1];

            if (realLevels[id] === undefined) {
                realLevels[id] = 0;
            }

            const level = parseInt(match[2]) + 1;

            realLevels[id] = Math.max(realLevels[id], level);
        }
    };

    const getTexts = (
        realTexts: typeof hr_texts,
        rawTexts: typeof skillExpTexts
    ) => {
        for (const info of rawTexts) {
            const match = info.name.match(skillExpRegex);
            if (match === null) {
                continue;
            }

            if (isInvalidContent(info)) {
                continue;
            }

            const id = match[1];
            realTexts[id] = info.content;
        }
    };

    getNames(hr_names, skillNames);
    getNames(mr_names, skillMrNames);

    getLevels(max_levels, skillLvTexts);
    getLevels(max_levels, skillLvMrTexts);

    getTexts(hr_texts, skillExpTexts);
    getTexts(mr_texts, skillExpMrTexts);

    let ret1 = [] as FinalSkillInfo[];
    let hr_ids = {} as { [key: string]: string };
    let mr_ids = {} as typeof hr_ids;

    const getFinalResult = (
        realNames: typeof hr_names,
        realLevels: typeof max_levels,
        realtexts: typeof hr_texts,
        realIds: typeof hr_ids
    ) => {
        for (let i = 0; i < Object.keys(realNames).length; ++i) {
            const key = Object.keys(realNames)[i];

            const names_vec = realNames[key];
            const maxLevel = realLevels[key];
            const texts_vec = realtexts[key];

            const names = {} as { [key: string]: string };
            const texts = {} as { [key: string]: string };

            names_vec.forEach((name, index) => {
                const langCode = LangCodeMap.get(index as ContentLangIndex);

                if (langCode === undefined) {
                    return;
                }

                names[langCode] = name;
                texts[langCode] = texts_vec[index];
            });

            let id = makeId(names["en"]);

            let elem = {
                id,
                maxLevel,
                names,
                texts,
            } as FinalSkillInfo;

            let skillKey = parseInt(key);

            if (200 <= skillKey) {
                skillKey -= 200;
            }

            ret1.push(elem);
            realIds[skillKey] = id;
        }
    };

    getFinalResult(hr_names, max_levels, hr_texts, hr_ids);
    getFinalResult(mr_names, max_levels, mr_texts, mr_ids);

    return [ret1, hr_ids, mr_ids] as [
        FinalSkillInfo[],
        { [key: string]: string },
        { [key: string]: string }
    ];
}

const armorIdRegex = new RegExp(/A_(.+)_(\d+)_Name/);

function parseArmor(
    armors: Armor[],
    armorNames: { [key: string]: PlayerSkillDetailMsg[] },
    hrSkillIds: { [key: number]: string },
    mrSkillIds: { [key: number]: string }
) {
    const names = {} as { [key: string]: { [key: number]: string[] } };

    for (const armorPart in armorNames) {
        const partNames = armorNames[armorPart];
        names[armorPart] = {};

        for (const info of partNames) {
            const match = info.name.match(armorIdRegex);
            if (match === null) {
                continue;
            }

            if (isInvalidContent(info)) {
                continue;
            }

            const part = match[1].toLowerCase();
            const id = parseInt(match[2]);

            names[part][id] = info.content;
        }
    }

    const armorIdSet = new Set();

    const ret = [];

    for (const armor of armors) {
        let part = "";
        let id = -1;

        if (armor.pl_armor_id.Head !== undefined) {
            part = "head";
            id = armor.pl_armor_id.Head;
        } else if (armor.pl_armor_id.Chest !== undefined) {
            part = "chest";
            id = armor.pl_armor_id.Chest;
        } else if (armor.pl_armor_id.Arm !== undefined) {
            part = "arm";
            id = armor.pl_armor_id.Arm;
        } else if (armor.pl_armor_id.Waist !== undefined) {
            part = "waist";
            id = armor.pl_armor_id.Waist;
        } else if (armor.pl_armor_id.Leg !== undefined) {
            part = "leg";
            id = armor.pl_armor_id.Leg;
        }

        if (part === "") {
            continue;
        }

        const langNames = names[part][id];

        if (langNames === undefined) {
            continue;
        }

        const armorId = makeId(langNames[ContentLangIndex.en]);
        const rarity = armor.rare - 1;
        let sexType = "";

        switch (armor.sexual_equipable) {
            case "Both":
                {
                    sexType = "all";
                }
                break;

            case "MaleOnly":
                {
                    sexType = "male";
                }
                break;

            case "FemaleOnly":
                {
                    sexType = "female";
                }
                break;
        }

        if (rarity === 9) {
            if (armorId.startsWith("chaotic")) {
                sexType = "male";
            } else if (armorId.startsWith("nephilim")) {
                sexType = "female";
            }
        }

        let savePart = part;

        switch (part) {
            case "head":
                {
                    savePart = "helm";
                }
                break;
            case "chest":
                {
                    savePart = "torso";
                }
                break;
            case "leg":
                {
                    savePart = "feet";
                }
                break;
        }

        const slots = [];

        for (
            let size_index = armor.decorations_num_list.length - 1;
            0 <= size_index;
            --size_index
        ) {
            for (let i = 0; i < armor.decorations_num_list[size_index]; ++i) {
                slots.push(size_index + 1);
            }
        }

        while (slots.length < 3) {
            slots.push(0);
        }

        if (3 < slots.length) {
            console.error(
                `Slots length should be 3, not ${slots.length}: ${armor.decorations_num_list}`
            );
        }

        const skills = {} as {
            [key: string]: ArmorFinalSkillInfo;
        };

        for (const index in armor.skill_list) {
            const skillInfo = armor.skill_list[index];

            if (skillInfo === "None") {
                continue;
            }

            let isMr = false;
            let skillId = -1;

            if (skillInfo.MrSkill !== undefined) {
                isMr = true;
                skillId = skillInfo.MrSkill;

                skills[mrSkillIds[skillId]] = {
                    level: armor.skill_lv_list[index],
                } as ArmorFinalSkillInfo;
            } else if (skillInfo.Skill !== undefined) {
                skillId = skillInfo.Skill;

                skills[hrSkillIds[skillId]] = {
                    level: armor.skill_lv_list[index],
                } as ArmorFinalSkillInfo;
            }
        }

        const stat = {
            defense: armor.def_val,
            fireRes: armor.fire_reg_val,
            waterRes: armor.water_reg_val,
            iceRes: armor.ice_reg_val,
            elecRes: armor.thunder_reg_val,
            dragonRes: armor.dragon_reg_val,
        } as ArmorStatInfo;

        const armorNames = {} as { [key: string]: string };

        for (const [index, langCode] of LangCodeMap) {
            armorNames[langCode] = names[part][id][index];
        }

        const result = {
            id: armorId,
            part: savePart,
            sexType,
            names: armorNames,
            rarity,
            stat,
            skills,
            slots,
        } as FinalArmorInfo;

        if (armorIdSet.has(armorId)) {
            const prevIndex = ret.findIndex(prevArmor => prevArmor.id === armorId);
            const prev = ret[prevIndex];

            let sameBaseStats = true;
            let sameSkills = true;
            let sameNames = true;

            // Same base stats
            sameBaseStats = prev.part === result.part && prev.rarity === result.rarity;

            for (let i = 0; i < prev.slots.length; ++i) {
                sameBaseStats = sameBaseStats && (prev.slots[i] === result.slots[i]);
            }

            sameBaseStats = sameBaseStats && (
                prev.stat.defense === result.stat.defense &&
                prev.stat.fireRes === result.stat.fireRes &&
                prev.stat.waterRes === result.stat.waterRes &&
                prev.stat.iceRes === result.stat.iceRes &&
                prev.stat.elecRes === result.stat.elecRes &&
                prev.stat.dragonRes === result.stat.dragonRes
            );

            // Same skills & stormsoul exception
            let prevHasStormSoul = false;
            let curHasStormSoul = false;

            for (const skillId in prev.skills) {
                sameSkills = sameSkills && (prev.skills[skillId]?.level === result.skills[skillId]?.level);

                if (skillId === "stormsoul") {
                    prevHasStormSoul = true;
                }
            }

            for (const skillId in result.skills) {
                sameSkills = sameSkills && (prev.skills[skillId]?.level === result.skills[skillId]?.level);

                if (skillId === "stormsoul") {
                    curHasStormSoul = true
                }
            }

            // Same names
            for (const lang in prev.names) {
                sameNames = sameNames && (prev.names[lang] === result.names[lang]);
            }

            if (prev.sexType === result.sexType) {
                if (sameBaseStats && sameSkills) {
                    if (sameNames === false) {
                        console.log(`Armor id ${armorId} has two armors with different names`);
                    }

                    continue;
                }

                if (!prevHasStormSoul && curHasStormSoul) {
                    ret.splice(prevIndex, 1);
                } else if (prevHasStormSoul && !curHasStormSoul) {
                    continue;
                } else {
                    throw `Error: duplicate armor id ${armorId} exists`;
                }
            } else {
                if (sameBaseStats && sameSkills) {
                    prev.sexType = "all";
                    continue;
                }

                prev.id = `${prev.id}_${prev.sexType}`;
                result.id = `${result.id}_${result.sexType}`;

                for (const lang in prev.names) {
                    const prevName = prev.names[lang];
                    const curName = result.names[lang];

                    if (prevName === curName) {
                        prev.names[lang] = `${prevName} (${prev.sexType})`;
                        result.names[lang] = `${curName} (${result.sexType})`;
                    }
                }

                armorIdSet.add(prev.id);
                armorIdSet.add(result.id);
            }
        } else {
            armorIdSet.add(armorId);
        }

        ret.push(result);
    }

    return ret;
}
