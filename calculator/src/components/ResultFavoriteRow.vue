<script setup lang="ts">

import { ref } from "vue";

import { ResultFullEquipments, ResultArmor, ResultFavorite, EquipSlots, ResultTalisman } from "../definition/calculate_result";
import { SkillsData } from "../models/skills";
import { DecosData } from "../models/decos";
import { SlotsDataManager } from "../models/slots";

import { lm } from "../model/language_manager";
import { EquipPartData, getDecoCombTexts } from "../model/ui";
import { Language } from "../definition/language";


interface RowData {
	row_info: string;
	helm: EquipPartData;
	torso: EquipPartData;
	arm: EquipPartData;
	waist: EquipPartData;
	feet: EquipPartData;
	talisman: EquipPartData;
}

const props = defineProps<{
	langData: Language,
	data: ResultFullEquipments,
}>();

const equipColumns = ref([
	{
		title: "",
		dataIndex: "row_info",
		key: "row_info",
		width: 50,
	},
	{
		title: lm.getString("helm_name"),
		dataIndex: "helm",
		key: "helm",
		width: 50,
	},
	{
		title: lm.getString("torso_name"),
		dataIndex: "torso",
		key: "torso",
		width: 50,
	},
	{
		title: lm.getString("arm_name"),
		dataIndex: "arm",
		key: "arm",
		width: 50,
	},
	{
		title: lm.getString("waist_name"),
		dataIndex: "waist",
		key: "waist",
		width: 50,
	},
	{
		title: lm.getString("feet_name"),
		dataIndex: "feet",
		key: "feet",
		width: 50,
	},
	{
		title: lm.getString("talisman_name"),
		dataIndex: "talisman",
		key: "talisman",
		width: 50,
	}
]);

const decoColumns = [
	{
		title: lm.getString("decorations_name"),
		dataIndex: "decos",
		key: "decos",
		width: 500,
	},
	{
		title: lm.getString("leftover_slots"),
		dataIndex: "slots",
		key: "slots",
		width: 70,
	},
	{
		title: lm.getString("leftover_skills"),
		dataIndex: "leftover_skills",
		key: "leftover_skills",
		width: 200,
	}
];

function getArmorData(data: ResultArmor) {
	const skillTexts = [];

	for (const id in data.skills) {
		const name = SkillsData.getName(id, props.langData);
		const level = data.skills[id];

		const text = `${name} Lv${level}`;
		skillTexts.push(text);
	}

	return {
		skills: skillTexts,
		slots: `${lm.getString("slots_name")} ${JSON.stringify(SlotsDataManager.convertToBase(data.baseSlots))}`
	};
}

function getArmorDiffData(data: ResultArmor) {
	if (data.isAnomaly === false) {
		return "";
	}

	const skillTexts = [];

	if (Object.keys(data.diffSkills).length === 0) {
		skillTexts.push(`(${lm.getString("no_diff_skill")})`);
	} else {
		const texts = [];

		for (const id in data.diffSkills) {
			const name = SkillsData.getName(id, props.langData);
			const level = data.diffSkills[id];
			const absLevel = Math.abs(level);
			const levelText = level > 0 ? `+Lv${absLevel}` : `-Lv${absLevel}`;

			const text = `${name} ${levelText}`;
			texts.push(text);
		}

		skillTexts.push(texts.join(", "));
	}

	const slots = SlotsDataManager.convertToBase(data.slots);
	const slot_diff_texts = [];

	for (let i = 0; i < data.diffSlots.length; ++i) {
		let diff = data.diffSlots[i];
		let diff_text = "";

		if (0 < diff) {
			diff_text = `(+${data.diffSlots[i]})`;
		}

		let text = `${slots[i]}${diff_text}`;
		slot_diff_texts.push(text);
	}

	return {
		skills: skillTexts,
		slots: `${lm.getString('slots_name')} [${slot_diff_texts.join(", ")}]`
	};
}

function getTalismanText(talisman: ResultTalisman) {
	const taliSkills = talisman.skills;
	const taliSlots = talisman.slots;

	const skillTexts = [];

	for (const skillId in taliSkills) {
		const name = SkillsData.getName(skillId, props.langData);

		const text = `${name} Lv${taliSkills[skillId]}`;
		skillTexts.push(text);
	}

	const taliBaseSlots = SlotsDataManager.convertToBase(taliSlots);

	const slotsName = lm.getString("slots_name");

	return {
		skills: skillTexts,
		slots: `${slotsName} ${JSON.stringify(taliBaseSlots)}`
	};
}

function getRowData(data: ResultFullEquipments) {
	const originalData = {
		row_info: lm.getString("base_armor_info"),
		helm: getArmorData(data.armors["helm"]),
		torso: getArmorData(data.armors["torso"]),
		arm: getArmorData(data.armors["arm"]),
		waist: getArmorData(data.armors["waist"]),
		feet: getArmorData(data.armors["feet"]),
		talisman: getTalismanText(data.talisman),
	} as RowData;

	const diffData = {
		row_info: lm.getString("anomaly_craft_info"),
		helm: getArmorDiffData(data.armors["helm"]),
		torso: getArmorDiffData(data.armors["torso"]),
		arm: getArmorDiffData(data.armors["arm"]),
		waist: getArmorDiffData(data.armors["waist"]),
		feet: getArmorDiffData(data.armors["feet"]),
		talisman: { skills: [], slots: "" },
	} as RowData;

	return [originalData, diffData];
}

function getDecoCombData(data: ResultFullEquipments) {
	const decoCombs = data.decoCombs.map(comb => {
		const allDecoTexts = getDecoCombTexts(comb, props.langData);

		const leftoverSkills = [];

		for (const skillId in comb.leftoverSkills) {
			const level = comb.leftoverSkills[skillId];

			const skillName = SkillsData.getName(skillId, props.langData);

			const text = `${skillName} Lv${level}`;

			leftoverSkills.push(text);
		}

		return { decos: allDecoTexts, slots: JSON.stringify(comb.leftoverSlotsSum), leftover_skills: leftoverSkills };
	});

	return decoCombs;
}

</script>

<template>
	<a-table :columns="equipColumns" :data-source="getRowData(props.data)"
		:pagination="{ defaultPageSize: 100, hideOnSinglePage: true }">
		<template #bodyCell="{ column, record }">
			<template v-if="column.key !== 'row_info'">
				<template v-if="record[column.key].skills !== undefined && record[column.key].skills.length !== 0">
					<a-tag v-for="skill in record[column.key].skills">
						{{ skill }}
					</a-tag>
					<br />
					<div style="margin-bottom: 10px;"></div>
				</template>
				{{ record[column.key].slots }}
			</template>
		</template>
	</a-table>
	<a-table :columns="decoColumns" :data-source="getDecoCombData(props.data)"
		:pagination="{ defaultPageSize: 100, hideOnSinglePage: true }">
		<template #bodyCell="{ column, record }">
			<template v-if="column.key !== 'slots'">
				<a-tag v-for="elem in record[column.key]">
					{{ elem }}
				</a-tag>
			</template>
		</template>
	</a-table>
</template>