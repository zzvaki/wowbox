export type GameFlavor =
  | "retail"
  | "classic"
  | "classic_era"
  | "classic_anniversary"
  | "classic_titan"
  | "classic_ptr"
  | "ptr"
  | "beta";

export type AddonSource = "curseforge" | "wowinterface" | "unknown";
export type AddonStatus =
  | "current"
  | "update"
  | "untracked"
  | "checking"
  | "updating"
  | "error";

export interface GameInstallation {
  id: string;
  flavor: GameFlavor;
  label: string;
  productFolder: string;
  path: string;
  addonsPath: string;
  addonCount: number;
  available: boolean;
}

export interface AddonInfo {
  id: string;
  title: string;
  notes: string;
  author: string;
  version: string;
  interfaceVersion: string;
  source: AddonSource;
  sourceId?: string;
  folderName: string;
  folders: string[];
  path: string;
  status: AddonStatus;
  latestVersion?: string;
  latestFileId?: string;
  latestDownloadUrl?: string;
  websiteUrl?: string;
  error?: string;
  modifiedAt?: string;
}

export interface UpdateCheckResult {
  addonId: string;
  status: "current" | "update" | "untracked" | "error";
  latestVersion?: string;
  latestFileId?: string;
  downloadUrl?: string;
  websiteUrl?: string;
  error?: string;
}

export interface UpdateRequest {
  addon: AddonInfo;
  downloadUrl: string;
  apiKey?: string;
}

export interface UpdateResult {
  addonId: string;
  version: string;
  backupPath: string;
  installedFolders: string[];
}

export interface AppSettings {
  gameRoot: string;
  clientPaths: Partial<Record<GameFlavor, string>>;
  curseforgeApiKey: string;
  rememberApiKey: boolean;
  checkOnLaunch: boolean;
}
