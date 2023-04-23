<script setup lang="ts">
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/tauri";

import { MinMaxSkills, Skills, Slots } from "../definition/calculate_result";
import { Language } from "../definition/language";

import { SkillsData } from "../models/skills";

import UIData from "../ui_data/ui_data.json";


const props = defineProps<{
	langData: Language;
	skills: MinMaxSkills;
	slots: Slots;
	selectedSkills: Skills;
	selectedSlots: Slots;
	originalSkills: Skills;
	originalSlots: Slots;
}>();

const slotsColumn = ref([
	{
		title: UIData["additional_slot"][props.langData],
		dataIndex: "index_view",
		key: "index_view",
		width: 50,
	},
	{
		title: UIData["additional_slot_level"][props.langData],
		dataIndex: "max_level",
		key: "max_level",
		width: 50
	}
]);

const skillsColumn = ref([
	{
		title: UIData["additional_skill"][props.langData],
		dataIndex: "additional_skill",
		key: "additional_skill",
		width: 50,
	},
	{
		title: UIData["additional_skill_level"][props.langData],
		dataIndex: "max_level",
		key: "max_level",
		width: 50
	}
]);

function getSlotTableData() {
	return props.slots.map((maxLevel, index) => {
		return {
			index: index,
			index_view: `${UIData['slots_name'][props.langData]} ${index + 1}`,
			max_level: maxLevel,
			min_level: props.originalSlots[index] + 1,
		};
	});
}

function getSkillTableData() {
	return Object.keys(props.skills).map(skillId => {
		return {
			skill_id: skillId,
			additional_skill: SkillsData.getName(skillId, props.langData),
			max_level: props.skills[skillId][1],
			min_level: props.skills[skillId][0],
		};
	});
}

function getDefaultSlotLevel(index: number) {
	return props.originalSlots[index];
}

function getDefaultSkillLevel(skillId: string) {
	return props.originalSkills[skillId] ?? 0
}

</script>

<template>
	<a-table :columns="slotsColumn" :data-source="getSlotTableData()" :pagination="{ defaultPageSize: 200, hideOnSinglePage: true }">
		<template #bodyCell="{ column, record }">
			<template v-if="column.key === 'max_level'">
				<a-select v-model:value="selectedSlots[record.index]">
					<a-select-option :value="getDefaultSlotLevel(record.index)">---</a-select-option>
					<a-select-option v-if="0 <= record.max_level - record.min_level" v-for="level in (record.max_level - record.min_level + 1)" :value="level - 1 + record.min_level">
						{{ level - 1 + record.min_level }}
					</a-select-option>
				</a-select>
			</template>
		</template>
	</a-table>

	<a-table :columns="skillsColumn" :data-source="getSkillTableData()" :pagination="{ defaultPageSize: 200, hideOnSinglePage: true }">
		<template #bodyCell="{ column, record }">
			<template v-if="column.key === 'max_level'">
				<a-select v-model:value="selectedSkills[record.skill_id]">
					<a-select-option :value="getDefaultSkillLevel(record.skill_id)">---</a-select-option>
					<a-select-option v-for="level in (record.max_level - record.min_level + 1)" :value="level - 1 + record.min_level">
						{{ level - 1 + record.min_level }}
					</a-select-option>
				</a-select>
			</template>
		</template>
	</a-table>
</template>