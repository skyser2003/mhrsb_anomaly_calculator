import { invoke } from "@tauri-apps/api/tauri";
import { AnomalyArmorInfo, TalismanInfo, EquipSlots, SexType, Skills, Slots, CalculateResult, CalculateAdditionalSkillsResult } from "../definition/calculate_result";
import { ArmorStatInfo } from "../definition/armor_define";

export interface AnomalyAddInfo {
	armorId: string;
	skills: {
		id: string;
		level: number;
	}[];
	slots: EquipSlots;
	stat: ArmorStatInfo;
};

export class InvokeManager {
	private static async invoke<T>(cmd: string, args: any): Promise<T> {
		return invoke<T>(cmd, args);
	}

	public static async calculateSkillset(anomalyFilename: string, talismanFilename: string, sexType: SexType, weaponSlots: EquipSlots, selectedSkills: Skills, freeSlots: Slots, includeLteEquips: boolean) {
		return this.invoke<{ result: CalculateResult, log: string }>("cmd_calculate_skillset", {
			anomalyFilename,
			talismanFilename,
			sexType,
			weaponSlots,
			selectedSkills,
			freeSlots,
			includeLteEquips,
		});
	}

	public static async calculateAdditionalSkillset(anomalyFilename: string, talismanFilename: string, sexType: SexType, weaponSlots: EquipSlots, selectedSkills: Skills, freeSlots: Slots, includeLteEquips: boolean) {
		return this.invoke<CalculateAdditionalSkillsResult>("cmd_calculate_additional_skills", {
			anomalyFilename,
			talismanFilename,
			sexType,
			weaponSlots,
			selectedSkills,
			freeSlots,
			includeLteEquips,
		});
	}

	public static async parseFileAnomaly(filename: string) {
		return this.invoke<AnomalyArmorInfo[]>("cmd_parse_anomaly", { filename });
	}

	public static async getFileAnomalies() {
		return this.invoke<AnomalyArmorInfo[]>("cmd_get_file_anomalies", {});
	}

	public static async clearFileAnomalies() {
		return this.invoke<void>("cmd_clear_file_anomalies", {});
	}

	public static async setManualAnomalies(anomalies: { [key: string]: AnomalyArmorInfo[] }) {
		return this.invoke<boolean>("cmd_set_manual_anomalies", { anomalies });
	}

	public static async addManualAnomaly(addInfo: AnomalyAddInfo) {
		return this.invoke<AnomalyArmorInfo>("cmd_add_manual_anomaly", {
			originalId: addInfo.armorId,
			skillDiffs: addInfo.skills,
			slotDiffs: addInfo.slots,
			statDiff: addInfo.stat,
		});
	}

	public static async clearManualAnomalies() {
		return this.invoke<boolean>("cmd_clear_manual_anomalies", {});
	}

	public static async parseFileTalisman(filename: string) {
		return this.invoke<TalismanInfo[]>("cmd_parse_talisman", { filename });
	}

	public static async clearFileTalismans() {
		return this.invoke<void>("cmd_clear_file_talismans", {});
	}

	public static async getFileTalismans() {
		return this.invoke<TalismanInfo[]>("cmd_get_file_talismans", {});
	}

	public static async setManualTalismans(talismans: TalismanInfo[]) {
		return this.invoke<boolean>("cmd_set_manual_talismans", { talismans });
	}

	public static async addManualTalisman(talisman: TalismanInfo) {
		return this.invoke<TalismanInfo>("cmd_add_manual_talisman", { talisman });
	}

	public static async clearManualTalismans() {
		return this.invoke<boolean>("cmd_clear_manual_talismans", {});
	}

	public static async setBannedDecos(decoIds: { [key: string]: boolean }) {
		return this.invoke<boolean>("cmd_set_banned_decos", { decoIds });
	}
}