<script setup lang="ts">
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/tauri";

import { MinMaxSkills, Skills, Slots } from "../definition/calculate_result";
import { Language } from "../definition/language";

import { SkillsData } from "../models/skills";

import { lm } from "../model/language_manager";


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
		title: lm.getString("additional_slot"),
		dataIndex: "index_view",
		key: "index_view",
		width: 150,
	},
	{
		title: lm.getString("additional_slot_level"),
		dataIndex: "max_level",
		key: "max_level",
	}
]);

const skillsColumn = ref([
	{
		title: lm.getString("additional_skill"),
		dataIndex: "additional_skill",
		key: "additional_skill",
		width: 150,
	},
	{
		title: lm.getString("additional_skill_level"),
		dataIndex: "max_level",
		key: "max_level",
	}
]);

function getSlotTableData() {
	return props.slots.map((maxLevel, index) => {
		return {
			index: index,
			index_view: `${lm.getString('slots_name')} ${index + 1}`,
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
				<a-radio-group v-model:value="selectedSlots[record.index]">
					<a-radio-button :value="getDefaultSlotLevel(record.index)">{{ getDefaultSlotLevel(record.index) }}</a-radio-button>
					<a-radio-button v-if="0 <= record.max_level - record.min_level" v-for="level in (record.max_level - record.min_level + 1)" :value="level - 1 + record.min_level">
						{{ level - 1 + record.min_level }}
					</a-radio-button>
				</a-radio-group>
			</template>
		</template>
	</a-table>

	<a-table :columns="skillsColumn" :data-source="getSkillTableData()" :pagination="{ defaultPageSize: 200, hideOnSinglePage: true }">
		<template #bodyCell="{ column, record }">
			<template v-if="column.key === 'max_level'">
				<a-radio-group v-model:value="selectedSkills[record.skill_id]">
					<a-radio-button :value="getDefaultSkillLevel(record.skill_id)">{{ getDefaultSkillLevel(record.skill_id) }}</a-radio-button>
					<a-radio-button v-for="level in (record.max_level - record.min_level + 1)" :value="level - 1 + record.min_level">
						{{ level - 1 + record.min_level }}
					</a-radio-button>
				</a-radio-group>
			</template>
		</template>
	</a-table>
</template>