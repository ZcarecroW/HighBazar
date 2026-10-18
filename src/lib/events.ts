import { isTauri } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

type Handler = (payload: any) => void;
const handlers = new Map<string, Set<Handler>>();

/** Subscribes to a backend event (or to the browser-preview mock bus). */
export async function onEvent<T>(name: string, cb: (payload: T) => void) {
  if (isTauri()) {
    return listen<T>(name, (e) => cb(e.payload));
  }
  if (!handlers.has(name)) handlers.set(name, new Set());
  handlers.get(name)!.add(cb);
  return () => handlers.get(name)?.delete(cb);
}

/** Used by the browser-preview mock only. */
export function mockEmit(name: string, payload: unknown) {
  handlers.get(name)?.forEach((h) => h(payload));
}
