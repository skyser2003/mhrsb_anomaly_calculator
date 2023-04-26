import { FinalSkillInfo } from "../definition/skill_define";

import SkillsVec from "../data/skill.json";
import { Skills } from "../definition/calculate_result";

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

    sortByName(skills: Skills, lang: string) {
        const sortedKeys = Object.keys(skills).sort((id1, id2) => {
            const name1 = this.skillsMap[id1].names[lang];
            const name2 = this.skillsMap[id2].names[lang];

            return name1 > name2 ? 1 : -1;
        });

        const retSkills = {} as Skills;

        for (const key of sortedKeys) {
            retSkills[key] = skills[key];
        }

        return retSkills;
    }
}

export const SkillsData = new SkillsDataManager(SkillsVec);
