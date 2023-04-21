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
	<table class="ant-table">
		<thead class="ant-table-thead">
			<tr>
				<template v-for="statKey in Object.keys(stat)">
					<template v-if="statKey === 'defense'">
						<th class="ant-table-cell">{{ UIData[statKey][langData] }}</th>
					</template>
					<template v-else>
						<th><img :src="`${getResImageName(statKey)}`" :width="20" /></th>
					</template>
				</template>
			</tr>
		</thead>
		<tbody class="ant-table-tbody">
			<tr>
				<template v-for="value in stat">
					<td class="ant-table-cell">{{ value }}</td>
				</template>
			</tr>
		</tbody>
	</table>
</template>