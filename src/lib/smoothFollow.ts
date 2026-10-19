// Keeps a scroll container glued to its bottom and lets new content glide
// in instead of jumping, so the eye can keep reading while chat moves.
//
// The animation works on the distance to the bottom, not on scrollTop, so
// entries removed at the top (feed limit) don't disturb it. When more content
// arrives during a glide, a new glide starts from the current position and
// speed, so there is never a jerk.
import type { ChatScroll, ScrollEasing } from "./types";

const EASE: Record<Exclude<ScrollEasing, "spring">, (s: number) => number> = {
  // Starts gently, brakes gently.
  easeInOut: (s) => (s < 0.5 ? 4 * s * s * s : 1 - Math.pow(-2 * s + 2, 3) / 2),
  // Starts right away, brakes softly.
  easeOut: (s) => 1 - Math.pow(1 - s, 3),
  linear: (s) => s,
};

/** Distance to the bottom at time t (ms) of a glide that starts at d0 with velocity v0 (px/ms). */
export function glide(easing: ScrollEasing, durationMs: number, d0: number, v0: number, t: number): number {
  const dur = Math.max(1, durationMs);
  if (easing === "spring") {
    // Critically damped spring: settles within ~durationMs, never overshoots.
    const w = 6 / dur;
    return (d0 + (v0 + w * d0) * t) * Math.exp(-w * t);
  }
  const s = Math.min(1, Math.max(0, t / dur));
  // Hermite term carries the current speed into the new glide and fades it out.
  return d0 * (1 - EASE[easing](s)) + v0 * dur * s * (1 - s) * (1 - s);
}

export class SmoothFollow {
  private el: HTMLElement;
  private settings: () => ChatScroll;
  private raf = 0;
  private d0 = 0;
  private v0 = 0;
  private t0 = 0;
  private lastHeight = 0;
  private lastSet = -1;
  // Scroll events also come from content changes and the browser's scroll
  // anchoring; only those right after real input count as the user's.
  private lastInput = 0;
  private dragging = false;
  private resize: ResizeObserver;
  /** true while the view sticks to the bottom; false once the user scrolls up. */
  following = true;
  onFollowChange?: (following: boolean) => void;

  /** `inner` wraps all entries; any change of its size (new messages, translations
   *  or emote images that load later) is followed. */
  constructor(el: HTMLElement, inner: HTMLElement, settings: () => ChatScroll) {
    this.el = el;
    this.settings = settings;
    this.lastHeight = el.scrollHeight;
    this.resize = new ResizeObserver(() => this.contentChanged());
    this.resize.observe(inner);
    this.resize.observe(el);
    el.addEventListener("scroll", this.onScroll, { passive: true });
    el.addEventListener("wheel", this.onWheel, { passive: true });
    el.addEventListener("keydown", this.onInput);
    el.addEventListener("touchmove", this.onInput, { passive: true });
    el.addEventListener("pointerdown", this.onPointerDown);
    window.addEventListener("pointerup", this.onPointerUp);
  }

  destroy() {
    cancelAnimationFrame(this.raf);
    this.resize.disconnect();
    this.el.removeEventListener("scroll", this.onScroll);
    this.el.removeEventListener("wheel", this.onWheel);
    this.el.removeEventListener("keydown", this.onInput);
    this.el.removeEventListener("touchmove", this.onInput);
    this.el.removeEventListener("pointerdown", this.onPointerDown);
    window.removeEventListener("pointerup", this.onPointerUp);
  }

  private max() {
    return this.el.scrollHeight - this.el.clientHeight;
  }

  private distance() {
    return Math.max(0, this.max() - this.el.scrollTop);
  }

  private setDistance(d: number) {
    const top = Math.max(0, this.max() - d);
    this.lastSet = top;
    this.el.scrollTop = top;
  }

  /** Call after new content was rendered. */
  contentChanged() {
    const h = this.el.scrollHeight;
    if (h === this.lastHeight) {
      // Only the container changed size (window resized): stay at the bottom.
      if (this.following && !this.raf) this.setDistance(0);
      return;
    }
    this.lastHeight = h;
    if (!this.following) return;
    const cfg = this.settings();
    if (!cfg.smooth || cfg.durationMs <= 0 || matchMedia("(prefers-reduced-motion: reduce)").matches) {
      cancelAnimationFrame(this.raf);
      this.raf = 0;
      this.setDistance(0);
      return;
    }
    const now = performance.now();
    let d = this.distance();
    // A large backlog (very busy chat) is skipped instead of scrolled through.
    const limit = this.el.clientHeight * 1.5;
    if (d > limit) d = limit;
    // Speed of the running glide (px/ms, negative = moving towards the bottom).
    this.v0 = this.raf ? this.velocity(cfg) : 0;
    this.d0 = d;
    this.t0 = now;
    this.setDistance(d);
    if (!this.raf) this.raf = requestAnimationFrame(this.frame);
  }

  private velocity(cfg: ChatScroll) {
    const t = performance.now() - this.t0;
    const a = glide(cfg.easing, cfg.durationMs, this.d0, this.v0, t);
    const b = glide(cfg.easing, cfg.durationMs, this.d0, this.v0, t + 8);
    return (b - a) / 8;
  }

  private frame = (now: number) => {
    const cfg = this.settings();
    const t = now - this.t0;
    let d = glide(cfg.easing, cfg.durationMs, this.d0, this.v0, t);
    const done = cfg.easing === "spring" ? Math.abs(d) < 0.5 && t > 50 : t >= cfg.durationMs;
    if (done || d < 0) d = 0;
    this.setDistance(d);
    if (d === 0 && (done || cfg.easing !== "spring")) {
      this.raf = 0;
      return;
    }
    this.raf = requestAnimationFrame(this.frame);
  };

  private setFollowing(v: boolean) {
    if (this.following === v) return;
    this.following = v;
    this.onFollowChange?.(v);
  }

  /** Stop following (the user wants to read older messages). */
  stop() {
    cancelAnimationFrame(this.raf);
    this.raf = 0;
    this.setFollowing(false);
  }

  /** Jump (or glide) back to the newest message and keep following. */
  follow() {
    this.setFollowing(true);
    this.lastHeight = -1;
    this.contentChanged();
  }

  private onInput = () => {
    this.lastInput = performance.now();
  };

  private onPointerDown = () => {
    this.dragging = true;
    this.onInput();
  };

  private onPointerUp = () => {
    this.dragging = false;
  };

  private onWheel = (e: WheelEvent) => {
    // Ctrl + wheel zooms the chat; it is not scrolling.
    if (e.ctrlKey) return;
    this.onInput();
    // Scrolling up interrupts the glide right away.
    if (e.deltaY < 0 && this.following) {
      cancelAnimationFrame(this.raf);
      this.raf = 0;
      this.setFollowing(false);
    }
  };

  private onScroll = () => {
    // Our own scroll steps and content changes are not user input.
    if (Math.abs(this.el.scrollTop - this.lastSet) < 2) return;
    if (!this.dragging && performance.now() - this.lastInput > 300) return;
    const atBottom = this.distance() < 40;
    if (!atBottom) {
      cancelAnimationFrame(this.raf);
      this.raf = 0;
    }
    // Scrolling back down to the newest message resumes following.
    this.setFollowing(atBottom);
  };
}
