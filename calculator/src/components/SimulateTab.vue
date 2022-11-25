<script setup lang="ts">
import { ref, Ref } from "vue";
import { invoke } from "@tauri-apps/api/tauri";

import { SmileOutlined } from '@ant-design/icons-vue';

import SkillCategories from "../data/skill_category.json";
import SkillsVec from "../data/skill.json";

import { SkillCategory } from "../definition/skill_category_define";
import { FinalSkillInfo } from "../definition/skill_define";
import { CalculateResult, SearchFavorite, EquipSlots, Skills, Slots, ResultFavorite, SexType, CalcChoices } from "../definition/calculate_result";
import { CacheManager } from "../model/data_manager";

import SimulateResultTable from "./SimulateResultTable.vue"; 

import UIData from "../ui_data/ui_data.json";
import { Language } from "../definition/language";

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

const skillCats = ref(SkillCategories) as Ref<SkillCategory[]>;
const skillsVec = ref(SkillsVec) as Ref<FinalSkillInfo[]>;

const skills = ref({}) as Ref<{ [key: string]: FinalSkillInfo }>;

const sexType = ref<SexType>("");
const weaponSlots = ref([0, 0, 0]) as Ref<EquipSlots>;
const selectedSkills = ref({}) as Ref<Skills>;
const freeSlots = ref([0, 0, 0, 0]) as Ref<Slots>;
const includeLteEquips = ref(false);

const is_calculating = ref(false);

const calcResult = ref({ full_equipments: [], calc_time: 0 } as CalculateResult) as Ref<CalculateResult>;

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

const anomalyFilename = CacheManager.getAnomalyFilename();
const talismanFilename = CacheManager.getTalismanFilename();

if (anomalyFilename !== null) {
	await invoke("cmd_parse_anomaly", { filename: anomalyFilename });
}

if (talismanFilename !== null) {
	await invoke("cmd_parse_talisman", { filename: talismanFilename });
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
		includeLteEquips: includeLteEquips.value,
	};

	CacheManager.setCalcChoices(calcInput);

	console.log(calcInput);

	is_calculating.value = true;
	calcResult.value.calc_time = 0;
	calcResult.value.full_equipments = [];

	try {
		const result = await invoke("cmd_calculate_skillset", {
			"anomalyFilename": CacheManager.getAnomalyFilename(),
			"talismanFilename": CacheManager.getTalismanFilename(),
			"sexType": calcInput.sexType,
			"weaponSlots": calcInput.weaponSlots,
			"selectedSkills": calcInput.selectedSkills,
			"freeSlots": calcInput.freeSlots,
			"includeLteEquips": calcInput.includeLteEquips,
		}) as { [key: string]: any };

		calcResult.value = result["result"] as CalculateResult;
		is_calculating.value = false;

		console.log(result);
	} catch (e) {
		is_calculating.value = false;
		console.error("cmd_calculate_skillset failed, ", e);
	}
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

	calcResult.value.calc_time = 0;
	calcResult.value.full_equipments = [];
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

function setSearchCondition(fav: SearchFavorite) {
	sexType.value = fav.sexType;
	weaponSlots.value = JSON.parse(JSON.stringify(fav.weaponSlots));
	selectedSkills.value = JSON.parse(JSON.stringify(fav.reqSkills));
	freeSlots.value = JSON.parse(JSON.stringify(fav.reqSlots));

	for (const skill of skillsVec.value) {
		if (selectedSkills.value[skill.id] === undefined) {
			selectedSkills.value[skill.id] = 0;
		}
	}
}

function addResultFavorite(fav: ResultFavorite) {
	emits("add_result_favorite", fav);
}

function canSubmit() {
	return sexType.value !== "";
}

</script>

<template>
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
			<td>{{ UIData["free_slots"][langData] }}</td>
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
			<td style="width: 100px">{{ UIData["sex_type"][langData] }}</td>
			<td>
				<a-select v-model:value="sexType" style="width: 100px">
					<a-select-option value="">---</a-select-option>
					<a-select-option value="male">{{ UIData["male"][langData] }}</a-select-option>
					<a-select-option value="female">{{ UIData["female"][langData] }}</a-select-option>
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
				{{ UIData["weapon_slots"][langData] }}
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

	<a-checkbox v-model:checked="includeLteEquips">{{ UIData["include_lte_equips"][langData] }}</a-checkbox>
	
	<br />
	<br />

	<a-button @click="calculate" :disabled="canSubmit() === false" :type="canSubmit() === true ? 'primary' : 'dashed'" >Calculate</a-button>
	<a-button @click="clear">Clear</a-button>

	<a-divider style="border-color: #7cb305" dashed />

	<div>Answer count : {{ calcResult.full_equipments ? calcResult.full_equipments.length : 0 }}</div>
	<div>Time : {{ calcResult.calc_time }} sec</div>
	<br />
	<div><a-button @click="addFavorite">Save Favorite</a-button> </div>

	<br />

	<template v-if="is_calculating">
		<a-spin size="large" />
	</template>
	<template v-else>
		<SimulateResultTable :langData="langData" :calcResult="calcResult" v-on:add_result_favorite="addResultFavorite" />
	</template>
</template>