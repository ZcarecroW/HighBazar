// Color themes. The CSS values live in style.css ([data-theme="…"]); these
// numbers are only used for the preview swatches in the theme picker.
export interface ThemeInfo {
  id: string;
  /** accent lightness (default 0.57) */
  bl?: number;
  /** accent hue / chroma */
  h: number;
  c: number;
  /** surface hue / chroma */
  sh: number;
  sc: number;
  /** secondary (icon) hue */
  h2: number;
}

export const THEMES: ThemeInfo[] = [
  { id: "bazaar", h: 293, c: 0.23, sh: 293, sc: 0.014, h2: 293 },
  { id: "midnight", h: 268, c: 0.18, sh: 262, sc: 0.02, h2: 230 },
  { id: "ocean", bl: 0.53, h: 238, c: 0.15, sh: 230, sc: 0.016, h2: 200 },
  { id: "aurora", bl: 0.52, h: 182, c: 0.12, sh: 200, sc: 0.014, h2: 150 },
  { id: "forest", bl: 0.52, h: 152, c: 0.14, sh: 145, sc: 0.014, h2: 125 },
  { id: "gold", h: 72, c: 0.14, sh: 75, sc: 0.018, h2: 85 },
  { id: "sunset", h: 42, c: 0.18, sh: 40, sc: 0.016, h2: 70 },
  { id: "crimson", h: 22, c: 0.2, sh: 15, sc: 0.014, h2: 40 },
  { id: "sakura", h: 355, c: 0.16, sh: 350, sc: 0.016, h2: 320 },
  { id: "cyber", h: 328, c: 0.26, sh: 275, sc: 0.03, h2: 200 },
];

export function themeSwatch(t: ThemeInfo) {
  return {
    accent: `oklch(${t.bl ?? 0.57} ${t.c} ${t.h})`,
    accent2: `oklch(0.72 ${t.c * 0.9} ${t.h2})`,
    light: `oklch(0.96 ${t.sc} ${t.sh})`,
    lightPanel: `oklch(0.995 ${t.sc * 0.25} ${t.sh})`,
    dark: `oklch(0.155 ${t.sc * 1.3} ${t.sh})`,
    darkPanel: `oklch(0.23 ${t.sc * 1.5} ${t.sh})`,
  };
}
