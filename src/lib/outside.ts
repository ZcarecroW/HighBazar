import { onBeforeUnmount, watch, type Ref } from "vue";

/** Closes a popup on a click outside `root` – listening only while it is open. */
export function useCloseOutside(root: Ref<HTMLElement | undefined>, open: Ref<boolean>) {
  const onDown = (e: MouseEvent) => {
    if (root.value && !root.value.contains(e.target as Node)) open.value = false;
  };
  const stop = () => document.removeEventListener("mousedown", onDown);
  watch(open, (v) => (v ? document.addEventListener("mousedown", onDown) : stop()));
  onBeforeUnmount(stop);
}
