import type { AddonInfo, GameInstallation, UpdateCheckResult } from "@/types";

export const mockInstallations: GameInstallation[] = [
  {
    id: "retail",
    flavor: "retail",
    label: "正式服",
    productFolder: "_retail_",
    path: "/Applications/World of Warcraft/_retail_",
    addonsPath:
      "/Applications/World of Warcraft/_retail_/Interface/AddOns",
    addonCount: 12,
    available: true,
  },
  {
    id: "classic",
    flavor: "classic",
    label: "巫妖王之怒",
    productFolder: "_classic_",
    path: "/Applications/World of Warcraft/_classic_",
    addonsPath:
      "/Applications/World of Warcraft/_classic_/Interface/AddOns",
    addonCount: 8,
    available: true,
  },
  {
    id: "classic-era",
    flavor: "classic_era",
    label: "经典旧世",
    productFolder: "_classic_era_",
    path: "/Applications/World of Warcraft/_classic_era_",
    addonsPath:
      "/Applications/World of Warcraft/_classic_era_/Interface/AddOns",
    addonCount: 5,
    available: true,
  },
];

export const mockAddons: AddonInfo[] = [
  {
    id: "curseforge:3358",
    title: "Details! Damage Meter",
    notes: "强大、灵活的战斗统计与伤害分析工具。",
    author: "Terciob",
    version: "11.2.0.14010",
    interfaceVersion: "110200",
    source: "curseforge",
    sourceId: "3358",
    folderName: "Details",
    folders: ["Details", "Details_EncounterDetails", "Details_DataStorage"],
    path: "/Applications/World of Warcraft/_retail_/Interface/AddOns/Details",
    status: "current",
    latestVersion: "11.2.0.14010",
    modifiedAt: "2026-07-27T10:30:00Z",
  },
  {
    id: "curseforge:61284",
    title: "Mythic Dungeon Tools",
    notes: "创建、分享与管理史诗钥石地下城路线。",
    author: "Nnoggie",
    version: "5.4.3",
    interfaceVersion: "110200",
    source: "curseforge",
    sourceId: "61284",
    folderName: "MythicDungeonTools",
    folders: ["MythicDungeonTools"],
    path:
      "/Applications/World of Warcraft/_retail_/Interface/AddOns/MythicDungeonTools",
    status: "update",
    latestVersion: "5.5.1",
    latestFileId: "720001",
    latestDownloadUrl: "https://example.com/mdt.zip",
    modifiedAt: "2026-07-19T08:00:00Z",
  },
  {
    id: "curseforge:61351",
    title: "Plater Nameplates",
    notes: "高度可定制的姓名板，包含脚本与模组支持。",
    author: "Terciob",
    version: "610-Retail",
    interfaceVersion: "110200",
    source: "curseforge",
    sourceId: "61351",
    folderName: "Plater",
    folders: ["Plater"],
    path: "/Applications/World of Warcraft/_retail_/Interface/AddOns/Plater",
    status: "update",
    latestVersion: "612-Retail",
    latestFileId: "720002",
    latestDownloadUrl: "https://example.com/plater.zip",
    modifiedAt: "2026-07-20T16:15:00Z",
  },
  {
    id: "wowinterface:24608",
    title: "Leatrix Plus",
    notes: "一组实用、轻量且高度可配置的生活质量改进。",
    author: "Leatrix",
    version: "11.2.04",
    interfaceVersion: "110200",
    source: "wowinterface",
    sourceId: "24608",
    folderName: "Leatrix_Plus",
    folders: ["Leatrix_Plus"],
    path:
      "/Applications/World of Warcraft/_retail_/Interface/AddOns/Leatrix_Plus",
    status: "current",
    latestVersion: "11.2.04",
    modifiedAt: "2026-07-26T11:20:00Z",
  },
  {
    id: "local:betterwardrobe",
    title: "Better Wardrobe",
    notes: "扩展幻化收藏和套装浏览体验。",
    author: "SLOKnightfall",
    version: "4.1.2",
    interfaceVersion: "110107",
    source: "unknown",
    folderName: "BetterWardrobe",
    folders: ["BetterWardrobe"],
    path:
      "/Applications/World of Warcraft/_retail_/Interface/AddOns/BetterWardrobe",
    status: "untracked",
    modifiedAt: "2026-06-11T02:40:00Z",
  },
];

export function mockCheckUpdates(addons: AddonInfo[]): UpdateCheckResult[] {
  return addons.map((addon, index) => {
    if (addon.source === "unknown") {
      return { addonId: addon.id, status: "untracked" };
    }
    if (index === 1 || index === 2) {
      return {
        addonId: addon.id,
        status: "update",
        latestVersion: addon.latestVersion,
        latestFileId: addon.latestFileId,
        downloadUrl: addon.latestDownloadUrl,
      };
    }
    return {
      addonId: addon.id,
      status: "current",
      latestVersion: addon.version,
    };
  });
}
