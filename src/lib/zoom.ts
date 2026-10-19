// UI zoom (whole window, native webview zoom) and chat zoom (live chat only,
// independent of the UI zoom).
import { isTauri } from "@tauri-apps/api/core";
import { getCurrentWebview } from "@tauri-apps/api/webview";

export const UI_ZOOM = { min: 0.7, max: 1.6, step: 0.05 };
export const CHAT_ZOOM = { min: 0.6, max: 2.5, step: 0.1 };

export function clampZoom(v: number, r: { min: number; max: number }) {
  if (!Number.isFinite(v)) return 1;
  return Math.round(Math.min(r.max, Math.max(r.min, v)) * 100) / 100;
}

export async function applyUiZoom(z: number) {
  const zoom = clampZoom(z, UI_ZOOM);
  if (isTauri()) await getCurrentWebview().setZoom(zoom);
  else document.documentElement.style.setProperty("zoom", String(zoom));
}

/** CSS zoom for the chat so that its size only depends on the chat zoom. */
export function chatScale(chatZoom: number, uiZoom: number) {
  return clampZoom(chatZoom, CHAT_ZOOM) / clampZoom(uiZoom, UI_ZOOM);
}
