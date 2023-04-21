<script setup lang="ts">

import { ref } from "vue";
import { invoke } from "@tauri-apps/api/tauri";
import { getVersion } from "@tauri-apps/api/app";

import SimulateTab from "./components/SimulateTab.vue";
import AnomaliesTab from "./components/AnomaliesTab.vue";
import TalismansTab from "./components/TalismansTab.vue";
import SearchFavoriteTab from "./components/SearchFavoriteTab.vue";
import ResultFavoriteTab from "./components/ResultFavoriteTab.vue";

import UIData from "./ui_data/ui_data.json";
import { ResultFavorite, SearchFavorite } from "./definition/calculate_result";
import { Language } from "./definition/language";
import { CacheManager } from "./model/data_manager";

const gaScript1 = document.createElement("script");
gaScript1.setAttribute("async", "");
gaScript1.setAttribute("src", "https://www.googletagmanager.com/gtag/js?id=G-DNNYHLWD3N");

const gaScript2 = document.createElement("script");
gaScript2.innerHTML = `
  window.dataLayer = window.dataLayer || [];
  function gtag(){dataLayer.push(arguments);}
  gtag('js', new Date());

  gtag('config', 'G-DNNYHLWD3N');
`;

document.head.insertBefore(gaScript2, document.head.firstChild);
document.head.insertBefore(gaScript1, document.head.firstChild);

const appVersion = ref("");
getVersion().then((version) => {
  appVersion.value = version;
});

const langData = ref<Language>("ko");
const activeKey = ref("0");

const designTheme = ref("");

const simulateTab = ref<InstanceType<typeof SimulateTab>>();
const anomalyTab = ref<InstanceType<typeof AnomaliesTab>>();
const talismanTab = ref<InstanceType<typeof TalismansTab>>();
const searchFavoriteTab = ref<InstanceType<typeof SearchFavoriteTab>>();
const resultFavoriteTab = ref<InstanceType<typeof ResultFavoriteTab>>();

const searchFavorites = ref<SearchFavorite[]>([]);
const resultFavorites = ref<ResultFavorite[]>([]);

loadTheme();
loadLanguage();
loadFiles();
loadSearchFavorites();
loadResultFavorites();
loadLatestTab();

function loadTheme() {
  designTheme.value = CacheManager.getDesignTheme();
}

function loadLanguage() {
  langData.value = CacheManager.getLanguage();
  console.log(langData.value);
}

async function loadFiles() {
  const anomalyFilename = CacheManager.getAnomalyFilename();
  const talismanFilename = CacheManager.getTalismanFilename();

  const promises = [];

  if (anomalyFilename !== null) {
    promises.push(invoke("cmd_parse_anomaly", { filename: anomalyFilename }));
  }

  if (talismanFilename !== null) {
    promises.push(invoke("cmd_parse_talisman", { filename: talismanFilename }));
  }

  await Promise.all(promises);
}

function loadSearchFavorites() {
  searchFavorites.value = CacheManager.getSearchFavorites();
}

function loadResultFavorites() {
  resultFavorites.value = CacheManager.getResultFavorites();
}

function loadLatestTab() {
  const latestTab = CacheManager.getTab();

  if (latestTab !== null) {
    setTab(latestTab);
  }
}

function addSearchFavorite(fav: SearchFavorite) {
  loadSearchFavorites();
  
  searchFavorites.value.push(fav);
  CacheManager.setSearchFavorites(searchFavorites.value);
}

function setSearchCondition(fav: SearchFavorite) {
  simulateTab.value?.setSearchCondition(fav);
  setTab("0");
}

function addResultFavorite(fav: ResultFavorite) {
  loadResultFavorites();

  resultFavorites.value.push(fav);
  CacheManager.setResultFavorites(resultFavorites.value);
}

function changeTab() {
  CacheManager.setTab(activeKey.value);

  if (activeKey.value === "1") { // Search favorites tab
    searchFavoriteTab.value?.onTabActivate();
  } else if (activeKey.value === "2") { // Result favorites tab
    resultFavoriteTab.value?.onTabActivate();
  } else if (activeKey.value === "3") { // Anomaly armors tab
    anomalyTab.value?.getFileAnomalies();
  } else if (activeKey.value === "4") { // Talisman tab
    talismanTab.value?.getFileTalismans();
  }
}

function setTab(key: string) {
  activeKey.value = key;
  changeTab();
}

function onChangeLanguage() {
  CacheManager.setLanguage(langData.value);
  location.reload();
}

function onChangeDesignTheme() {
  CacheManager.setDesignTheme(designTheme.value);
  location.reload();
}

</script>

<template>
  <div>
    <h2>{{ UIData["language"][langData] }}</h2>
  </div>
  <a-radio-group v-model:value="langData" @change="onChangeLanguage">
    <a-radio-button value="ko">한국어</a-radio-button>
    <a-radio-button value="en">English</a-radio-button>
  </a-radio-group>
  <div>
    <h2>{{ UIData["design_theme"][langData] }}</h2>
  </div>
  <a-radio-group v-model:value="designTheme" @change="onChangeDesignTheme">
    <a-radio-button value="light">{{ UIData["light_theme"][langData] }}</a-radio-button>
    <a-radio-button value="dark">{{ UIData["dark_theme"][langData] }} </a-radio-button>
  </a-radio-group>

  <a-divider style="border-color: #7cb305" dashed />

  <a-tabs v-model:activeKey="activeKey" type="card" @change="changeTab()">
    <a-tab-pane key="0" :tab="UIData['simulation_tab'][langData]">
      <Suspense>
        <SimulateTab ref="simulateTab" :langData="langData" v-on:add_search_favorite="addSearchFavorite" v-on:add_result_favorite="addResultFavorite" />
      </Suspense>
    </a-tab-pane>
    <a-tab-pane key="1" :tab="UIData['search_favorite_tab'][langData]">
      <SearchFavoriteTab ref="searchFavoriteTab" :langData="langData" :favorites="searchFavorites" v-on:set_search_condition="setSearchCondition" />
    </a-tab-pane>
    <a-tab-pane key="2" :tab="UIData['result_favorite_tab'][langData]">
      <ResultFavoriteTab ref="resultFavoriteTab" :langData="langData" :favorites="resultFavorites" />
    </a-tab-pane>
    <a-tab-pane key="3" :tab="UIData['anomaly_tab'][langData]">
      <Suspense>
        <AnomaliesTab ref="anomalyTab" :langData="langData" />
      </Suspense>
    </a-tab-pane>
    <a-tab-pane key="4" :tab="UIData['talisman_tab'][langData]">
      <Suspense>
        <TalismansTab ref="talismanTab" :langData="langData" />
      </Suspense>
    </a-tab-pane>
  </a-tabs>
  <a-divider>
    {{ UIData["github_url"][langData] }} -
    <a href="https://github.com/skyser2003/mhrsb_anomaly_calculator" title="https://github.com/skyser2003/mhrsb_anomaly_calculator" target="_blank">
      https://github.com/skyser2003/mhrsb_anomaly_calculator
    </a>
  </a-divider>
  <a-divider>{{ UIData["questions_and_feedbacks"][langData] }} - mhrsb.calculator@gmail.com</a-divider>
  <a-divider>{{ UIData["app_version"][langData] }} {{ appVersion }}</a-divider>
</template>
