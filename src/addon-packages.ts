import type { AddonInfo, AddonStatus, UpdateCheckResult } from "@/types";

const statusPriority: Record<AddonStatus, number> = {
  update: 5,
  current: 4,
  updating: 3,
  checking: 2,
  error: 1,
  untracked: 0,
};

function uniqueFolders(folders: string[]) {
  const seen = new Set<string>();
  return folders.filter((folder) => {
    const key = folder.toLowerCase();
    if (seen.has(key)) return false;
    seen.add(key);
    return true;
  });
}

function applyResult(
  addon: AddonInfo,
  result: UpdateCheckResult | undefined,
): AddonInfo {
  if (!result) return { ...addon, status: "error" };
  return {
    ...addon,
    status: result.status,
    title: result.title || addon.title,
    author: result.author || addon.author,
    notes: result.summary || addon.notes,
    source: result.sourceId ? "curseforge" : addon.source,
    sourceId: result.sourceId || addon.sourceId,
    packageFolders: result.packageFolders ?? addon.packageFolders,
    inferredFolders: result.sourceId ? [] : addon.inferredFolders,
    latestVersion: result.latestVersion,
    latestFileId: result.latestFileId,
    latestDownloadUrl: result.downloadUrl,
    websiteUrl: result.websiteUrl,
    error: result.error,
  };
}

function strongestStatus(addons: AddonInfo[]): AddonStatus {
  return addons.reduce(
    (status, addon) =>
      statusPriority[addon.status] > statusPriority[status]
        ? addon.status
        : status,
    "untracked" as AddonStatus,
  );
}

export function applyAndGroupUpdateResults(
  addons: AddonInfo[],
  results: UpdateCheckResult[],
): AddonInfo[] {
  const resultsByAddon = new Map(
    results.map((result) => [result.addonId, result]),
  );
  const checked = addons.map((addon) =>
    applyResult(addon, resultsByAddon.get(addon.id)),
  );

  const projectsByFolder = new Map<string, Set<string>>();
  const resultsByProject = new Map<string, UpdateCheckResult>();
  for (const result of results) {
    if (!result.sourceId) continue;
    const existing = resultsByProject.get(result.sourceId);
    if (
      !existing ||
      (result.packageFolders?.length ?? 0) >
        (existing.packageFolders?.length ?? 0)
    ) {
      resultsByProject.set(result.sourceId, result);
    }
    for (const folder of result.packageFolders ?? []) {
      const key = folder.toLowerCase();
      const projects = projectsByFolder.get(key) ?? new Set<string>();
      projects.add(result.sourceId);
      projectsByFolder.set(key, projects);
    }
  }

  const grouped = new Map<string, AddonInfo[]>();
  for (const addon of checked) {
    let sourceId = addon.source === "curseforge" ? addon.sourceId : undefined;
    if (!sourceId) {
      const candidates = new Set<string>();
      for (const folder of uniqueFolders([
        addon.folderName,
        ...addon.folders,
      ])) {
        const projects = projectsByFolder.get(folder.toLowerCase());
        if (projects?.size === 1) {
          candidates.add(projects.values().next().value as string);
        }
      }
      if (candidates.size === 1) {
        sourceId = candidates.values().next().value;
      }
    }
    const key = sourceId ? `curseforge:${sourceId}` : `addon:${addon.id}`;
    const members = grouped.get(key) ?? [];
    members.push(addon);
    grouped.set(key, members);
  }

  return Array.from(grouped.entries()).map(([key, members]) => {
    if (!key.startsWith("curseforge:")) return members[0];
    const sourceId = key.slice("curseforge:".length);
    const remote = resultsByProject.get(sourceId);
    const primary =
      members.find((addon) => addon.id === remote?.addonId) ??
      members.find((addon) => addon.sourceId === sourceId) ??
      members[0];
    const folders = uniqueFolders(
      members.flatMap((addon) => [addon.folderName, ...addon.folders]),
    );
    const packageFolders = uniqueFolders([
      ...(remote?.packageFolders ?? []),
      ...members.flatMap((addon) => addon.packageFolders ?? []),
    ]);
    const modifiedAt = members
      .map((addon) => addon.modifiedAt)
      .filter((value): value is string => Boolean(value))
      .sort()
      .at(-1);

    return {
      ...primary,
      id: key,
      source: "curseforge",
      sourceId,
      folders,
      inferredFolders: remote?.sourceId
        ? []
        : uniqueFolders(
            members.flatMap((addon) => addon.inferredFolders ?? []),
          ),
      packageFolders,
      status: strongestStatus(members),
      modifiedAt,
    };
  });
}
