<script setup lang="ts">
import { nextTick, onBeforeUnmount, onMounted, ref } from "vue";

// One tooltip for the whole app: every element with `data-tip` shows its text
// on hover (after a short pause), on keyboard focus, or pinned with a click.
// Event delegation keeps it to a few listeners however many tips a page has.
const text = ref("");
const pos = ref({ top: "0px", left: "0px" });
const box = ref<HTMLElement>();
let anchor: HTMLElement | null = null;
/** The tip the pointer is on while its delay runs. */
let pending: HTMLElement | null = null;
let timer = 0;
let pinned = false;

function tipOf(e: Event): HTMLElement | null {
  const el = e.target as Element | null;
  return el?.closest?.<HTMLElement>("[data-tip]") ?? null;
}

function show(a: HTMLElement) {
  clearTimeout(timer);
  pending = null;
  anchor = a;
  text.value = a.dataset.tip ?? "";
  void nextTick(place);
}

function hide() {
  clearTimeout(timer);
  pending = null;
  anchor = null;
  pinned = false;
  text.value = "";
}

/** Above the anchor, or below if there is no room; always inside the window. */
function place() {
  const tip = box.value;
  if (!tip || !anchor) return;
  const r = anchor.getBoundingClientRect();
  const m = 8;
  const vw = document.documentElement.clientWidth;
  let top = r.top - tip.offsetHeight - m;
  if (top < m) top = r.bottom + m;
  const left = Math.max(m, Math.min(r.left + r.width / 2 - tip.offsetWidth / 2, vw - tip.offsetWidth - m));
  pos.value = { top: `${Math.round(top)}px`, left: `${Math.round(left)}px` };
}

function onOver(e: PointerEvent) {
  const a = tipOf(e);
  // Moving over the icon's own shapes fires again: keep the running delay.
  if (!a || pinned || a === anchor || a === pending) return;
  clearTimeout(timer);
  pending = a;
  // Moving from one tip to the next shows it right away.
  timer = window.setTimeout(() => show(a), anchor ? 0 : 350);
}
function onOut(e: PointerEvent) {
  const a = tipOf(e);
  if (!a || pinned || a.contains(e.relatedTarget as Node | null)) return;
  clearTimeout(timer);
  pending = null;
  timer = window.setTimeout(hide, 100);
}
function onFocus(e: FocusEvent) {
  const a = tipOf(e);
  if (a?.matches(":focus-visible")) show(a);
}
function onBlur(e: FocusEvent) {
  if (!pinned && tipOf(e) === anchor) hide();
}
// Capture phase: runs before the info button stops the click.
function onClick(e: MouseEvent) {
  const a = tipOf(e);
  if (a?.classList.contains("tip-btn")) {
    if (pinned && anchor === a) hide();
    else {
      show(a);
      pinned = true;
    }
  } else if (pinned) hide();
}
function onKey(e: KeyboardEvent) {
  if (e.key === "Escape" && anchor) hide();
}
// Only when the anchor moves: the live feed scrolls by itself all the time.
function onScroll(e: Event) {
  const t = e.target;
  if (anchor && (t === document || (t instanceof Node && t.contains(anchor)))) hide();
}
function onResize() {
  if (anchor) hide();
}

const opts = { capture: true, passive: true } as const;
onMounted(() => {
  document.addEventListener("pointerover", onOver, { passive: true });
  document.addEventListener("pointerout", onOut, { passive: true });
  document.addEventListener("focusin", onFocus);
  document.addEventListener("focusout", onBlur);
  document.addEventListener("click", onClick, true);
  document.addEventListener("keydown", onKey);
  window.addEventListener("scroll", onScroll, opts);
  window.addEventListener("resize", onResize, { passive: true });
});
onBeforeUnmount(() => {
  clearTimeout(timer);
  document.removeEventListener("pointerover", onOver);
  document.removeEventListener("pointerout", onOut);
  document.removeEventListener("focusin", onFocus);
  document.removeEventListener("focusout", onBlur);
  document.removeEventListener("click", onClick, true);
  document.removeEventListener("keydown", onKey);
  window.removeEventListener("scroll", onScroll, opts);
  window.removeEventListener("resize", onResize);
});
</script>

<template>
  <div v-if="text" id="hb-tip" ref="box" role="tooltip" class="tip-box" :style="pos">{{ text }}</div>
</template>
