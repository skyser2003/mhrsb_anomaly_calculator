<script setup lang="ts">
import { ref } from "vue";

import { CalculateResult, ResultFavorite, ResultTalisman, getTotalStat } from "../definition/calculate_result";

import { ArmorsData } from "../models/armors";
import { SkillsData } from "../models/skills";
import { SlotsDataManager } from "../models/slots";

import SimulateResultRow from "./SimulateResultRow.vue";
import StatTable from "./StatTable.vue";

import { lm } from "../model/language_manager";
import { Language } from "../definition/language";
import { ArmorStatInfo, getDefaultStat } from "../definition/armor_define";
import { EquipPartData } from "../model/ui";

interface TableData {
	helm: string;
	torso: string;
	arm: string;
	waist: string;
	feet: string;
	talisman: EquipPartData;
	stat: ArmorStatInfo;
	leftover_slots: string;
	common_leftover_skills: string[];
	anomalyInfo: { [key: string]: boolean };
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
		title: lm.getString("helm_name"),
		dataIndex: "helm",
		key: "helm"
	},
	{
		title: lm.getString("torso_name"),
		dataIndex: "torso",
		key: "torso"
	},
	{
		title: lm.getString("arm_name"),
		dataIndex: "arm",
		key: "arm"
	},
	{
		title: lm.getString("waist_name"),
		dataIndex: "waist",
		key: "waist"
	},
	{
		title: lm.getString("feet_name"),
		dataIndex: "feet",
		key: "feet"
	},
	{
		title: lm.getString("talisman_name"),
		dataIndex: "talisman",
		key: "talisman"
	},
	{
		title: lm.getString("common_leftover_skills"),
		dataIndex: "common_leftover_skills",
		key: "common_leftover_skills"
	},
	{
		title: lm.getString("leftover_slots"),
		dataIndex: "leftover_slots",
		key: "leftover_slots"
	},
	{
		title: lm.getString("stat_name"),
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

	const slotsName = lm.getString("slots_name");

	return {
		skills: skillTexts,
		slots: `${slotsName} ${JSON.stringify(taliBaseSlots)}`
	} as EquipPartData;
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
			const emptyKey = JSON.stringify([0, 0, 0, 0]);
			allLeftoverSlots[emptyKey] = true;
		}

		const allLeftoverSlotsTexts = [];

		for (const key in allLeftoverSlots) {
			allLeftoverSlotsTexts.push(key);
		}

		const leftoverSkillsText = [];

		for (const skillId in equips.commonLeftoverSkills) {
			const level = equips.commonLeftoverSkills[skillId];
			const skillName = SkillsData.getName(skillId, props.langData);

			const text = `${skillName} Lv${level}`;

			leftoverSkillsText.push(text);
		}

		const anomalyInfo = {} as { [key: string]: boolean };

		for (const key in equips.armors) {
			const armor = equips.armors[key];
			anomalyInfo[key] = armor.isAnomaly;
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
			common_leftover_skills: leftoverSkillsText,
			anomalyInfo
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
	<a-table :columns="columns" :data-source="generateTableData(calcResult)"
		:pagination="{ defaultPageSize: 200, hideOnSinglePage: true }" :expand-row-by-click="true"
		:row-class-name="() => 'cursor-pointer'">
		<template #expandIcon="{ }">
		</template>

		<template #bodyCell="{ text, column, record }">
			<template v-if="record.anomalyInfo[column.key] === true">
				{{ text }} <a-image :src="`${getAnomalyImageName()}`" :width="20" :preview="false" />
			</template>
			<template v-else-if="column.key === 'talisman'">
				<template v-if="record[column.key].skills !== undefined && record[column.key].skills.length !== 0">
					<a-tag v-for="skill in record[column.key].skills">
						{{ skill }}
					</a-tag>
					<br />
					<div style="margin-bottom: 10px;"></div>
				</template>
				{{ record[column.key].slots }}
			</template>
			<template v-else-if="column.key === 'common_leftover_skills'">
				<a-tag v-for="skill in record.common_leftover_skills">
					{{ skill }}
				</a-tag>
			</template>
			<template v-else-if="column.key === 'stat'">
				<StatTable :langData="langData" :stat="record.stat" />
			</template>
		</template>

		<template #expandedRowRender="{ index }">
			<SimulateResultRow :langData="langData" :data="calcResult.fullEquipments[index]"
				v-on:add_result_favorite="addResultFavorite" />
		</template>
	</a-table>
</template>

<style>
.cursor-pointer {
	cursor: pointer;
}
</style>