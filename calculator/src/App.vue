<script setup lang="ts">

import { Ref, ref } from "vue";

import SimulateTab from "./components/SimulateTab.vue";
import AnomaliesTab from "./components/AnomaliesTab.vue";
import TalismansTab from "./components/TalismansTab.vue";
import SearchFavoriteTab from "./components/SearchFavoriteTab.vue";
import ResultFavoriteTab from "./components/ResultFavoriteTab.vue";

import UIData from "./ui_data/ui_data.json";
import { ResultFavorite, SearchFavorite } from "./definition/calculate_result";
import { Language } from "./definition/language";
import { CacheManager } from "./model/data_manager";


const langData = ref<Language>("ko");
const activeKey = ref("0");

const simulateTab = ref<InstanceType<typeof SimulateTab>>();
const anomalyTab = ref<InstanceType<typeof AnomaliesTab>>();
const talismanTab = ref<InstanceType<typeof TalismansTab>>();

const searchFavorites = ref([]) as Ref<SearchFavorite[]>;
const resultFavorites = ref([]) as Ref<ResultFavorite[]>;

loadLanguage();
loadSearchFavorites();
loadResultFavorites();
loadLatestTab();

function loadLanguage() {
  langData.value = CacheManager.getLanguage();
  console.log(langData.value);
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

  if (activeKey.value === "3") { // Anomaly armors tab
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
  const lang = langData.value;

  console.log("boo");
  CacheManager.setLanguage(lang);
  console.log("boo");
  location.reload();
  console.log("boo");
}

</script>

<template>
  <div><h2>{{ UIData["language"][langData] }}</h2></div>
  <a-radio-group v-model:value="langData" @change="onChangeLanguage">
    <a-radio-button value="ko">한국어</a-radio-button>
    <a-radio-button value="en">English</a-radio-button>
  </a-radio-group>

  <a-divider style="border-color: #7cb305" dashed />

  <a-tabs v-model:activeKey="activeKey" type="card" @change="changeTab()">
    <a-tab-pane key="0" :tab="UIData['simulation_tab'][langData]">
      <Suspense>
        <SimulateTab ref="simulateTab" :langData="langData" v-on:add_search_favorite="addSearchFavorite" v-on:add_result_favorite="addResultFavorite" />
      </Suspense>
    </a-tab-pane>
    <a-tab-pane key="1" :tab="UIData['search_favorite_tab'][langData]">
      <SearchFavoriteTab :langData="langData" :favorites="searchFavorites" v-on:set_search_condition="setSearchCondition" />
    </a-tab-pane>
    <a-tab-pane key="2" :tab="UIData['result_favorite_tab'][langData]">
      <ResultFavoriteTab :langData="langData" :favorites="resultFavorites" />
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
  <a-divider>Created by</a-divider>
  <a-divider>
    <a-anchor>
    <a-anchor-link href="https://github.com/skyser2003/mhrsb_anomay_calculator" title="https://github.com/skyser2003/mhrsb_anomay_calculator" target="_blank" />
    </a-anchor>
  </a-divider>
    <a-divider>Questions and feedbacks to - mhrsb.calculator@gmail.com</a-divider>
</template>

<style scoped>
.logo.vite:hover {
  filter: drop-shadow(0 0 2em #747bff);
}

.logo.vue:hover {
  filter: drop-shadow(0 0 2em #249b73);
}

#loading {
  position: fixed;
  width: 100%;
  height: 100%;
  background-color: rgba(230, 230, 230, 60%);

  display: flex;
  justify-content: center;
  align-items: center;
}

#loading.disabled {
  visibility: hidden;
  width: 0px;
  height: 0px;
}

.lds-default {
  display: inline-block;
  position: relative;
  width: 80px;
  height: 80px;
}

.lds-default div {
  position: absolute;
  width: 6px;
  height: 6px;
  background: #fff;
  border-radius: 50%;
  animation: lds-default 1.2s linear infinite;
}

.lds-default div:nth-child(1) {
  animation-delay: 0s;
  top: 37px;
  left: 66px;
}

.lds-default div:nth-child(2) {
  animation-delay: -0.1s;
  top: 22px;
  left: 62px;
}

.lds-default div:nth-child(3) {
  animation-delay: -0.2s;
  top: 11px;
  left: 52px;
}

.lds-default div:nth-child(4) {
  animation-delay: -0.3s;
  top: 7px;
  left: 37px;
}

.lds-default div:nth-child(5) {
  animation-delay: -0.4s;
  top: 11px;
  left: 22px;
}

.lds-default div:nth-child(6) {
  animation-delay: -0.5s;
  top: 22px;
  left: 11px;
}

.lds-default div:nth-child(7) {
  animation-delay: -0.6s;
  top: 37px;
  left: 7px;
}

.lds-default div:nth-child(8) {
  animation-delay: -0.7s;
  top: 52px;
  left: 11px;
}

.lds-default div:nth-child(9) {
  animation-delay: -0.8s;
  top: 62px;
  left: 22px;
}

.lds-default div:nth-child(10) {
  animation-delay: -0.9s;
  top: 66px;
  left: 37px;
}

.lds-default div:nth-child(11) {
  animation-delay: -1s;
  top: 62px;
  left: 52px;
}

.lds-default div:nth-child(12) {
  animation-delay: -1.1s;
  top: 52px;
  left: 62px;
}

@keyframes lds-default {

  0%,
  20%,
  80%,
  100% {
    transform: scale(1);
  }

  50% {
    transform: scale(1.5);
  }
}
</style>
