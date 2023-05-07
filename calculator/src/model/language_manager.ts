import { Language } from "../definition/language";
import UIData from "../ui_data/ui_data.json";

export class LanguageManager {
	private language: Language;
	private data: { [key: string]: { [key: string]: string } }

	constructor() {
		this.language = "en";
		this.data = UIData;
	}

	setLanguage(lang: Language) {
		this.language = lang;
	}

	getString(key: string) {
		const value = this.data[key];

		if (value === undefined) {
			console.error(`LanguageManager: key '${key}' not found`);
			return "";
		}

		const langValue = value[this.language];

		if (langValue === undefined) {
			console.error(`LanguageManager: key '${key}' not found for language '${this.language}'`)
			return "";
		}

		return langValue;
	}
}

export const lm = new LanguageManager();