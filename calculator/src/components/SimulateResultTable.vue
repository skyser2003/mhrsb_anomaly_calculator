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
import { getDefaultStat } from "../definition/armor_define";

interface TableData {
	helm: string;
	torso: string;
	arm: string;
	waist: string;
	feet: string;
	talisman: string;
	stat: string;
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
	if (calcResult.full_equipments === undefined) {
		return [];
	}

	return calcResult.full_equipments.map((equips, index) => {
		const allLeftoverSlots = {} as { [key: string]: boolean };

		for (const comb of equips.deco_combs) {
			const key = JSON.stringify(comb.leftover_slots_sum);

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
			helm: ArmorsData.getName(equips.armors["helm"].base_id, props.langData),
			torso: ArmorsData.getName(equips.armors["torso"].base_id, props.langData),
			arm: ArmorsData.getName(equips.armors["arm"].base_id, props.langData),
			waist: ArmorsData.getName(equips.armors["waist"].base_id, props.langData),
			feet: ArmorsData.getName(equips.armors["feet"].base_id, props.langData),
			talisman: getTalismanText(equips.talisman),
			stat: "",
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

		<template #bodyCell="{ text, index, column }">
			<template
				v-if="ArmorsData.isArmorPart(column.key) && calcResult.full_equipments[index].armors[column.key].is_anomaly === true">
				{{ text }} <a-image :src="`${getAnomalyImageName()}`" :width="20" :preview="false" />
			</template>
			<template v-else-if="column.key === 'stat'">
				<StatTable :langData="langData" :stat="getTotalStat(calcResult.full_equipments[index].armors)" />
			</template>
		</template>

		<template #expandedRowRender="{ index }">
			<SimulateResultRow :langData="langData" :data="calcResult.full_equipments[index]" v-on:add_result_favorite="addResultFavorite" />
		</template>
	</a-table>
</template>

<style>

.cursor-pointer {
	cursor: pointer;
}

</style>