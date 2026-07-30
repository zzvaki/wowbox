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
export type PluginDataSource = "curseforge" | "wowinterface";
export type AppLocale = "zh-CN" | "zh-TW" | "en-US" | "ja-JP";
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
  remoteDetails?: AddonDetails;
  remoteRequestTraces?: AddonRequestTrace[];
  error?: string;
  modifiedAt?: string;
}

export interface UpdateCheckResult {
  addonId: string;
  status: "current" | "update" | "untracked" | "error";
  title?: string;
  author?: string;
  summary?: string;
  sourceId?: string;
  latestVersion?: string;
  latestFileId?: string;
  downloadUrl?: string;
  websiteUrl?: string;
  error?: string;
}

export interface AddonAuthor {
  name: string;
  url?: string;
}

export interface AddonDetails {
  projectId: string;
  name: string;
  slug: string;
  summary: string;
  description: string;
  authors: AddonAuthor[];
  categories: string[];
  downloadCount: number;
  thumbsUpCount: number;
  rating?: number;
  websiteUrl?: string;
  wikiUrl?: string;
  issuesUrl?: string;
  sourceUrl?: string;
  dateCreated?: string;
  dateModified?: string;
  dateReleased?: string;
}

export interface AddonRequestTrace {
  method: string;
  url: string;
  status: "success" | "error";
  statusCode?: number;
  durationMs: number;
  content: string;
  error?: string;
}

export interface AddonDetailsResponse {
  details?: AddonDetails;
  requests: AddonRequestTrace[];
  error?: string;
}

export interface UpdateRequest {
  addon: AddonInfo;
  downloadUrl: string;
}

export interface UpdateResult {
  addonId: string;
  version: string;
  backupPath: string;
  installedFolders: string[];
}

export interface DeleteAddonResult {
  addonId: string;
  trashPath: string;
  removedFolders: string[];
}

export interface AppSettings {
  language: AppLocale;
  gameRoot: string;
  clientPaths: Partial<Record<GameFlavor, string>>;
  pluginDataSource: PluginDataSource;
  curseforgeApiKey: string;
  rememberApiKey: boolean;
  checkOnLaunch: boolean;
}
