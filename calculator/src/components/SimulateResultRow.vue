<script setup lang="ts">

import { ref } from "vue";

import { ResultFullEquipments, ResultArmor, ResultFavorite, EquipSlots, Skills } from "../definition/calculate_result";
import { SkillsData } from "../models/skills";
import { SlotsDataManager } from "../models/slots";

import UIData from "../ui_data/ui_data.json";
import { getDecoCombTexts } from "../model/ui";
import { Language } from "../definition/language";

interface RowData {
	row_info: string;
	helm: string;
	torso: string;
	arm: string;
	waist: string;
	feet: string;
}

const props = defineProps<{
	langData: Language,
	data: ResultFullEquipments,
}>();

const emits = defineEmits<{
	(event: "add_result_favorite", fav: ResultFavorite): void
}>();

const equipColumns = ref([
	{
		title: "",
		dataIndex: "row_info",
		key: "row_info",
		width: 50,
	},
	{
		title: UIData["helm_name"][props.langData],
		dataIndex: "helm",
		key: "helm",
		width: 50,
	},
	{
		title: UIData["torso_name"][props.langData],
		dataIndex: "torso",
		key: "torso",
		width: 50,
	},
	{
		title: UIData["arm_name"][props.langData],
		dataIndex: "arm",
		key: "arm",
		width: 50,
	},
	{
		title: UIData["waist_name"][props.langData],
		dataIndex: "waist",
		key: "waist",
		width: 50,
	},
	{
		title: UIData["feet_name"][props.langData],
		dataIndex: "feet",
		key: "feet",
		width: 50,
	}
]);

const decoColumns = [
	{
		title: UIData["decorations_name"][props.langData],
		dataIndex: "decos",
		key: "decos",
		width: 500,
	},
	{
		title: UIData["leftover_slots"][props.langData],
		dataIndex: "slots",
		key: "slots",
		width: 70,
	},
	{
		title: UIData["excess_skills"][props.langData],
		dataIndex: "leftover_skills",
		key: "leftover_skills",
		width: 200,
	},
	{
		title: UIData["save"][props.langData],
		dataIndex: "add_result_favorite",
		key: "add_result_favorite",
		width: 100,
	}
];

const savedCheck = ref<{ [key: number]: boolean }>({});

function getArmorData(data: ResultArmor) {
	const skillTexts = [];

	for (const id in data.skills) {
		const name = SkillsData.getName(id, props.langData);
		const level = data.skills[id];

		const text = `${name} Lv${level}`;
		skillTexts.push(text);
	}

	return `${skillTexts.join(", ")} / ${UIData["slots_name"][props.langData]} ${JSON.stringify(SlotsDataManager.convertToBase(data.baseSlots))}`;
}

function getArmorDiffData(data: ResultArmor) {
	if (data.isAnomaly === false) {
		return "";
	}

	const skillTexts = [];

	if (Object.keys(data.diffSkills).length === 0) {
		skillTexts.push(`(${UIData["no_diff_skill"][props.langData]})`);
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

	for(let i = 0; i < data.diffSlots.length; ++i) {
		let diff = data.diffSlots[i];
		let diff_text = "";

		if (0 < diff) {
			diff_text = `(+${ data.diffSlots[i] })`;
		}

		let text = `${slots[i]}${diff_text}`;
		slot_diff_texts.push(text);
	}

	return `${skillTexts.join(", ")} / ${UIData['slots_name'][props.langData]} [${slot_diff_texts.join(", ")}]`;
}

function getRowData(data: ResultFullEquipments) {
	const originalData = {
		row_info: UIData["base_armor_info"][props.langData],
		helm: getArmorData(data.armors["helm"]),
		torso: getArmorData(data.armors["torso"]),
		arm: getArmorData(data.armors["arm"]),
		waist: getArmorData(data.armors["waist"]),
		feet: getArmorData(data.armors["feet"]),
	} as RowData;

	const diffData = {
		row_info: UIData["anomaly_craft_info"][props.langData],
		helm: getArmorDiffData(data.armors["helm"]),
		torso: getArmorDiffData(data.armors["torso"]),
		arm: getArmorDiffData(data.armors["arm"]),
		waist: getArmorDiffData(data.armors["waist"]),
		feet: getArmorDiffData(data.armors["feet"]),
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

		if (leftoverSkills.length === 0) {
			leftoverSkills.push(`(${UIData["no_excess_skill"][props.langData]})`);
		}

		return { decos: allDecoTexts.join(" - "), slots: JSON.stringify(comb.leftoverSlotsSum), leftover_skills: leftoverSkills.join(", ") };
	});

	return decoCombs;
}

function addResultFavorite(index: number) {
	const copyData = JSON.parse(JSON.stringify(props.data)) as ResultFullEquipments;

	const armors = copyData.armors;
	const talisman = copyData.talisman;
	const decoComb = copyData.decoCombs[index];

	for (const skillId in copyData.commonLeftoverSkills) {
		const prevLevel = decoComb.leftoverSkills[skillId] || 0;
		const level = prevLevel + copyData.commonLeftoverSkills[skillId];

		decoComb.leftoverSkills[skillId] = level;
	}

	const fav: ResultFavorite = {
		name: "",
		sexType: copyData.sexType,
		weaponSlots: copyData.weaponSlots,
		armors,
		talisman,
		decoComb
	};
	
	emits("add_result_favorite", fav);

	savedCheck.value[index] = true;

	setTimeout(() => {
		savedCheck.value[index] = false;
	}, 5000);
}

</script>

<template>
	<a-table :columns="equipColumns" :data-source="getRowData(props.data)" :pagination="{ defaultPageSize: 100, hideOnSinglePage: true}">
	</a-table>
	<a-table :columns="decoColumns" :data-source="getDecoCombData(props.data)" :pagination="{ defaultPageSize: 100, hideOnSinglePage: true}">
		<template #bodyCell="{ index, column }">
			<template v-if="column.key === 'add_result_favorite'">
				<a-button :type="savedCheck[index] === true ? 'dashed' : 'primary'" :disabled="savedCheck[index] === true" @click="addResultFavorite(index)">Save</a-button>
				<template v-if="savedCheck[index] === true">
					Saved!
				</template>
			</template>
		</template>
	</a-table>
</template>