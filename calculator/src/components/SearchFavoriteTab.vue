<script setup lang="ts">
import { ref, onMounted, nextTick } from "vue";

import { CheckOutlined, EditOutlined } from '@ant-design/icons-vue';

import Sortable from "sortablejs";

import { SkillsData } from "../models/skills";

import { SearchFavorite } from "../definition/calculate_result";

import UIData from "../ui_data/ui_data.json";
import { CacheManager } from "../model/data_manager";


interface Row {
	id: string;
	name: string;
	weapon_slots: string;
	skills: string;
	req_slots: string;
}

const props = defineProps<{
	langData: "en" | "ko",
	favorites: SearchFavorite[]
}>();

const emits = defineEmits<{
	(event: "set_search_condition", fav: SearchFavorite): void
}>();

defineExpose({ onTabActivate });


const columns = ref([
	{
		title: "id",
		dataIndex: "id",
		key: "id",
		width: 50,
	},
	{
		title: "name",
		dataIndex: "name",
		key: "name",
		width: 200,
	},
	{
		title: UIData["sex_type"][props.langData],
		dataIndex: "sex_type",
		key: "sex_type",
		width: 100,
	},
	{
		title: UIData["weapon_slots"][props.langData],
		dataIndex: "weapon_slots",
		key: "weapon_slots",
		width: 100,
	},
	{
		title: "skills",
		dataIndex: "skills",
		key: "skills"
	},
	{
		title: UIData["free_slots"][props.langData],
		dataIndex: "req_slots",
		key: "req_slots",
		width: 100,
	},
	{
		title: UIData["set_search"][props.langData],
		dataIndex: "set_search",
		key: "set_search"
	},
	{
		title: UIData["delete"][props.langData],
		dataIndex: "delete",
		key: "delete"
	}
]);

const isReordering = ref<boolean>(false);
const isEditing = ref<{ [key: number]: boolean }>({});

function onTabActivate() {
	isReordering.value = false;
	isEditing.value = {};
	props.favorites.length = 0;
	props.favorites.push(...CacheManager.getSearchFavorites());
}

function generateTableData(favs: SearchFavorite[]) {
	return favs.map((fav, index) => {
		const id = `${index}`;

		const skillTexts = [];

		for (const skillId in fav.reqSkills) {
			const level = fav.reqSkills[skillId];
			const name = SkillsData.getName(skillId, props.langData);

			const text = `${name} Lv${level}`;
			skillTexts.push(text);
		}

		const skillsText = skillTexts.join(", ");
		const weaponSlotsText = JSON.stringify(fav.weaponSlots);
		const reqSlotsText = JSON.stringify(fav.reqSlots);

		console.log(fav);

		return {
			key: index,
			id,
			name: fav.name,
			sex_type: fav.sexType === "" ? "" : UIData[fav.sexType][props.langData],
			weapon_slots: weaponSlotsText,
			skills: skillsText,
			req_slots: reqSlotsText
		} as Row;
	});
}

function setSearch(index: number) {
	emits("set_search_condition", props.favorites[index]);
}

function deleteFavorite(index: number) {
	props.favorites.splice(index, 1);
	CacheManager.setSearchFavorites(props.favorites);
}

function beginEditName(index: number) {
	if (isReordering.value) {
		return;
	}

	isEditing.value[index] = true;
}

function saveName(index: number) {
	props.favorites[index].name = props.favorites[index].name.trim();
	CacheManager.setSearchFavorites(props.favorites);

	isEditing.value[index] = false;
}

let sortable: Sortable;

onMounted(() => {
	const root = document.querySelector("#search_favorite_table .ant-table-tbody")! as HTMLElement;

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

			CacheManager.setSearchFavorites(props.favorites);
		},
	});
});

async function switchReorder() {
	sortable.option("disabled", !isReordering.value);

	if (isReordering.value) {
		isEditing.value = {};
	} else {
		CacheManager.setSearchFavorites(props.favorites);

		props.favorites.length = 0;
		await nextTick();
		props.favorites.splice(0, 0, ...CacheManager.getSearchFavorites());
	}
}

</script>

<template>
	<a-switch v-model:checked="isReordering" @change="switchReorder" />
	<span style="padding-left: 10px;">{{ UIData["reorder"][langData] }}</span>

	<br />
	<br />

	<a-table :columns="columns" :data-source="generateTableData(props.favorites)" :pagination="{ hideOnSinglePage: true }" id="search_favorite_table">
		<template #bodyCell="{ text, index, column }">
			<template v-if="column.key === 'name'">
				<template v-if="isEditing[index] === true">
					<a-input style="width: 150px" v-model:value="props.favorites[index].name" @pressEnter="saveName(index)" />
					<CheckOutlined @click="saveName(index)" />
				</template>
			
				<template v-else>
					<template v-if="text === undefined || text === null || text.length === 0 ">
						<span class="empty_name">{{ UIData["empty_favorite_name"][props.langData] }}</span>
					</template>
					<template v-else>
						{{ text }}
					</template>
					<EditOutlined @click="beginEditName(index)" :disabled="isReordering" />
				</template>
			</template>

			<template v-else-if="column.key === 'set_search'">
				<a-button @click="setSearch(index)" type="primary" :disabled="isReordering">★</a-button>
			</template>
			
			<template v-else-if="column.key === 'delete'">
				<a-popconfirm :title="UIData['confirm_delete'][langData]" ok-text="O" cancel-text="X" @confirm="deleteFavorite(index)" @cancel="" >
					<a-button :disabled="isReordering">X</a-button>
				</a-popconfirm>
			</template>
			
			<template v-else>
				{{ text }}
			</template>
		</template>
	</a-table>
</template>

<style scoped>

.empty_name {
	text-decoration-line: line-through;
}

</style>