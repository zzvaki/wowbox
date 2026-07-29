import { invoke } from "@tauri-apps/api/core";
import { mockAddons, mockCheckUpdates, mockInstallations } from "@/data/mock";
import type {
  AddonInfo,
  GameInstallation,
  UpdateCheckResult,
  UpdateRequest,
  UpdateResult,
} from "@/types";

const inTauri = () => Boolean(window.__TAURI_INTERNALS__);
const delay = (milliseconds: number) =>
  new Promise((resolve) => window.setTimeout(resolve, milliseconds));

export async function detectInstallations(
  customRoot?: string,
): Promise<GameInstallation[]> {
  if (!inTauri()) {
    await delay(450);
    return mockInstallations;
  }
  return invoke<GameInstallation[]>("detect_installations", {
    customRoot: customRoot || null,
  });
}

export async function scanAddons(
  addonsPath: string,
  flavor: string,
): Promise<AddonInfo[]> {
  if (!inTauri()) {
    await delay(650);
    return mockAddons.map((addon) => ({ ...addon }));
  }
  return invoke<AddonInfo[]>("scan_addons", { addonsPath, flavor });
}

export async function checkAddonUpdates(
  addons: AddonInfo[],
  flavor: string,
  provider: string,
  userCurseforgeApiKey?: string,
): Promise<UpdateCheckResult[]> {
  if (!inTauri()) {
    await delay(900);
    return mockCheckUpdates(addons);
  }
  return invoke<UpdateCheckResult[]>("check_updates", {
    addons,
    flavor,
    provider,
    userCurseforgeApiKey: userCurseforgeApiKey || null,
  });
}

export async function installAddonUpdate(
  request: UpdateRequest,
): Promise<UpdateResult> {
  if (!inTauri()) {
    await delay(1050);
    return {
      addonId: request.addon.id,
      version: request.addon.latestVersion ?? request.addon.version,
      backupPath: ".wowbox-backups/mock",
      installedFolders: request.addon.folders,
    };
  }
  return invoke<UpdateResult>("update_addon", { request });
}

export async function chooseGameRoot(): Promise<string | null> {
  if (!inTauri()) {
    return "/Applications/World of Warcraft";
  }
  return invoke<string | null>("choose_game_root");
}

export async function syncAuthorizedGameRoots(paths: string[]): Promise<void> {
  if (!inTauri()) return;
  await invoke("sync_authorized_game_roots", { configuredPaths: paths });
}
