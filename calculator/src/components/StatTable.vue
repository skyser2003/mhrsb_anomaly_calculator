<script setup lang="ts">

import { ref } from "vue";

import UIData from "../ui_data/ui_data.json";
import { Language } from "../definition/language";
import { ArmorStatInfo, getDefaultStat } from "../definition/armor_define";

const props = defineProps<{
	langData: Language,
	stat: ArmorStatInfo,
}>();

const columns = ref([
	{
		title: "unused",
		dataIndex: "defense",
		key: "defense",
	},
	{
		title: "unused",
		dataIndex: "fireRes",
		key: "fireRes",
	},
	{
		title: "unused",
		dataIndex: "waterRes",
		key: "waterRes",
	},
	{
		title: "unused",
		dataIndex: "iceRes",
		key: "iceRes",
	},
	{
		title: "unused",
		dataIndex: "elecRes",
		key: "elecRes",
	},
	{
		title: "unused",
		dataIndex: "dragonRes",
		key: "dragonRes",
	},
]);

function getResImageName(key: string) {
	return new URL(`../assets/${key}.png`, import.meta.url).href;
}

</script>

<template>
	<a-table class="stat_table" :columns="columns" :data-source="[stat, stat]" :pagination="{ defaultPageSize: 100, hideOnSinglePage: true}">
		<template #headerCell="{}">
			<template></template>
		</template>
		<template #bodyCell="{ index, column, text }">
			<template v-if="column.key === 'defense'">
				<template v-if="index === 0">
					{{ UIData["defense"][langData] }}
				</template>
				<template v-else>
					{{ stat.defense }}
				</template>
			</template>
			<template v-else>
				<template v-if="index === 0">
					<a-image :src="`${getResImageName(column.key)}`" :width="20" :preview="false" />
				</template>
				<template v-else>
					{{ text }}
				</template>
			</template>
		</template>
	</a-table>
</template>

<style>
.stat_table .ant-table {
	margin: 0 !important;
}

.stat_table .ant-table-thead {
	display: none;
}
</style>