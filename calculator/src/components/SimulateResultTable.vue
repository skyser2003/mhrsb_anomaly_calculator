<script setup lang="ts">
import { ref, Ref } from "vue";

import { CalculateResult, ResultFavorite, ResultTalisman, getTotalStat } from "../definition/calculate_result";

import { ArmorsData } from "../models/armors";
import { SkillsData } from "../models/skills";
import { SlotsDataManager } from "../models/slots";

import SimulateResultRow from "./SimulateResultRow.vue";
import StatTable from "./StatTable.vue";

import UIData from "../ui_data/ui_data.json";
import { Language } from "../definition/language";
import { ArmorStatInfo, getDefaultStat } from "../definition/armor_define";

interface TableData {
	helm: string;
	torso: string;
	arm: string;
	waist: string;
	feet: string;
	talisman: string;
	stat: ArmorStatInfo;
	leftover_slots: string;
}

const props = defineProps<{
	langData: Language,
	calcResult: CalculateResult
}>();

const emits = defineEmits<{
	(event: "add_result_favorite", fav: ResultFavorite): void
}>();

const columns = ref([
	{
		title: UIData["helm_name"][props.langData],
		dataIndex: "helm",
		key: "helm"
	},
	{
		title: UIData["torso_name"][props.langData],
		dataIndex: "torso",
		key: "torso"
	},
	{
		title: UIData["arm_name"][props.langData],
		dataIndex: "arm",
		key: "arm"
	},
	{
		title: UIData["waist_name"][props.langData],
		dataIndex: "waist",
		key: "waist"
	},
	{
		title: UIData["feet_name"][props.langData],
		dataIndex: "feet",
		key: "feet"
	},
	{
		title: UIData["talisman_name"][props.langData],
		dataIndex: "talisman",
		key: "talisman"
	},
	{
		title: UIData["leftover_slots"][props.langData],
		dataIndex: "leftover_slots",
		key: "leftover_slots"
	},
	{
		title: UIData["stat_name"][props.langData],
		dataIndex: "stat",
		key: "stat"
	}
]);

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

	const slotsName = UIData["slots_name"][props.langData];

	return skillTexts.join(", ") + " / " + `${slotsName} ${JSON.stringify(taliBaseSlots)}`;
}

function generateTableData(calcResult: CalculateResult) {
	if (calcResult.fullEquipments === undefined) {
		return [];
	}

	return calcResult.fullEquipments.map((equips, index) => {
		const allLeftoverSlots = {} as { [key: string]: boolean };

		for (const comb of equips.decoCombs) {
			const key = JSON.stringify(comb.leftoverSlotsSum);

			allLeftoverSlots[key] = true;
		}

		if (Object.keys(allLeftoverSlots).length === 0) {
			const emptyKey = JSON.stringify([0,0,0,0]);
			allLeftoverSlots[emptyKey] = true;
		}

		const allLeftoverSlotsTexts = [];

		for (const key in allLeftoverSlots) {
			allLeftoverSlotsTexts.push(key);
		}

		return {
			key: index,
			helm: ArmorsData.getName(equips.armors["helm"].baseId, props.langData),
			torso: ArmorsData.getName(equips.armors["torso"].baseId, props.langData),
			arm: ArmorsData.getName(equips.armors["arm"].baseId, props.langData),
			waist: ArmorsData.getName(equips.armors["waist"].baseId, props.langData),
			feet: ArmorsData.getName(equips.armors["feet"].baseId, props.langData),
			talisman: getTalismanText(equips.talisman),
			stat: getTotalStat(equips.armors),
			leftover_slots: allLeftoverSlotsTexts.join(", "),
		} as TableData;
	});
}

function addResultFavorite(fav: ResultFavorite) {
	emits("add_result_favorite", fav);
}

function getAnomalyImageName() {
	return new URL('../assets/anomaly_item.png', import.meta.url).href;
}

</script>

<template>
	<a-table :columns="columns" :data-source="generateTableData(calcResult)" :pagination="{ defaultPageSize: 200, hideOnSinglePage: true}" :expand-row-by-click="true" :row-class-name="() => 'cursor-pointer'">
		<template #expandIcon="{}">
		</template>

		<template #bodyCell="{ text, index, column, record }">
			<template
				v-if="ArmorsData.isArmorPart(column.key) && calcResult.fullEquipments[index].armors[column.key].isAnomaly === true">
				{{ text }} <a-image :src="`${getAnomalyImageName()}`" :width="20" :preview="false" />
			</template>
			<template v-else-if="column.key === 'stat'">
				<StatTable :langData="langData" :stat="record.stat" />
			</template>
		</template>

		<template #expandedRowRender="{ index }">
			<SimulateResultRow :langData="langData" :data="calcResult.fullEquipments[index]" v-on:add_result_favorite="addResultFavorite" />
		</template>
	</a-table>
</template>

<style>

.cursor-pointer {
	cursor: pointer;
}

</style>