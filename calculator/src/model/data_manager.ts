import { ArmorParts } from "../definition/armor_define";
import { AnomalyArmorInfo, CalcChoices, ResultFavorite, SearchFavorite, TalismanInfo } from "../definition/calculate_result";
import { Language, LanguagesValue } from "../definition/language";

const setItem = window.localStorage.setItem.bind(window.localStorage);
const getItem = window.localStorage.getItem.bind(window.localStorage);

export class DataManager {

}

export class CacheManager {
	static languageName = "ui_language";
	static activeTabName = "active_tab";

	static calcChoicesName = "calc_choices";

	static searchFavoriteName = "search_favorites";
	static resultFavoriteName = "result_favorites";

	static anomalyFilenameName = "anomaly_filename";
	static talismanFilenameName = "talisman_filename";

	static manualAnomaliesName = "manual_anomalies";
	static manualTalismansName = "manual_talismans";

	static defaultCalcChoices: CalcChoices = {
		sexType: "",
		weaponSlots: [0, 0, 0],
		selectedSkills: {},
		freeSlots: [0, 0, 0, 0],
	};

	static setLanguage(lang: Language) {
		setItem(this.languageName, lang);
	}

	static getLanguage() {
		const lang = getItem(this.languageName) as Language;

		if (lang === null) {
			const systemLang = navigator.language;

			for (const lang of LanguagesValue) {
				if (systemLang.startsWith(lang)) {
					return lang;
				}
			}
		}

		return lang;
	}

	static setTab(key: string) {
		setItem(this.activeTabName, key);
	}

	static getTab() {
		return getItem(this.activeTabName) ?? "0";
	}

	static setCalcChoices(choices: CalcChoices) {
		setItem(this.calcChoicesName, JSON.stringify(choices));
	}

	static setEmptyCalcChoices() {
		setItem(this.calcChoicesName, JSON.stringify(this.defaultCalcChoices));
	}

	static getCalcChoices() {
		try {
			const value = JSON.parse(getItem(this.calcChoicesName)!) as CalcChoices;

			if (value !== null) {
				return value;
			}
		} catch (e) {

		}

		return JSON.parse(JSON.stringify(this.defaultCalcChoices)) as CalcChoices;
	}

	static setSearchFavorites(favs: SearchFavorite[]) {
		setItem(this.searchFavoriteName, JSON.stringify(favs));
	}

	static getSearchFavorites() {
		try {
			return JSON.parse(getItem(this.searchFavoriteName)!) as SearchFavorite[] ?? [];
		} catch (e) {
			return [];
		}
	}

	static setResultFavorites(favs: ResultFavorite[]) {
		setItem(this.resultFavoriteName, JSON.stringify(favs));
	}

	static getResultFavorites() {
		try {
			return JSON.parse(getItem(this.resultFavoriteName)!) as ResultFavorite[] ?? [];
		} catch (e) {
			return [];
		}
	}

	static setAnomalyFilename(filename: string) {
		setItem(this.anomalyFilenameName, filename);
	}

	static getAnomalyFilename() {
		return getItem(this.anomalyFilenameName) ?? "";
	}

	static setTalismanFilename(filename: string) {
		setItem(this.talismanFilenameName, filename);
	}

	static getTalismanFilename() {
		return getItem(this.talismanFilenameName) ?? "";
	}

	static setManualAnomalies(anomalies: {
		[key: string]: AnomalyArmorInfo[];
	}) {
		setItem(this.manualAnomaliesName, JSON.stringify(anomalies));
	}

	static getManualAnomalies() {
		try {
			return JSON.parse(getItem(this.manualAnomaliesName)!) as { [key: string]: AnomalyArmorInfo[] } ?? this.getDefaultManualAnomalies();
		} catch (e) {
			return this.getDefaultManualAnomalies();
		}
	}

	static getDefaultManualAnomalies() {
		const ret = {} as { [key: string]: AnomalyArmorInfo[] };

		for (const part of ArmorParts) {
			ret[part] = [];
		}

		return ret;
	}

	static setManualTalismans(talismans: TalismanInfo[]) {
		setItem(this.manualTalismansName, JSON.stringify(talismans));
	}

	static getManualTalismans() {
		try {
			return JSON.parse(getItem(this.manualTalismansName)!) as TalismanInfo[] ?? [];
		} catch (e) {
			return [];
		}
	}
}

export class InvokeManager {

}

export const cm = new CacheManager();