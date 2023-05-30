<script setup lang="ts">

import { ref } from "vue";

import { SkillCategory } from "../definition/skill_category_define";

import { Language } from "../definition/language";
import { DecosData } from "../models/decos"
import { SkillsData } from "../models/skills";
import { CacheManager } from "../model/data_manager";

import SkillCategories from "../data/skill_category.json";
import { FinalDecoInfo } from "../definition/deco_define";

import { lm } from "../model/language_manager";

const props = defineProps<{
	langData: Language;
}>();

const skillCats = ref<SkillCategory[]>(SkillCategories);
const decosPerCat = ref<{ [key: string]: FinalDecoInfo[] }>({});
const selectedDecos = ref<{ [key: string]: boolean }>({});
const showSkillName = ref<boolean>(false);

const decosBySkill = DecosData.getAllDecosBySkill();

for (const decoId in CacheManager.getBannedDecos()) {
	selectedDecos.value[decoId] = true;
}

for (const cat of skillCats.value) {
	const catId = cat.id;
	decosPerCat.value[catId] = [];
	
	for(const skillId of cat.skills) {
		const decos = decosBySkill[skillId];

		if (decos !== undefined) {
			decosPerCat.value[catId].push(...decos);
		}
	}
}

for (const catId in decosPerCat.value) {
	decosPerCat.value[catId].sort((a, b) => {
		return a.names[props.langData].localeCompare(b.names[props.langData]);
	});
}

function onBannedDecoChange() {
	const selectedDecosList = {} as { [key: string]: boolean };

	for(const decoId in selectedDecos.value) {
		if (selectedDecos.value[decoId] === true) {
			selectedDecosList[decoId] = true;
		}
	}

	CacheManager.setBannedDecos(selectedDecosList);
}

function getDecoText(deco: FinalDecoInfo) {
	let text = deco.names[props.langData];

	if (showSkillName.value === true) {
		text += ` (${SkillsData.getName(deco.skillId, props.langData)})`;
	}

	return text;
}

</script>

<template>
	<a-switch v-model:checked="showSkillName" />
	<span style="padding-left: 10px;">{{ lm.getString("show_skill_name_pickban") }}</span>

	<br />
	<br />

	<table v-for="cat in skillCats">
		<tr>
			{{ cat.names[langData] }}
		</tr>
		<tr>
			<div>
				<template v-for="deco in decosPerCat[cat.id]">
					<div style="display: inline-block; width: 200px; height: 50px; margin: 10px;">
						<div>
							<a-checkbox v-model:checked="selectedDecos[deco.id]" style="padding-left: 10px;" @change="onBannedDecoChange">
								{{ getDecoText(deco) }}
							</a-checkbox>
						</div>
					</div>
				</template>
			</div>
			<br />
		</tr>
	</table>


</template>