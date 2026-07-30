import { describe, expect, it } from "vitest";
import { applyAndGroupUpdateResults } from "@/addon-packages";
import type { AddonInfo, UpdateCheckResult } from "@/types";

function localAddon(folderName: string): AddonInfo {
  return {
    id: `local:${folderName.toLowerCase()}`,
    title: folderName,
    notes: "",
    author: "",
    version: "1.0.0",
    interfaceVersion: "110200",
    source: "unknown",
    folderName,
    folders: [folderName],
    path: `/AddOns/${folderName}`,
    status: "untracked",
  };
}

describe("applyAndGroupUpdateResults", () => {
  it("merges every installed folder listed by the matched CurseForge file", () => {
    const addons = [
      localAddon("Details"),
      localAddon("Details_DataStorage"),
      localAddon("Details_EncounterDetails"),
    ];
    const results: UpdateCheckResult[] = [
      {
        addonId: "local:details",
        status: "update",
        title: "Details! Damage Meter",
        sourceId: "3358",
        latestVersion: "2.0.0",
        packageFolders: [
          "Details",
          "Details_DataStorage",
          "Details_EncounterDetails",
        ],
      },
      { addonId: "local:details_datastorage", status: "untracked" },
      { addonId: "local:details_encounterdetails", status: "untracked" },
    ];

    const grouped = applyAndGroupUpdateResults(addons, results);

    expect(grouped).toHaveLength(1);
    expect(grouped[0]).toMatchObject({
      id: "curseforge:3358",
      title: "Details! Damage Meter",
      source: "curseforge",
      sourceId: "3358",
      status: "update",
      folders: [
        "Details",
        "Details_DataStorage",
        "Details_EncounterDetails",
      ],
    });
  });

  it("does not merge a similar folder unless CurseForge lists it as a module", () => {
    const addons = [
      localAddon("Atlas"),
      localAddon("AtlasLoot"),
      localAddon("Atlas_Config"),
    ];
    const results: UpdateCheckResult[] = [
      {
        addonId: "local:atlas",
        status: "current",
        sourceId: "100",
        packageFolders: ["Atlas", "Atlas_Config"],
      },
      { addonId: "local:atlasloot", status: "untracked" },
      { addonId: "local:atlas_config", status: "untracked" },
    ];

    const grouped = applyAndGroupUpdateResults(addons, results);

    expect(grouped).toHaveLength(2);
    expect(grouped.find((addon) => addon.id === "curseforge:100")?.folders)
      .toEqual(["Atlas", "Atlas_Config"]);
    expect(grouped.some((addon) => addon.folderName === "AtlasLoot")).toBe(true);
  });
});
