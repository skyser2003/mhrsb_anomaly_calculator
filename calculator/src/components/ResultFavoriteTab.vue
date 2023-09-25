<script setup lang="ts">
import { ref, onMounted, nextTick } from "vue";

import { CheckOutlined, EditOutlined } from '@ant-design/icons-vue';

import Sortable from "sortablejs";

import { SkillsData } from "../models/skills";
import { DecosData } from "../models/decos";

import { ResultFavorite, ResultFullEquipments, Skills, getTotalStat } from "../definition/calculate_result";

import { lm } from "../model/language_manager";
import { getDecoCombTexts } from "../model/ui";
import { Language } from "../definition/language";
import { CacheManager } from "../model/data_manager";

import ResultFavoriteRow from "./ResultFavoriteRow.vue";
import StatTable from "./StatTable.vue";
import { ArmorStatInfo } from "../definition/armor_define";


interface Row {
	id: string;
	name: string;
	sex_type: string;
	weapon_slots: string;
	skills: string[];
	deco_combs: string[];
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
		title: lm.getString("id_column"),
		dataIndex: "id",
		key: "id",
		width: 50
	},
	{
		title: lm.getString("name_column"),
		dataIndex: "name",
		key: "name",
		width: 200
	}, {
		title: lm.getString("sex_type"),
		dataIndex: "sex_type",
		key: "sex_type",
		width: 100,
	},
	{
		title: lm.getString("weapon_slots"),
		dataIndex: "weapon_slots",
		key: "weapon_slots",
	},
	{
		title: lm.getString("skills_column"),
		dataIndex: "skills",
		key: "skills",
	},
	{
		title: lm.getString("decorations_name"),
		dataIndex: "deco_combs",
		key: "deco_combs",
	},
	{
		title: lm.getString("leftover_slots"),
		dataIndex: "leftover_slots",
		key: "leftover_slots",
	},
	{
		title: lm.getString("stat_name"),
		dataIndex: "stat",
		key: "stat",
	},
	{
		title: lm.getString("delete"),
		dataIndex: "delete",
		key: "delete",
	}
]);

const isReordering = ref<boolean>(false);
const isEditing = ref<{ [key: number]: boolean }>({});
const expandedRowKeys = ref<string[]>([]);

function onTabActivate() {
	isReordering.value = false;
	isEditing.value = {};
	props.favorites.length = 0;
	props.favorites.push(...CacheManager.getResultFavorites());
}

function generateTableData(favs: ResultFavorite[]) {
	return favs.map((fav, index) => {
		const id = `${index}`;

		let skills = {} as Skills;

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

			for (let i = 0; i < decos.length; ++i) {
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

		skills = SkillsData.sortByName(skills, props.langData);

		const skillTexts = [];

		for (const skillId in skills) {
			const name = SkillsData.getName(skillId, props.langData);
			const level = skills[skillId];

			const text = `${name} Lv${level}`;
			skillTexts.push(text);
		}

		const allDecoTexts = getDecoCombTexts(fav.decoComb, props.langData);

		const weaponSlotsText = JSON.stringify(fav.weaponSlots);
		const leftoverSlotsText = JSON.stringify(fav.decoComb.leftoverSlotsSum);

		return {
			key: index,
			id,
			name: fav.name,
			sex_type: lm.getString(fav.sexType),
			weapon_slots: weaponSlotsText,
			skills: skillTexts,
			deco_combs: allDecoTexts,
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
	if (isReordering.value) {
		return;
	}

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

	return ret;
}

let sortable: Sortable;

onMounted(() => {
	const root = document.querySelector("#result_favorite_table .ant-table-tbody")! as HTMLElement;
	root.setAttribute("id", "result_favorite_table_body");

	sortable = Sortable.create(root, {
		animation: 150,
		draggable: ".ant-table-row",
		forceFallback: true,
		disabled: !isReordering.value,
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

			if (oldIndex > newIndex) {
				evt.target.insertBefore(evt.target.children.item(newIndex)!, evt.target.children.item(oldIndex)!.nextSibling);
			} else {
				evt.target.insertBefore(evt.target.children.item(newIndex)!, evt.target.children.item(oldIndex)!);
			}

			const oldElem = props.favorites.splice(oldIndex, 1)[0];
			props.favorites.splice(newIndex, 0, oldElem);

			CacheManager.setResultFavorites(props.favorites);
		},
	});
});

async function switchReorder() {
	sortable.option("disabled", !isReordering.value);

	if (isReordering.value) {
		isEditing.value = {};
		expandedRowKeys.value = [];
	} else {
		CacheManager.setResultFavorites(props.favorites);

		props.favorites.length = 0;
		await nextTick();
		props.favorites.splice(0, 0, ...CacheManager.getResultFavorites());
	}
}

function isRowExpandable(record: Row) {
	return !isReordering.value;
}

</script>

<template>
	<a-switch v-model:checked="isReordering" @change="switchReorder" />
	<span style="padding-left: 10px;">{{ lm.getString("reorder") }}</span>

	<br />
	<br />

	<a-table :columns="columns" :data-source="generateTableData(props.favorites)"
		:pagination="{ defaultPageSize: 200, hideOnSinglePage: true }" :row-expandable="isRowExpandable"
		v-model:expandedRowKeys="expandedRowKeys" id="result_favorite_table">
		<template #bodyCell="{ text, index, column, record }">
			<template v-if="column.key === 'name'">
				<template v-if="isEditing[index] === true">
					<a-input style="width: 150px" v-model:value="props.favorites[index].name"
						@pressEnter="saveName(index)" />
					<CheckOutlined @click="saveName(index)" />
				</template>

				<template v-else>
					<template v-if="text === undefined || text === null || text.length === 0">
						<span class="empty_name">{{ lm.getString("empty_favorite_name") }}</span>
					</template>
					<template v-else>
						{{ text }}
					</template>
					<EditOutlined @click="beginEditName(index)" :disabled="isReordering" style="padding-left: 10px" />
				</template>
			</template>

			<template v-else-if="column.key === 'skills'">
				<a-tag v-for="skill in record.skills">
					{{ skill }}
				</a-tag>
			</template>

			<template v-if="column.key === 'deco_combs'">
				<a-tag v-for="deco in record[column.key]">
					{{ deco }}
				</a-tag>
			</template>

			<template v-else-if="column.key === 'stat'">
				<StatTable :langData="langData" :stat="record.stat" />
			</template>

			<template v-else-if="column.key === 'delete'">
				<a-popconfirm :title="lm.getString('confirm_delete')" ok-text="O" cancel-text="X"
					@confirm="deleteFavorite(index)" @cancel="" :disabled="isReordering">
					<a-button :disabled="isReordering">X</a-button>
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