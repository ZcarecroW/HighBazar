import { ref } from "vue";
import { check, type Update } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";

export const pendingUpdate = ref<Update | null>(null);
export const updateProgress = ref<number | null>(null);
export const updateInstalling = ref(false);

/** Returns the update if one is available, null otherwise. */
export async function checkForUpdate(): Promise<Update | null> {
  const u = await check({ timeout: 20000 });
  pendingUpdate.value = u;
  return u;
}

export async function installUpdate() {
  const u = pendingUpdate.value;
  if (!u) return;
  let total = 0;
  let done = 0;
  updateProgress.value = 0;
  await u.downloadAndInstall((ev) => {
    if (ev.event === "Started") total = ev.data.contentLength ?? 0;
    else if (ev.event === "Progress") {
      done += ev.data.chunkLength;
      updateProgress.value = total ? Math.round((done / total) * 100) : null;
    } else if (ev.event === "Finished") updateInstalling.value = true;
  });
  await relaunch();
}
