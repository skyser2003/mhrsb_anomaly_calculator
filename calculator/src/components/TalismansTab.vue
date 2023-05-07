<script setup lang="ts">
// This starter template is using Vue 3 <script setup> SFCs
// Check out https://vuejs.org/api/sfc-script-setup.html#script-setup
import { ref } from "vue";
import { open } from '@tauri-apps/api/dialog';
import { invoke } from "@tauri-apps/api/tauri";
import type { SelectProps } from "ant-design-vue";

import SkillsVec from "../data/skill.json";

import { FinalSkillInfo } from "../definition/skill_define";
import { Language } from "../definition/language";

import { lm } from "../model/language_manager";
import { TalismanInfo, MAX_SLOT_LEVEL } from "../definition/calculate_result";
import { CacheManager } from "../model/data_manager";
import { InvokeManager } from "../model/invoke_manager";

const props = defineProps<{ langData: Language }>();

defineExpose({ getFileTalismans });

const columns = ref([
	{
		title: `${lm.getString("skill_column")} 1`,
		dataIndex: "skill1",
		key: "skill1",
		width: 300,
	},
	{
		title: `${lm.getString("skill_column")} 2`,
		dataIndex: "skill2",
		key: "skill2",
		width: 300,
	},
	{
		title: lm.getString("slots_name"),
		dataIndex: "slots",
		key: "slots",
	}
]);

const manualColumns = ref([
	{
		title: `${lm.getString("skill_column")} 1`,
		dataIndex: "skill1",
		key: "skill1",
		width: 300,
	},
	{
		title: `${lm.getString("skill_column")} 2`,
		dataIndex: "skill2",
		key: "skill2",
		width: 300,
	},
	{
		title: lm.getString("slots_name"),
		dataIndex: "slots",
		key: "slots",
	},
	{
		title: lm.getString("delete"),
		dataIndex: "delete",
		key: "delete"
	}
]);

const addAnomalyColumns = ref([
	{
		title: `${lm.getString("skill_column")} 1`,
		dataIndex: "skill1",
		key: "skill1",
		skillId: 0,
		width: 200,
	},
	{
		title: `${lm.getString("skill_column")} 2`,
		dataIndex: "skill2",
		key: "skill2",
		skillId: 1,
		width: 200,
	},
	{
		title: lm.getString("slots_name"),
		dataIndex: "slots",
		key: "slots",
	}
]);

const maxTalismanSkills = ref(2);
const skills = ref<{ [key: string]: FinalSkillInfo }>({});

const skillsVec = ref<FinalSkillInfo[]>(SkillsVec);
skillsVec.value.sort((elem1, elem2) => elem1.names[props.langData] > elem2.names[props.langData] ? 1 : -1);

for (const skill of skillsVec.value) {
	skills.value[skill.id] = skill;
}

const skillsOptions = ref<Exclude<SelectProps["options"], undefined>>([{
	value: "",
	label: "---"
}]);

for (let i = 0; i < skillsVec.value.length; ++i) {
	const skill = skillsVec.value[i];

	skillsOptions.value.push({
		value: skill.id,
		label: skill.names[props.langData],
	});
}

const fileTalismans = ref<TalismanInfo[]>([]);
const manualTalismans = ref<TalismanInfo[]>([]);

const talismanFilename = ref("");
const cachedTalismanFilename = CacheManager.getTalismanFilename();

if (cachedTalismanFilename !== null) {
	talismanFilename.value = cachedTalismanFilename;
	await parseTalismanFile(cachedTalismanFilename);
}

const talismanAddInfo = ref<TalismanInfo>({} as TalismanInfo);
initializeTalismanAddInfo()

await loadManualTalismans();

async function getTalismanFile() {
	const file = await open({
		multiple: false,
		directory: false,
		filters: [{
			name: "talisman_list",
			extensions: ["txt"]
		}]
	});

	if (file !== null && !Array.isArray(file)) {
		talismanFilename.value = file;

		parseTalismanFile(file);
	}
}

async function parseTalismanFile(filename: string) {
	console.log(`Talisman filename: ${filename}`);

	fileTalismans.value = await InvokeManager.parseFileTalisman(filename);
	CacheManager.setTalismanFilename(filename);

	console.log(`File talisman loaded: ${fileTalismans.value.length}`);
}

async function getFileTalismans() {
	fileTalismans.value = await InvokeManager.getFileTalismans();
}

async function clearFileTalismans() {
	talismanFilename.value = "";
	await InvokeManager.clearFileTalismans();

	fileTalismans.value = [];
	CacheManager.setTalismanFilename("");
}

async function loadManualTalismans() {
	const manuals = CacheManager.getManualTalismans();
	
	const result = await InvokeManager.setManualTalismans(manuals);

	if (result === true) {
		manualTalismans.value = manuals;
	}
}

function generateTalismanData(talismans: TalismanInfo[]) {
	return talismans.map(tali => {
		const rows = [];
		
		for (let i=0; i<2; ++i) {
			rows.push("");
		}

		for (let i=0; i<tali.skills.length; ++i) {
			const skillInfo = tali.skills[i];
			
			const name = skills.value[skillInfo.id].names[props.langData];
			const level = skillInfo.level;

			if (i < rows.length) {
				rows[i] = `${name} Lv${level}`;
			} else {
				rows.push("");
			}
		}

		return {
			skill1: rows[0],
			skill2: rows[1],
			slots: JSON.stringify(tali.slotSizes),
		};
	});
}

function dummyAddTalismanSkillSlots() {
	return [{
		skill1: "",
		skill2: "",
		slots: "",
	}];
}

function onAddTalismanSkillChange(index: number) {
	talismanAddInfo.value.skills[index].level = 0;
}

function getMaxLevel(skillId: string) {
	const skillInfo = skills.value[skillId];

	if (skillInfo === undefined) {
		return 0;
	} else {
		return skillInfo.maxLevel;
	}
}

function initializeTalismanAddInfo() {
	talismanAddInfo.value = { id: "", skills: [], slotSizes: [0, 0, 0] };

	for (let i = 0; i < maxTalismanSkills.value; ++i) {
		talismanAddInfo.value.skills.push({ id: "", level: 0 });
	}
}

async function addManualTalisman() {
	console.log(talismanAddInfo.value);

	const inserted = await InvokeManager.addManualTalisman(talismanAddInfo.value);

	console.log(`Add manual result: `, inserted);

	if (inserted !== null) {
		manualTalismans.value.push(inserted);
		CacheManager.setManualTalismans(manualTalismans.value);

		initializeTalismanAddInfo();
	}
}

async function deleteManualTalisman(index: number) {
	const talisman = manualTalismans.value[index];

	manualTalismans.value.splice(index, 1);
	CacheManager.setManualTalismans(manualTalismans.value);

	const result = await InvokeManager.setManualTalismans(manualTalismans.value);

	if (result === true) {
		console.log("Talisman deleted: ", talisman);
	}
}

async function deleteAllManualTalismans() {
	const result = await InvokeManager.clearManualTalismans();

	if (result === true) {
		manualTalismans.value.length = 0;
		CacheManager.setManualTalismans(manualTalismans.value);
	}
}

</script>

<template>
	<div class="container">
		<div>
			<h1 style="display: inline-block;">{{ lm.getString("mod_managed_talisman_explanation") }} </h1>
			<a href="https://www.nexusmods.com/monsterhunterrise/mods/1092" target="_blank"> (Mod link)</a>
			<div>({{ lm.getString("mod_usage_explanation") }})</div>
		</div>

		<br />

		<a-button @click="getTalismanFile()">{{ lm.getString("register_file") }}</a-button>
		<a-input v-model:value="talismanFilename" placeholder="Talisman list filename (exported via mod)" style="width: 500px" />
		<a-button @click="parseTalismanFile(talismanFilename)" type="primary">{{ lm.getString("manual_refresh") }}</a-button>
		<a-button @click="clearFileTalismans">{{ lm.getString("cancel_register") }}</a-button>

		<a-divider style="border-color: #7cb305" dashed />

		<a-table :columns="columns" :data-source="generateTalismanData(fileTalismans)"
			:pagination="{ defaultPageSize: 100, hideOnSinglePage: true}">
		</a-table>

		<a-divider style="border-color: #7cb305" dashed />

		<div>
			<h1>{{ lm.getString("manually_managed_talisman_explanation") }}</h1>
		</div>
		
		<a-table :columns="addAnomalyColumns" :data-source="dummyAddTalismanSkillSlots()"
			:pagination="{ hideOnSinglePage: true }">
			<template #bodyCell="{ text, column }">
				<template v-if="column.skillId !== undefined">
					<a-select v-model:value="talismanAddInfo.skills[column.skillId].id" @change="onAddTalismanSkillChange(column.skillId)" :options="skillsOptions" show-search filter-option option-filter-prop="label" style="min-width: 200px" >
					</a-select>
					<a-select v-model:value="talismanAddInfo.skills[column.skillId].level"
						:disabled="talismanAddInfo.skills[column.skillId].id === ''">
						<a-select-option :value="0">---</a-select-option>
						<a-select-option v-for="level in getMaxLevel(talismanAddInfo.skills[column.skillId].id)" :value="level">
							{{ level }}
						</a-select-option>
					</a-select>
				</template>
		
				<template v-else-if="column.key === 'slots'">
					<template v-for="slot, index in talismanAddInfo.slotSizes">
						<a-select v-model:value="talismanAddInfo.slotSizes[index]">
							<a-select-option :value="0">0</a-select-option>
							<a-select-option v-for="slotValue in MAX_SLOT_LEVEL" :value="slotValue">{{ slotValue }}
							</a-select-option>
						</a-select>
					</template>
				</template>
		
				<template v-else>
					{{ text }}
				</template>
			</template>
		</a-table>

		<br />

		<div>

		<a-button @click="addManualTalisman()" :disabled="talismanAddInfo.skills.every(skillInfo => skillInfo.id === '')">{{ lm.getString("add_button") }}</a-button>
		
		<a-divider style="border-color: #7cb305" dashed />
		
		<a-popconfirm :title="lm.getString('confirm_delete_all')" ok-text="O" cancel-text="X" @confirm="deleteAllManualTalismans()" @cancel="">
			<a-button>{{ lm.getString("delete_all_button") }}</a-button>
		</a-popconfirm>
		
		</div>
		
		<br />

		<a-table :columns="manualColumns" :data-source="generateTalismanData(manualTalismans)"
			:pagination="{ defaultPageSize: 100, hideOnSinglePage: true}">
			<template #bodyCell="{ index, column }">
				<template v-if="column.key === 'delete'">
					<a-popconfirm :title="lm.getString('confirm_delete')" ok-text="O" cancel-text="X"
						@confirm="deleteManualTalisman(index)" @cancel="">
						<a-button>X</a-button>
					</a-popconfirm>
				</template>
			</template>

		</a-table>
	</div>
</template>

<style scoped>
.logo.vite:hover {
	filter: drop-shadow(0 0 2em #747bff);
}

.logo.vue:hover {
	filter: drop-shadow(0 0 2em #249b73);
}
</style>
