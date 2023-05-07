<script setup lang="ts">
import { ref, nextTick } from "vue";
import { invoke } from "@tauri-apps/api/tauri";

import { SmileOutlined } from '@ant-design/icons-vue';

import SkillCategories from "../data/skill_category.json";
import SkillsVec from "../data/skill.json";

import { SkillCategory } from "../definition/skill_category_define";
import { FinalSkillInfo } from "../definition/skill_define";
import { CalculateResult, SearchFavorite, EquipSlots, Skills, MinMaxSkills, Slots, ResultFavorite, SexType, CalcChoices, ResultFullEquipments, ResultArmor, CalculateAdditionalSkillsResult } from "../definition/calculate_result";
import { CacheManager } from "../model/data_manager";

import SimulateResultTable from "./SimulateResultTable.vue"; 
import AdditionalSkillsTable from "./AdditionalSkillsTable.vue";

import { lm } from "../model/language_manager";
import { Language } from "../definition/language";
import { InvokeManager } from "../model/invoke_manager";

enum CalcState {
	IDLE,
	CALCULATING_COMBINATION,
	CALCULATING_ADDITIONAL_SKILLS,
	DONE_COMBINATION,
	DONE_ADDITIONAL_SKILLS,
}

const free_slot_options = ref([
	{
		value: 0,
		label: 0
	},
	{
		value: 1,
		label: 1
	},
	{
		value: 2,
		label: 2
	},
	{
		value: 3,
		label: 3
	},
	{
		value: 4,
		label: 4
	},
	{
		value: 5,
		label: 5
	},
	{
		value: 6,
		label: 6
	},
	{
		value: 7,
		label: 7
	},
	{
		value: 8,
		label: 8
	},
	{
		value: 9,
		label: 9
	},
	{
		value: 10,
		label: 10
	}
]);

const props = defineProps<{
	langData: Language
}>();

const emits = defineEmits<{
	(event: "add_search_favorite", fav: SearchFavorite): void,
	(event: "add_result_favorite", fav: ResultFavorite): void
}>();

defineExpose({setSearchCondition});

const skillCats = ref<SkillCategory[]>(SkillCategories);
const skillsVec = ref<FinalSkillInfo[]>(SkillsVec);

const skills = ref<{ [key: string]: FinalSkillInfo }>({});

const sexType = ref<SexType>("");
const weaponSlots = ref<EquipSlots>([0, 0, 0]);
const selectedSkills = ref<Skills>({});
const freeSlots = ref<Slots>([0, 0, 0, 0]);
const includeLteEquips = ref(false);

const resultSortKey = ref("slots_sum");
const calc_state = ref(CalcState.IDLE);

const calcResult = ref<CalculateResult>({ fullEquipments: [], calcTime: 0 });
const resultEquipmentsCount = ref(0);

const additionalSkills = ref<MinMaxSkills>({});
const additionalSlots = ref<Slots>([0, 0, 0, 0]);
const originalSkills = ref<Skills>({});
const originalSlots = ref<Slots>([0, 0, 0, 0]);

for (const skill of skillsVec.value) {
	skills.value[skill.id] = skill;
	selectedSkills.value[skill.id] = 0;
}

try {
	const calcInput = CacheManager.getCalcChoices();
	sexType.value = calcInput.sexType;
	weaponSlots.value = calcInput.weaponSlots;
	freeSlots.value = calcInput.freeSlots;

	for (const skillId in calcInput.selectedSkills) {
		let level = calcInput.selectedSkills[skillId];

		if (0 <= level) {
			selectedSkills.value[skillId] = level;
		}
	}
} catch(e) {
	console.error(e);
}

const yesCategorySkills = {} as { [key: string]: boolean };
const noCategorySkills = [];

for (const cat of skillCats.value) {
	for(const skillId of cat.skills) {
		yesCategorySkills[skillId] = true;
	}
}

for (const skillId in skills.value) {
	if (yesCategorySkills[skillId] === undefined) {
		noCategorySkills.push(skillId);
	}
}

if (0 < noCategorySkills.length) {
	skillCats.value.push({
		id: "no_category",
		names: { "en": "no_category", "ko": "no_category" },
		skills: noCategorySkills
	});
}

for (const cat of skillCats.value) {
	cat.skills.sort((id1, id2) => {
		const skill1 = skills.value[id1];
		const skill2 = skills.value[id2];

		return skill1.names[props.langData] > skill2.names[props.langData] ? 1 : -1;
	});
}

async function loadManuals() {
	console.log("Loading manuals...");

	const manualAnomalies = CacheManager.getManualAnomalies();
	const manualTalismans = CacheManager.getManualTalismans();

	const proms = [
		InvokeManager.setManualAnomalies(manualAnomalies),
		InvokeManager.setManualTalismans(manualTalismans)
	];

	const [result1, result2] = await Promise.all(proms);

	if (result1 === false) {
		console.error("Manual anomaly load failed");
	}

	if (result2 === false) {
		console.error("Manual talisman load failed");
	}

	console.log("Manual anomaly/talisman loading done");
}


async function calculate() {
	const localSelectedSkills = {} as { [key: string]: number };

	for (const skillId in selectedSkills.value) {
		if (skills.value[skillId] === undefined) {
			delete selectedSkills.value[skillId];
			continue;
		}

		let level = selectedSkills.value[skillId];

		if (level !== 0) {
			localSelectedSkills[skillId] = level;
		}
	}

	const calcInput: CalcChoices = {
		sexType: sexType.value,
		weaponSlots: weaponSlots.value,
		selectedSkills: localSelectedSkills,
		freeSlots: freeSlots.value,
	};

	CacheManager.setCalcChoices(calcInput);

	console.log(calcInput);

	calc_state.value = CalcState.CALCULATING_COMBINATION;
	calcResult.value.calcTime = 0;
	calcResult.value.fullEquipments = [];
	resultEquipmentsCount.value = 0;

	await loadManuals();

	try {
		const result = await InvokeManager.calculateSkillset(
			CacheManager.getAnomalyFilename(),
			CacheManager.getTalismanFilename(),
			calcInput.sexType,
			calcInput.weaponSlots,
			calcInput.selectedSkills,
			calcInput.freeSlots,
			includeLteEquips.value
		);

		const localCalcResult = result["result"] as CalculateResult;

		if (resultSortKey.value !== "slots_sum") {
			sortResult(resultSortKey.value, localCalcResult);
		}

		calcResult.value = localCalcResult;
		resultEquipmentsCount.value = calcResult.value.fullEquipments.length;

		console.log(result);
	} catch (e) {
		console.error("cmd_calculate_skillset failed, ", e);
	}

	calc_state.value = CalcState.DONE_COMBINATION;
}

async function calculateAdditionalSkills() {
	const localSelectedSkills = {} as { [key: string]: number };

	for (const skillId in selectedSkills.value) {
		if (skills.value[skillId] === undefined) {
			delete selectedSkills.value[skillId];
			continue;
		}

		let level = selectedSkills.value[skillId];

		if (level !== 0) {
			localSelectedSkills[skillId] = level;
		}
	}

	const calcInput: CalcChoices = {
		sexType: sexType.value,
		weaponSlots: weaponSlots.value,
		selectedSkills: localSelectedSkills,
		freeSlots: freeSlots.value,
	};

	CacheManager.setCalcChoices(calcInput);

	console.log(calcInput);

	calc_state.value = CalcState.CALCULATING_ADDITIONAL_SKILLS;
	resultEquipmentsCount.value = 0;
	calcResult.value.calcTime = 0;
	calcResult.value.fullEquipments = [];
	additionalSkills.value = {};
	additionalSlots.value = [0, 0, 0, 0];

	await loadManuals();

	try {
		const result = await InvokeManager.calculateAdditionalSkillset(
			CacheManager.getAnomalyFilename(),
			CacheManager.getTalismanFilename(),
			calcInput.sexType,
			calcInput.weaponSlots,
			calcInput.selectedSkills,
			calcInput.freeSlots,
			includeLteEquips.value,
		);

		const sortedKeys = Object.keys(result.skills).sort((id1, id2) => {
			const skill1 = skills.value[id1];
			const skill2 = skills.value[id2];

			return skill1.names[props.langData] > skill2.names[props.langData] ? 1 : -1;
		});

		resultEquipmentsCount.value = result.equipmentsCount;
		additionalSkills.value = {};
		additionalSlots.value = result.slots;

		for (const skillId of sortedKeys) {
			additionalSkills.value[skillId] = result.skills[skillId];
		}

		calcResult.value.calcTime = result.calcTime;
		originalSkills.value = JSON.parse(JSON.stringify(selectedSkills.value));
		originalSlots.value = JSON.parse(JSON.stringify(freeSlots.value));

		console.log(result);
	} catch (e) {
		console.error("cmd_calculate_additional_skills failed, ", e);
	}

	calc_state.value = CalcState.DONE_ADDITIONAL_SKILLS;
}

function clear() {
	sexType.value = "";
	selectedSkills.value = {};
	weaponSlots.value = [0, 0, 0];
	freeSlots.value = [0, 0, 0, 0];

	for (const skillId in skills.value) {
		selectedSkills.value[skillId] = 0;
	}

	CacheManager.setEmptyCalcChoices();

	calcResult.value.calcTime = 0;
	calcResult.value.fullEquipments = [];
}

function addFavorite() {
	const validSkills = {} as Skills;

	for (const id in selectedSkills.value) {
		const level = selectedSkills.value[id];

		if (level === 0) {
			continue;
		}

		validSkills[id] = level;
	}

	const ret = {} as SearchFavorite;
	ret.name = "";
	ret.sexType = sexType.value;
	ret.weaponSlots = JSON.parse(JSON.stringify(weaponSlots.value));
	ret.reqSkills = validSkills;
	ret.reqSlots = JSON.parse(JSON.stringify(freeSlots.value));

	emits("add_search_favorite", ret);
}

async function setSearchCondition(fav: SearchFavorite) {
	sexType.value = fav.sexType;
	weaponSlots.value = JSON.parse(JSON.stringify(fav.weaponSlots));
	selectedSkills.value = JSON.parse(JSON.stringify(fav.reqSkills));
	freeSlots.value = JSON.parse(JSON.stringify(fav.reqSlots));

	const calcInput: CalcChoices = {
		sexType: sexType.value,
		weaponSlots: weaponSlots.value,
		selectedSkills: selectedSkills.value,
		freeSlots: freeSlots.value,
	};

	CacheManager.setCalcChoices(calcInput);

	for (const skill of skillsVec.value) {
		if (selectedSkills.value[skill.id] === undefined) {
			selectedSkills.value[skill.id] = 0;
		}
	}

	await nextTick();

	const elem = document.getElementById("calculate_button")!;
	window.scrollTo(0, elem.offsetTop);
}

function addResultFavorite(fav: ResultFavorite) {
	emits("add_result_favorite", fav);
}

function canSubmit() {
	return sexType.value !== "" && (calc_state.value === CalcState.IDLE || calc_state.value === CalcState.DONE_COMBINATION || calc_state.value === CalcState.DONE_ADDITIONAL_SKILLS);
}

const calcSlots = (equip: ResultFullEquipments) => {
	let maxSlots = 0;

	for (const comb of equip.decoCombs) {
		let slots = comb.leftoverSlotsSum.reduce((a, b) => a + b, 0);

		maxSlots = Math.max(maxSlots, slots);
	}

	return maxSlots;
};

function sortBySlotsSum(equip1: ResultFullEquipments, equip2: ResultFullEquipments) {
	const slots1 = calcSlots(equip1);
	const slots2 = calcSlots(equip2);

	return slots1 > slots2 ? -1 : 1;
}

function sortByDefense(equip1: ResultFullEquipments, equip2: ResultFullEquipments) {
	let defense1 = 0;
	let defense2 = 0;

	for (const key in equip1.armors) {
		defense1 += equip1.armors[key].stat.defense;
	}

	for (const key in equip2.armors) {
		defense2 += equip2.armors[key].stat.defense;
	}

	return defense1 > defense2 ? -1 : 1;
}

function sortResult(sortKey: string, calcResultData: CalculateResult) {
	const sortFunc = sortKey === "slots_sum" ? sortBySlotsSum : sortByDefense;

	calcResultData.fullEquipments.sort(sortFunc);
}

</script>

<template>
	<div id="remote_controller">
		<table class="ant-table">
			<thead class="ant-table-thead">
				<tr class="ant-table-row">
					<th class="ant-table-cell">{{ lm.getString("remote_controller") }}</th>
				</tr>
			</thead>
			<tbody class="ant-table-tbody">
				<tr class="ant-table-row">
					<td class="ant-table-cell"><a href="#top">{{ lm.getString("controller_to_top") }}</a></td>
				</tr>
				<tr class="ant-table-row">
					<td class="ant-table-cell"><a href="#calculate_button">{{ lm.getString("controller_to_calc_button") }}</a></td>
				</tr>
			</tbody>
		</table>
	</div>

	<table v-for="cat in skillCats">
		<tr>
			{{ cat.names[langData] }}
		</tr>
		<tr>
			<div>
				<div style="display: inline-block; width: 200px; height: 50px; margin: 10px;" v-for="id in cat.skills">
					<div>
						{{ skills[id].names[langData] }}
					</div>
					<div>
						<a-select v-model:value="selectedSkills[id]" :style="`width: 120px; ${selectedSkills[id] === 0 ? '' : 'border: dashed blue;'} `">
							<a-select-option :value="0" selected>---</a-select-option>
							<a-select-option v-for="level in skills[id].maxLevel" :value="level">
								Lv {{ level }}
							</a-select-option>
							<template #suffixIcon>
								<smile-outlined class="ant-select-suffix" />
							</template>
						</a-select>
					</div>
				</div>
			</div>
			<br />
		</tr>
	</table>
	
	<table>
		<tr>
			<td>{{ lm.getString("free_slots") }}</td>
			<td>
				<a-select v-model:value="freeSlots[0]" style="width: 120px" :options="free_slot_options">
					<template #suffixIcon>
						<smile-outlined class="ant-select-suffix" />
					</template>
				</a-select>
			</td>
			<td>
				<a-select v-model:value="freeSlots[1]" style="width: 120px" :options="free_slot_options">
					<template #suffixIcon>
						<smile-outlined class="ant-select-suffix" />
					</template>
				</a-select>
			</td>
			<td>
				<a-select v-model:value="freeSlots[2]" style="width: 120px" :options="free_slot_options">
					<template #suffixIcon>
						<smile-outlined class="ant-select-suffix" />
					</template>
				</a-select>
			</td>
			<td>
				<a-select v-model:value="freeSlots[3]" style="width: 120px" :options="free_slot_options">
					<template #suffixIcon>
						<smile-outlined class="ant-select-suffix" />
					</template>
				</a-select>
			</td>
		</tr>
	</table>

	<br />

	<table>
		<tr>
			<td style="width: 100px">{{ lm.getString("sex_type") }}</td>
			<td>
				<a-select v-model:value="sexType" style="width: 100px">
					<a-select-option value="">---</a-select-option>
					<a-select-option value="male">{{ lm.getString("male") }}</a-select-option>
					<a-select-option value="female">{{ lm.getString("female") }}</a-select-option>
				</a-select>
			</td>
		</tr>
		<tr>
			<td colspan="2">
				<br />
			</td>
		</tr>
		<tr>
			<td>
				{{ lm.getString("weapon_slots") }}
			</td>
		</tr>
		<tr>
			<td>Slot 1</td>
			<td>
				<a-radio-group v-model:value="weaponSlots[0]" button-style="solid">
					<a-radio-button v-for="level in 5" :value="level - 1">{{ level - 1 }}</a-radio-button>
				</a-radio-group>
			</td>
		</tr>
		<tr>
			<td>Slot 2</td>
			<td>
				<a-radio-group v-model:value="weaponSlots[1]" button-style="solid">
					<a-radio-button v-for="level in 5" :value="level - 1">{{ level - 1 }}</a-radio-button>
				</a-radio-group>
			</td>
		</tr>
		<tr>
			<td>Slot 3</td>
			<td>
				<a-radio-group v-model:value="weaponSlots[2]" button-style="solid">
					<a-radio-button v-for="level in 5" :value="level - 1">{{ level - 1 }}</a-radio-button>
				</a-radio-group>
			</td>
		</tr>
	</table>

	<a-divider style="border-color: #7cb305" dashed />

	<a-checkbox v-model:checked="includeLteEquips">{{ lm.getString("include_lte_equips") }}</a-checkbox>
	
	<br />
	<br />

	<a-button @click="calculate" :disabled="canSubmit() === false" :type="canSubmit() === true ? 'primary' : 'dashed'" id="calculate_button" >
		{{ lm.getString("calculate_button") }}
	</a-button>
	<a-button @click="calculateAdditionalSkills" :disabled="canSubmit() === false" :type="canSubmit() === true ? 'primary' : 'dashed'" style="margin-left: 10px" >
		{{ lm.getString("calculate_additional_skills_button") }}
	</a-button>

	<br />
	<br />

	<a-button @click="clear">{{ lm.getString("clear_search_condition") }}</a-button>

	<br />
	<br />

	<a-button @click="addFavorite" :disabled="canSubmit() === false">{{ lm.getString("save_search_favorite") }}</a-button>

	<a-divider style="border-color: #7cb305" dashed />

	<table>
		<tr>
			<td>{{ lm.getString("result_count") }}</td>
			<td>{{ resultEquipmentsCount }}</td>
		</tr>
		<tr>
			<td>{{ lm.getString("calc_time") }}</td>
			<td>{{ calcResult.calcTime }} sec</td>
		</tr>
		<tr>
			<td style="width: 100px">{{ lm.getString("sort_result_criteria") }}</td>
			<td>
				<a-select v-model:value="resultSortKey" @change="sortResult(resultSortKey, calcResult)" style="min-width: 200px">		
					<a-select-option value="slots_sum">{{ lm.getString("slots_sum") }}</a-select-option>
					<a-select-option value="defense">{{ lm.getString("defense") }}</a-select-option>
				</a-select>
			</td>
		</tr>
	</table>

	<br />

	<template v-if="calc_state === CalcState.IDLE || calc_state === CalcState.DONE_COMBINATION">
		<SimulateResultTable :langData="langData" :calcResult="calcResult" v-on:add_result_favorite="addResultFavorite" />
	</template>
	<template v-else-if="calc_state === CalcState.DONE_ADDITIONAL_SKILLS">
		<template v-if="200 <= resultEquipmentsCount">
			<div>{{ lm.getString("additional_skills_excess_200") }}</div>
		</template>
		<template v-else-if="resultEquipmentsCount === 0">
			<div>{{ lm.getString("additional_skills_not_found") }}</div>
		</template>
		<template v-else>
			<AdditionalSkillsTable :langData="langData" :skills="additionalSkills" :slots="additionalSlots" :selectedSkills="selectedSkills" :selectedSlots="freeSlots" :originalSkills="originalSkills" :originalSlots="originalSlots" />
		</template>
	</template>
	<template v-else>
		<a-spin size="large" />
	</template>
</template>

<style scoped>

#remote_controller {
	position: fixed;

	right: 10px;
	bottom: 10px;

	z-index: 999;
}

#remote_controller a {
	display: block;
}

#remote_controller .ant-table-cell {
	padding: 10px;
}

</style>