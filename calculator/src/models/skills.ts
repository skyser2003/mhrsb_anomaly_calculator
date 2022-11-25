import { FinalSkillInfo } from "../definition/skill_define";

import SkillsVec from "../data/skill.json";

class SkillsDataManager {
    private skillsMap: { [key: string]: FinalSkillInfo };

    constructor(private skillsVec: FinalSkillInfo[]) {
        this.skillsMap = {};

        for (const skill of skillsVec) {
            this.skillsMap[skill.id] = skill;
        }
    }

    getName(id: string, lang: string) {
        return this.skillsMap[id].names[lang];
    }
}

export const SkillsData = new SkillsDataManager(SkillsVec);
