import { nextTick, watch } from "vue";
import { useApp, type Jump, type View } from "../stores/app";

/** Scrolls to `#jump-<section>`, flashes it briefly and optionally focuses its first field. */
export function revealSection(section: string | undefined, focus = false) {
  if (!section) return;
  // Wait for the page (and lazy panels) to render.
  let tries = 0;
  const tick = () => {
    const el = document.getElementById(`jump-${section}`);
    if (!el) {
      if (++tries < 20) setTimeout(tick, 50);
      return;
    }
    el.scrollIntoView({ block: "start", behavior: "smooth" });
    el.classList.add("jump-flash");
    setTimeout(() => el.classList.remove("jump-flash"), 1600);
    if (focus) el.querySelector<HTMLElement>("input:not([type=range]):not([type=checkbox]), textarea, select")?.focus({ preventScroll: true });
  };
  void nextTick(tick);
}

/** Lets a page react to quick-search jumps (select a profile, open a tab or section). */
export function useJump(view: View, apply: (j: Jump) => void = () => {}) {
  const app = useApp();
  watch(
    () => app.jump,
    (j) => {
      if (!j || j.view !== view) return;
      apply(j);
      revealSection(j.section, j.focus);
      app.jump = null;
    },
    { immediate: true },
  );
}
