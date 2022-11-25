import { ResultDecorationCombination } from "../definition/calculate_result";

import { SkillsData } from "../models/skills";
import { DecosData } from "../models/decos";

import UIData from "../ui_data/ui_data.json";
import { Language } from "../definition/language";


export function getDecoCombTexts(comb: ResultDecorationCombination, langData: Language) {
	const skillData = {} as { [key: string]: { [key: string]: number } };

	for (const id in comb.skill_decos) {
		const skillName = SkillsData.getName(id, langData);
		const decos = comb.skill_decos[id];

		skillData[skillName] = {};

		for (let index = 0; index < decos.length; ++index) {
			const count = decos[index];

			if (count == 0) {
				continue;
			}

			const info = DecosData.getInfo(id, index);

			const decoName = info.names[langData];

			skillData[skillName][decoName] = count;
		};
	}

	const allDecoTexts = [];

	for (const skillName in skillData) {
		const decoData = skillData[skillName];

		const decoTexts = [];

		for (const decoName in decoData) {
			const count = decoData[decoName];

			const text = `${decoName} x ${count}${UIData["deco_unit"][langData]}`;
			decoTexts.push(text);
		}

		const text = `(${decoTexts.join(" | ")})`;

		allDecoTexts.push(text);
	}

	return allDecoTexts;
}