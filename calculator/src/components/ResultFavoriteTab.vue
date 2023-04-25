<script setup lang="ts">
import { ref, onMounted } from "vue";

import { CheckOutlined, EditOutlined } from '@ant-design/icons-vue';

import Sortable from "sortablejs";

import { SkillsData } from "../models/skills";
import { DecosData } from "../models/decos";

import { ResultFavorite, ResultFullEquipments, Skills, getTotalStat } from "../definition/calculate_result";

import uiData from "../ui_data/ui_data.json";
import { getDecoCombTexts } from "../model/ui";
import { Language } from "../definition/language";
import { CacheManager } from "../model/data_manager";

import ResultFavoriteRow from "./ResultFavoriteRow.vue";
import StatTable from "./StatTable.vue";
import { ArmorStatInfo } from "../definition/armor_define";


const UIData = uiData as { [key: string]: { [key: string]: string } };

interface Row {
	id: string;
	name: string;
	sex_type: string;
	weapon_slots: string;
	skills: string;
	deco_combs: string;
	leftover_slots: string;
	stat: ArmorStatInfo;
}


const props = defineProps<{
	langData: Language,
	favorites: ResultFavorite[]
}>();

defineExpose({ onTabActivate });

const columns = ref([
	{
		title: "id",
		dataIndex: "id",
		key: "id",
		width: 50
	},
	{
		title: "name",
		dataIndex: "name",
		key: "name",
		width: 200
	}, {
		title: UIData["sex_type"][props.langData],
		dataIndex: "sex_type",
		key: "sex_type",
		width: 100,
	},
	{
		title: UIData["weapon_slots"][props.langData],
		dataIndex: "weapon_slots",
		key: "weapon_slots",
	},
	{
		title: "skills",
		dataIndex: "skills",
		key: "skills",
	},
	{
		title: UIData["decorations_name"][props.langData],
		dataIndex: "deco_combs",
		key: "deco_combs",
	},
	{
		title: UIData["leftover_slots"][props.langData],
		dataIndex: "leftover_slots",
		key: "leftover_slots",
	},
	{
		title: UIData["stat_name"][props.langData],
		dataIndex: "stat",
		key: "stat",
	},
	{
		title: UIData["delete"][props.langData],
		dataIndex: "delete",
		key: "delete",
	}
]);

const isEditing = ref<{ [key: number]: boolean }>({});

function onTabActivate() {
	isEditing.value = {};
	props.favorites.length = 0;
	props.favorites.push(...CacheManager.getResultFavorites());
}

function generateTableData(favs: ResultFavorite[]) {
	return favs.map((fav, index) => {
		const id = `${index}`;

		const skills = {} as Skills;
		const skillTexts = [];

		for (const equipId in fav.armors) {
			const equip = fav.armors[equipId];
			
			for (const skillId in equip.skills) {
				const level = equip.skills[skillId];

				if (skills[skillId] === undefined) {
					skills[skillId] = level;
				} else {
					skills[skillId] += level;
				}
			}
		}

		for (const skillId in fav.talisman.skills) {
			const level = fav.talisman.skills[skillId];

			if (skills[skillId] === undefined) {
				skills[skillId] = level;
			} else {
				skills[skillId] += level;
			}
		}

		for (const skillId in fav.decoComb.skillDecos) {
			const decos = fav.decoComb.skillDecos[skillId];
			let level = 0;

			for(let i = 0; i < decos.length; ++i) {
				const count = decos[i];
				const decoInfo = DecosData.getInfo(skillId, i);

				level += count * decoInfo.skillLevel;
			}

			if (skills[skillId] === undefined) {
				skills[skillId] = level;
			} else {
				skills[skillId] += level;
			}
		}

		for (const skillId in skills) {
			const name = SkillsData.getName(skillId, props.langData);
			const level = skills[skillId];

			const text = `${name} Lv${level}`;
			skillTexts.push(text);
		}

		const allDecoTexts = getDecoCombTexts(fav.decoComb, props.langData);

		const skillsText = skillTexts.join(", ");
		const weaponSlotsText = JSON.stringify(fav.weaponSlots);
		const leftoverSlotsText = JSON.stringify(fav.decoComb.leftoverSlotsSum);

		return {
			key: index,
			id,
			name: fav.name,
			sex_type: UIData[fav.sexType][props.langData],
			weapon_slots: weaponSlotsText,
			skills: skillsText,
			deco_combs: allDecoTexts.join(" - "),
			leftover_slots: leftoverSlotsText,
			stat: getTotalStat(fav.armors),
		} as Row;
	});
}

function deleteFavorite(index: number) {
	isEditing.value = {};
	props.favorites.splice(index, 1);
	CacheManager.setResultFavorites(props.favorites);
}

function beginEditName(index: number) {
	isEditing.value[index] = true;
}

function saveName(index: number) {
	props.favorites[index].name = props.favorites[index].name.trim();
	CacheManager.setResultFavorites(props.favorites);

	isEditing.value[index] = false;
}

function generateResultFullEquipments(fav: ResultFavorite) {
	const ret: ResultFullEquipments = {
		sexType: fav.sexType,
		weaponSlots: fav.weaponSlots,
		armors: fav.armors,
		talisman: fav.talisman,
		decoCombs: [fav.decoComb],
		commonLeftoverSkills: {},
	};

	console.log(ret.decoCombs[0].leftoverSkills);
	
	return ret;
}

onMounted(() => {
	const root = document.querySelector("#result_favorite_table .ant-table-tbody")! as HTMLElement;
	root.setAttribute("id", "result_favorite_table_body");

	const sortable = Sortable.create(root, {
		animation: 150,
		draggable: ".ant-table-row",
		forceFallback: true,
		filter: "svg, button, input, .ant-table-expanded-row",

		onEnd: async (evt) => {
			const oldIndex = evt.oldIndex!;
			const newIndex = evt.newIndex!;

			if (typeof oldIndex !== 'number' || typeof newIndex !== 'number') {
				return;
			}

			if (oldIndex === newIndex) {
				return;
			}

			const allChildren = [];

			for (let i = 0; i < evt.target.children.length; ++i) {
				allChildren.push(evt.target.children.item(i)!);
			}

			const oldTr = allChildren.splice(newIndex, 1)[0];
			allChildren.splice(oldIndex, 0, oldTr);

			evt.target.replaceChildren(...allChildren);

			const oldElem = props.favorites.splice(oldIndex, 1)[0];
			props.favorites.splice(newIndex, 0, oldElem);

			CacheManager.setResultFavorites(props.favorites);
		},
	});
});


</script>

<template>
	<a-table :columns="columns" :data-source="generateTableData(props.favorites)" :pagination="{ defaultPageSize: 200, hideOnSinglePage: true }" id="result_favorite_table">
		<template #bodyCell="{ text, index, column, record }">
			<template v-if="column.key === 'name'">
				<template v-if="isEditing[index] === true">
					<a-input style="width: 150px" v-model:value="props.favorites[index].name" @pressEnter="saveName(index)" />
					<CheckOutlined @click="saveName(index)" />
				</template>

				<template v-else>
					<template v-if="text === undefined || text === null || text.length === 0">
						<span class="empty_name">{{ UIData["empty_favorite_name"][props.langData] }}</span>
					</template>
					<template v-else>
						{{ text }}
					</template>
					<EditOutlined @click="beginEditName(index)" />
				</template>
			</template>
			
			<template v-else-if="column.key === 'stat'">
				<StatTable :langData="langData" :stat="record.stat" />
			</template>

			<template v-else-if="column.key === 'delete'">
				<a-popconfirm :title="UIData['confirm_delete'][langData]" ok-text="O" cancel-text="X" @confirm="deleteFavorite(index)" @cancel="">
					<a-button>X</a-button>
				</a-popconfirm>
			</template>
		</template>

		<template #expandedRowRender="{ index }">
			<ResultFavoriteRow :langData="langData" :data="generateResultFullEquipments(favorites[index])" />
		</template>
	</a-table>
</template>

<style scoped>

.empty_name {
	text-decoration-line: line-through;
}

</style>