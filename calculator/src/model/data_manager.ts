import { ArmorParts } from "../definition/armor_define";
import { AnomalyArmorInfo, CalcChoices, ResultFavorite, SearchFavorite, TalismanInfo } from "../definition/calculate_result";
import { Language, LanguagesValue } from "../definition/language";

const setItem = window.localStorage.setItem.bind(window.localStorage);
const getItem = window.localStorage.getItem.bind(window.localStorage);

export class DataManager {

}

export class CacheManager {
	static designThemeName = "design_theme";
	static languageName = "ui_language";
	static activeTabName = "active_tab";

	static calcChoicesName = "calc_choices";

	static searchFavoriteName = "search_favorites";
	static resultFavoriteName = "result_favorites";

	static anomalyFilenameName = "anomaly_filename";
	static talismanFilenameName = "talisman_filename";

	static manualAnomaliesName = "manual_anomalies";
	static manualTalismansName = "manual_talismans";

	static bannedDecosName = "banned_decos";

	static defaultCalcChoices: CalcChoices = {
		sexType: "",
		weaponSlots: [0, 0, 0],
		selectedSkills: {},
		freeSlots: [0, 0, 0, 0],
	};

	static setDesignTheme(theme: string) {
		setItem(this.designThemeName, theme);
	}

	static getDesignTheme() {
		return getItem(this.designThemeName) ?? "light";
	}

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
		const realFavs = favs.filter(fav => fav !== null && fav !== undefined);

		setItem(this.resultFavoriteName, JSON.stringify(realFavs));
	}

	static getResultFavorites() {
		try {
			const rawOldObj = JSON.parse(getItem(this.resultFavoriteName)!) as ResultFavorite[] ?? [];
			const oldObj = rawOldObj.filter(obj => obj !== null && obj !== undefined);

			// Legacy snake case to camel case
			for (const obj of oldObj) {
				this.changeToCamelCase(obj.decoComb);

				for (const part in obj.armors) {
					this.changeToCamelCase(obj.armors[part]);
				}
			}

			return oldObj;
		} catch (e) {
			console.error(e);
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

	static setBannedDecos(decos: string[]) {
		setItem(this.bannedDecosName, JSON.stringify(decos));
	}

	static getBannedDecos() {
		try {
			return JSON.parse(getItem(this.bannedDecosName)!) as string[] ?? [];
		} catch (e) {
			return [];
		}
	}

	static changeToCamelCase(obj: any) {
		for (const snakeKey in obj) {
			const camelKey = snakeKey.replace(/_([a-zA-Z])/g, snakePart => {
				return snakePart[1].toUpperCase();
			});

			if (snakeKey === camelKey) {
				continue;
			}

			(obj as any)[camelKey] = (obj as any)[snakeKey];
			delete (obj as any)[snakeKey];
		}
	}
}

export const cm = new CacheManager();