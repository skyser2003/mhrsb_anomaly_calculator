<script setup lang="ts">
// This starter template is using Vue 3 <script setup> SFCs
// Check out https://vuejs.org/api/sfc-script-setup.html#script-setup
import { ref, Ref } from "vue";
import { open } from '@tauri-apps/api/dialog';
import { invoke } from "@tauri-apps/api/tauri";
import type { SelectProps } from "ant-design-vue";


import RawArmorsVec from "../data/armor.json";
import SkillsVec from "../data/skill.json";

import { FinalArmorInfo, ArmorStatInfo, ArmorParts, ArmorFinalSkillInfo, getDefaultStat } from "../definition/armor_define";
import { FinalSkillInfo } from "../definition/skill_define";

import { Language } from "../definition/language";

import uiData from "../ui_data/ui_data.json";
import { EquipSlots, AnomalyArmorInfo, MAX_SLOT_LEVEL } from "../definition/calculate_result";
import { CacheManager } from "../model/data_manager";

const ArmorsVec = RawArmorsVec as unknown as FinalArmorInfo[];
const UIData = uiData as { [key: string]: { [key: string]: string } };

interface AnomalyAddInfo {
	armorId: string;
	skills: {
		id: string;
		level: number;
	}[];
	slots: EquipSlots;
	stat: ArmorStatInfo;
}

function initializeAnomalyAddInfo() {
	anomalyAddInfo.value = { armorId: "", skills: [], slots: [0, 0, 0], stat: getDefaultStat() } as AnomalyAddInfo;

	for (let i = 0; i < maxAnomalySkills.value; ++i) {
		anomalyAddInfo.value.skills.push({ id: "", level: 0 });
	}

	anomalyAddStatDiff.value = anomalyAddInfo.value.stat;
}

const props = defineProps<{ langData: Language }>();

defineExpose({ getFileAnomalies });

const columns = ref([
	{
		title: "Unused",
		dataIndex: "name",
		key: "name",
		width: 200,
	},
	{
		title: "Skill 1",
		dataIndex: "skill1",
		key: "skill1",
		width: 200,
	},
	{
		title: "Skill 2",
		dataIndex: "skill2",
		key: "skill2",
		width: 200,
	},
	{
		title: "Skill 3",
		dataIndex: "skill3",
		key: "skill3",
		width: 200,
	},
	{
		title: "Skill 4",
		dataIndex: "skill4",
		key: "skill4",
		width: 200,
	},
	{
		title: "Skill 5",
		dataIndex: "skill5",
		key: "skill5",
		width: 200,
	},
	{
		title: UIData["slots_name"][props.langData],
		dataIndex: "slots",
		key: "slots",
	}
]);

const manualColumns = ref([
	{
		title: "Unused",
		dataIndex: "name",
		key: "name",
		width: 200,
	},
	{
		title: "Skill 1",
		dataIndex: "skill1",
		key: "skill1",
		width: 200,
	},
	{
		title: "Skill 2",
		dataIndex: "skill2",
		key: "skill2",
		width: 200,
	},
	{
		title: "Skill 3",
		dataIndex: "skill3",
		key: "skill3",
		width: 200,
	},
	{
		title: "Skill 4",
		dataIndex: "skill4",
		key: "skill4",
		width: 200,
	},
	{
		title: "Skill 5",
		dataIndex: "skill5",
		key: "skill5",
		width: 200,
	},
	{
		title: UIData["slots_name"][props.langData],
		dataIndex: "slots",
		key: "slots",
	},
	{
		title: UIData["delete"][props.langData],
		dataIndex: "delete",
		key: "delete",
	}
]);


const addAnomalyColumns = ref([
	{
		title: UIData["armor_name"][props.langData],
		dataIndex: "name",
		key: "name",
		width: 200,
	},
	{
		title: "Skill 1",
		dataIndex: "skill1",
		key: "skill1",
		skillId: 0,
		width: 200,
	},
	{
		title: "Skill 2",
		dataIndex: "skill2",
		key: "skill2",
		skillId: 1,
		width: 200,
	},
	{
		title: "Skill 3",
		dataIndex: "skill3",
		key: "skill3",
		skillId: 2,
		width: 200,
	},
	{
		title: "Skill 4",
		dataIndex: "skill4",
		key: "skill4",
		skillId: 3,
		width: 200,
	},
	{
		title: "Skill 5",
		dataIndex: "skill5",
		key: "skill5",
		skillId: 4,
		width: 200,
	},
	{
		title: UIData["slot_diffs"][props.langData],
		dataIndex: "slots",
		key: "slots",
	}
]);

const statDiffColumns = ref([
	{
		title: UIData["defense"][props.langData],
		dataIndex: "defense",
		key: "defense",
	},
	{
		title: UIData["fire_res"][props.langData],
		dataIndex: "fireRes",
		key: "fireRes",
	},
	{
		title: UIData["water_res"][props.langData],
		dataIndex: "waterRes",
		key: "waterRes",
	},
	{
		title: UIData["ice_res"][props.langData],
		dataIndex: "iceRes",
		key: "iceRes",
	},
	{
		title: UIData["elec_res"][props.langData],
		dataIndex: "elecRes",
		key: "elecRes",
	},
	{
		title: UIData["dragon_res"][props.langData],
		dataIndex: "dragonRes",
		key: "dragonRes",
	},
]);

let skills = ref<{ [key: string]: FinalSkillInfo }>({});

let skillsVec = ref<FinalSkillInfo[]>(SkillsVec);
const armorsVec = ref<FinalArmorInfo[]>([]);

for (const armor of ArmorsVec) {
	if (7 <= armor.rarity) {
		armorsVec.value.push(armor);
	}
}

skillsVec.value.sort((elem1, elem2) => elem1.names[props.langData] > elem2.names[props.langData] ? 1 : -1);
armorsVec.value.sort((elem1, elem2) => elem1.names[props.langData] > elem2.names[props.langData] ? 1 : -1);

for (const skill of skillsVec.value) {
	skills.value[skill.id] = skill;
}

const parts = ref(ArmorParts);

let armorsByPart = ref({} as { [key: string]: { [key: string]: FinalArmorInfo } });
let armorsByPartVec = ref({} as { [key: string]: FinalArmorInfo[] });

for (const part of parts.value) {
	armorsByPart.value[part] = {};
	armorsByPartVec.value[part] = [];
}

for (const armor of armorsVec.value) {
	const id = armor.id;
	const part = armor.part;

	armorsByPart.value[part][id] = armor;
	armorsByPartVec.value[part].push(armor);
}

for (const part in armorsByPartVec.value) {
	const subArmors = armorsByPartVec.value[part];
	subArmors.sort((elem1, elem2) => elem1.names[props.langData] > elem2.names[props.langData] ? 1 : -1);
}

const armorsOptions = ref<Exclude<SelectProps["options"], undefined>>([{
	value: "",
	label: "---"
}]);

for(let i = 0; i < armorsVec.value.length; ++i) {
	const armor = armorsVec.value[i];

	armorsOptions.value.push({
		key: i,
		value: armor.id,
		label: armor.names[props.langData],
	});
}

const anomaly_filename = ref("");

const anomalyArmors = ref<AnomalyArmorInfo[]>([]);

const anomalyArmorsByPart = ref<{ [key: string]: AnomalyArmorInfo[] }>({});
const manualAnomaliesByPart = ref<{ [key: string]: AnomalyArmorInfo[] }>({});

const maxAnomalySkills = ref(5);
const maxStatDiff = ref<{[key: string]: number}>({
	defense: 50,
	fireRes: 20,
	waterRes: 20,
	iceRes: 20,
	elecRes: 20,
	dragonRes: 20,
});

const anomalyAddInfo = ref<AnomalyAddInfo>({} as AnomalyAddInfo);
const anomalyAddStatDiff = ref<{ [key: string]: number }>(anomalyAddInfo.value.stat);

initializeAnomalyAddInfo();

for (const part of parts.value) {
	anomalyArmorsByPart.value[part] = [];
	manualAnomaliesByPart.value[part] = [];
}

const anomalyFilename = CacheManager.getAnomalyFilename();

if (anomalyFilename) {
	anomaly_filename.value = anomalyFilename;
	getFileAnomalies();
}

await loadManualAnomalies();

async function loadAnomalyFile() {
	const file = await open({
		multiple: false,
		directory: false,
		filters: [{
			name: "anomaly_crafting_list",
			extensions: ["txt"]
		}]
	});

	if (file !== null && !Array.isArray(file)) {
		anomaly_filename.value = file;

		parseAnomalyFile(file);
	}
}

async function getFileAnomalies() {
	const fileAnomalies = await invoke("cmd_get_file_anomalies") as AnomalyArmorInfo[];
	
	setFileAnomalyData(fileAnomalies);
}

async function clearFileAnomalies() {
	anomaly_filename.value = "";
	await invoke("cmd_clear_file_anomalies");

	CacheManager.setAnomalyFilename("");
	setFileAnomalyData([]);
}

async function loadManualAnomalies() {
	manualAnomaliesByPart.value = CacheManager.getManualAnomalies();

	const result = await invoke("cmd_set_manual_anomalies", { anomalies: manualAnomaliesByPart.value });

	if (result === true) {
		console.log("Manual anomlay load successful");
	}
}

async function parseAnomalyFile(filename: string) {
	console.log(`Anomaly filename: ${filename}`);

	const fileAnomalies = await invoke("cmd_parse_anomaly", { filename }) as AnomalyArmorInfo[];
	CacheManager.setAnomalyFilename(filename);

	setFileAnomalyData(fileAnomalies);
}

async function setFileAnomalyData(anomalies: AnomalyArmorInfo[]) {
	anomalyArmors.value = [];
	
	for (const part of parts.value) {
		anomalyArmorsByPart.value[part] = [];
	}

	anomalyArmors.value = anomalies;
	anomalyArmors.value.sort((armor1, armor2) => armor1.original.names[props.langData] > armor2.original.names[props.langData] ? 1 : -1);

	for (const armor of anomalyArmors.value) {
		const part = armor.original.part;

		if (anomalyArmorsByPart.value[part] === undefined) {
			anomalyArmorsByPart.value[part] = [];
		}

		anomalyArmorsByPart.value[part].push(armor);
	}

	for (const armor of anomalyArmors.value) {
		maxAnomalySkills.value = Math.max(maxAnomalySkills.value, Object.keys(armor.skillDiffs).length);
	}

	for (let i = anomalyAddInfo.value.skills.length; i < maxAnomalySkills.value; ++i) {
		anomalyAddInfo.value.skills.push({ id: "", level: 0 });
	}
}

function generateAnomalyData(anomaliesByPart: { [key: string]: AnomalyArmorInfo[] }, part: string) {
	const ret = [];

	for (const armor of anomaliesByPart[part]) {
		const name = armor.original.names[props.langData];
		const skillTexts = [];

		for (const skillId in armor.skillDiffs) {
			const skillInfo = armor.skillDiffs[skillId];

			const skillName = skills.value[skillId].names[props.langData];
			const level = skillInfo.level;
			const absLevel = Math.abs(level);

			const levelText = level > 0 ? `Lv${absLevel}` : `-Lv${absLevel}`;

			const text = `${skillName} ${levelText}`;
			skillTexts.push(text);
		}

		for (let i = skillTexts.length; i < maxAnomalySkills.value; ++i) {
			skillTexts.push("");
		}

		const finalSlots = [];

		for(let i=0; i < armor.original.slots.length; ++i) {
			const oriSlot = armor.original.slots[i];
			const diffSlot = armor.slotDiffs[i];

			if (diffSlot === 0) {
				finalSlots.push(`${oriSlot}`);
			} else {
				finalSlots.push(`${oriSlot + diffSlot}(+${diffSlot})`);
			}
		}

		const finalSlotsText = `[${finalSlots.join(", ")}]`;

		ret.push({
			name: name,
			skill1: skillTexts[0],
			skill2: skillTexts[1],
			skill3: skillTexts[2],
			skill4: skillTexts[3],
			skill5: skillTexts[4],
			slots: finalSlotsText,
		});
	}

	return ret;
}

async function addManualAnomalyArmor() {
	console.log(anomalyAddInfo.value);

	const inserted = await invoke("cmd_add_manual_anomaly", {
		originalId: anomalyAddInfo.value.armorId,
		skillDiffs: anomalyAddInfo.value.skills,
		slotDiffs: anomalyAddInfo.value.slots,
		statDiff: anomalyAddInfo.value.stat,
	}) as AnomalyArmorInfo;

	console.log(`Add manual result: `, inserted);

	if (inserted !== null) {
		manualAnomaliesByPart.value[inserted.original.part].push(inserted);

		CacheManager.setManualAnomalies(manualAnomaliesByPart.value);

		initializeAnomalyAddInfo();
	}
}

async function deleteManualAnomaly(part: string, index: number) {
	const anomaly = manualAnomaliesByPart.value[part][index];
	
	manualAnomaliesByPart.value[part].splice(index, 1);
	CacheManager.setManualAnomalies(manualAnomaliesByPart.value);

	const result = await invoke("cmd_set_manual_anomalies", { anomalies: manualAnomaliesByPart.value });

	if (result === true) {
		console.log("Anomaly deleted: ", anomaly);
	}
}

function dummyStatDiffData() {
	return [getDefaultStat()];
}

function dummyAddAnomalySkillSlots() {
	return [{
		name: "",
		skill1: "",
		skill2: "",
		skill3: "",
		skill4: "",
		skill5: "",
		slots: "",
	}];
}

function getMaxLevel(skillId: string) {
	const skillInfo = skills.value[skillId];

	if (skillInfo === undefined) {
		return 0;
	} else {
		return skillInfo.maxLevel;
	}
}

function onAddAnomalySkillChange(index: number) {
	anomalyAddInfo.value.skills[index].level = 0;
}

</script>

<template>
	<div class="container">
		<div>
			<h1 style="display: inline-block;">{{ UIData["mod_managed_anomaly_explanation"][props.langData] }} </h1>
			<a href="https://www.nexusmods.com/monsterhunterrise/mods/1477" target="_blank">(Mod link)</a>
		</div>

		<a-button @click="loadAnomalyFile()">Load anomaly file</a-button>
		<a-input v-model:value="anomaly_filename" placeholder="Anomaly crafting filename (exported via mod)" style="width: 500px" />
		<a-button @click="parseAnomalyFile(anomaly_filename)" type="primary">Parse Anomaly</a-button>
		<a-button @click="clearFileAnomalies">Clear</a-button>

		<template v-for="part in parts">
			<a-table :columns="columns" :data-source="generateAnomalyData(anomalyArmorsByPart, part)"
				:pagination="{ defaultPageSize: 10000000, hideOnSinglePage: true}">
				<template #headerCell="{ column }">
					<template v-if="column.dataIndex === 'name'">
						{{ UIData[`${part}_name`][props.langData] }}
					</template>
					<template v-else>
						{{ column.title }}
					</template>
				</template>
			</a-table>
			<a-divider style="border-color: #7cb305" dashed />
		</template>

		<div>
			<h1>{{ UIData["manually_managed_anomaly_explanation"][props.langData] }}</h1>
		</div>

		<a-table :columns="addAnomalyColumns" :data-source="dummyAddAnomalySkillSlots()" :pagination="{ hideOnSinglePage: true }">
			<template #bodyCell="{ text, column }">
				<template v-if="column.key === 'name'">
					<select :name="`armor_select`" v-model="anomalyAddInfo.armorId">
						<option value="">---</option>
						<option v-for="armorInfo in armorsVec" :value="armorInfo.id">
							{{ armorInfo.names[props.langData] }}
						</option>
					</select>
				</template>

				<template v-else-if="column.skillId !== undefined">
					<select v-model="anomalyAddInfo.skills[column.skillId].id" @change="onAddAnomalySkillChange(column.skillId)">
						<option value="">---</option>
						<option v-for="skillInfo in skillsVec" :value="skillInfo.id">
							{{ skillInfo.names[langData] }}
						</option>
					</select>
					<a-select v-model:value="anomalyAddInfo.skills[column.skillId].level" :disabled="anomalyAddInfo.skills[column.skillId].id === ''">
						<a-select-option v-for="level in getMaxLevel(anomalyAddInfo.skills[column.skillId].id)" :value="-(getMaxLevel(anomalyAddInfo.skills[column.skillId].id) - level + 1)">
							{{ -(getMaxLevel(anomalyAddInfo.skills[column.skillId].id) - level + 1) }}
						</a-select-option>
						<a-select-option :value="0">---</a-select-option>
						<a-select-option v-for="level in getMaxLevel(anomalyAddInfo.skills[column.skillId].id)" :value="level">
							+{{ level }}
						</a-select-option>
					</a-select>
				</template>
				
				<template v-else-if="column.key === 'slots'">
					<template v-for="slot, index in anomalyAddInfo.slots">
						<a-select v-model:value="anomalyAddInfo.slots[index]">
							<a-select-option  :value="0">0</a-select-option>
							<a-select-option v-for="slotValue in MAX_SLOT_LEVEL" :value="slotValue">+{{ slotValue }}</a-select-option>
						</a-select>
					</template>
				</template>

				<template v-else>
					{{ text }}
				</template>
			</template>
		</a-table>

		<a-table :columns="statDiffColumns" :data-source="dummyStatDiffData()" :pagination="{ hideOnSinglePage: true }">
			<template #bodyCell="{ column }">
				<a-select v-model:value="anomalyAddStatDiff[column.key]">
					<a-select-option v-for="diff in maxStatDiff[column.key]" :value="(diff - maxStatDiff[column.key] - 1)">{{ diff - maxStatDiff[column.key] - 1 }}
					</a-select-option>
					<a-select-option :value="0">0</a-select-option>
					<a-select-option v-for="diff in maxStatDiff[column.key]" :value="diff">{{ diff }}</a-select-option>
				</a-select>
			</template>
		</a-table>

		<a-button @click="addManualAnomalyArmor()" :disabled="anomalyAddInfo.armorId.length === 0">Add</a-button>

		<a-divider style="border-color: #7cb305" dashed />

		<template v-for="part in parts">
			<a-table :columns="manualColumns" :data-source="generateAnomalyData(manualAnomaliesByPart, part)"
				:pagination="{ defaultPageSize: 10000000, hideOnSinglePage: true}">
				<template #headerCell="{ column }">
					<template v-if="column.dataIndex === 'name'">
						{{ UIData[`${part}_name`][props.langData] }}
					</template>
					<template v-else>
						{{ column.title }}
					</template>
				</template>

				<template #bodyCell="{ index, column }">
					<template v-if="column.key === 'delete'">
						<a-popconfirm :title="UIData['confirm_delete'][langData]" ok-text="O" cancel-text="X" @confirm="deleteManualAnomaly(part, index)"
							@cancel="">
							<a-button>X</a-button>
						</a-popconfirm>
					</template>
				</template>

			</a-table>
			<a-divider style="border-color: #7cb305" dashed />
		</template>
	</div>
</template>

<style>
.logo.vite:hover {
	filter: drop-shadow(0 0 2em #747bff);
}

.logo.vue:hover {
	filter: drop-shadow(0 0 2em #249b73);
}

#armorSelectContainer {
	width: max-content;
	min-width: 100%;
	text-overflow: 100%;
}

#armorSelectContainer .ant-select-item-option-content {
	width: max-content;
	display: inline-block;
	text-overflow: clip;
	min-width: 100%;
}
</style>
