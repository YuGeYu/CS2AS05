import { invoke } from "@tauri-apps/api/core";
import type { Loadout, StickerInfo, KeychainInfo, StatTrakInfo } from "../utils/types";

export type { Loadout, StickerInfo, KeychainInfo, StatTrakInfo };

export interface AppConfig {
  language: string | null;
  cs2Path: string | null;
}

export interface PluginCheckResult {
  allPresent: boolean;
  missingFiles: string[];
  versionMismatch: boolean;
  deployedVersion: string | null;
  panelVersion: string;
  counterstrikesharpInstalled: boolean;
}

export interface UpdateCheckResult {
  currentVersion: string;
  latestVersion: string;
  updateAvailable: boolean;
  releaseUrl: string;
  releaseNotes: string;
}

export const api = {
  getConfig: () => invoke<AppConfig>("get_config"),
  saveConfig: (config: AppConfig) => invoke<void>("save_config", { config }),
  saveLoadout: (slot: number, loadout: Loadout) =>
    invoke<void>("save_loadout", { slot, loadout }),
  loadLoadout: (slot: number) =>
    invoke<Loadout | null>("load_loadout", { slot }),
  detectCs2Path: () => invoke<string | null>("detect_cs2_path"),
  checkPluginFiles: () => invoke<PluginCheckResult>("check_plugin_files"),
  deployAddons: () => invoke<string>("deploy_addons"),
  checkUpdate: () => invoke<UpdateCheckResult>("check_update"),
};
