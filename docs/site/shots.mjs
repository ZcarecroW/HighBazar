// Takes the screenshots for the documentation and the README from the browser
// preview (mocked backend, nothing touches Twitch or an AI):
//   node docs/site/shots.mjs            all shots
//   node docs/site/shots.mjs ai stream  only these
// Needs Microsoft Edge (or set HB_BROWSER=chrome); writes docs/screenshots/*.webp.
import { createServer } from "vite";
import { chromium } from "playwright-core";
import { mkdirSync, statSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const root = join(dirname(fileURLToPath(import.meta.url)), "../..");
const out = join(root, "docs/screenshots");
mkdirSync(out, { recursive: true });

const W = 1600, H = 1000;
const wait = (ms) => new Promise((r) => setTimeout(r, ms));

/** Clicks the first button whose text matches. */
const click = (re) => async (p) => {
  await p.getByRole("button", { name: re }).first().click();
  await wait(400);
};
/** Scrolls the element containing `text` to the top of its scroll container. */
const scrollTo = (text) => async (p) => {
  await p.getByText(text, { exact: true }).first().evaluate((el) => el.scrollIntoView({ block: "start" }));
  await wait(300);
};
/** Crop: bounding box of the matched elements plus padding. */
async function box(p, selectors, pad = 24) {
  const boxes = [];
  for (const s of selectors) {
    const b = await p.locator(s).first().boundingBox();
    if (b) boxes.push(b);
  }
  if (!boxes.length) throw new Error(`nothing to crop: ${selectors}`);
  const x = Math.max(0, Math.min(...boxes.map((b) => b.x)) - pad);
  const y = Math.max(0, Math.min(...boxes.map((b) => b.y)) - pad);
  const r = Math.min(W, Math.max(...boxes.map((b) => b.x + b.width)) + pad);
  const btm = Math.min(H, Math.max(...boxes.map((b) => b.y + b.height)) + pad);
  return { x, y, width: r - x, height: btm - y };
}

// name → query string, steps, optional crop
const SHOTS = {
  dashboard: { q: "view=dashboard" },
  "dashboard-light": { q: "view=dashboard&theme=light" },
  "dashboard-de": { q: "view=dashboard&theme=light&lang=de" }, // README.de
  wizard: { q: "view=dashboard&wizard" },
  "wizard-ai": { q: "view=dashboard&wizard", steps: [click(/^Next$/), click(/^Next$/), click(/^Next$/)] },
  channels: { q: "view=channels" },
  translation: { q: "view=channels", steps: [scrollTo("Languages & translation")] },
  companion: { q: "view=companion" },
  "companion-timing": { q: "view=companion", steps: [click(/^Timing$/)] },
  "companion-reactions": { q: "view=companion", steps: [click(/^Reactions$/)] },
  "companion-research": { q: "view=companion", steps: [click(/^Research & web search$/)] },
  moderation: { q: "view=moderation" },
  "moderation-rules": { q: "view=moderation", steps: [click(/^Instant rules$/)] },
  "moderation-log": { q: "view=moderation", steps: [click(/^Log$/)] },
  ai: { q: "view=ai" },
  "ai-websearch": { q: "view=ai", steps: [click(/^Web search$/)] },
  "ai-usage": { q: "view=ai", steps: [click(/^Usage$/)] },
  stream: { q: "view=stream&level=simple" },
  "stream-advanced": { q: "view=stream&level=advanced", steps: [scrollTo("Chat settings")] },
  "stream-expert": { q: "view=stream&level=expert", steps: [scrollTo("Channel point rewards")] },
  settings: { q: "view=settings" },
  overlay: { q: "view=settings", steps: [async (p) => { await p.locator("#jump-overlay").evaluate((el) => el.scrollIntoView({ block: "start" })); await wait(300); }] },
  themes: { q: "view=settings", crop: async (p) => box(p, ["text=Color theme >> xpath=ancestor::*[contains(@class,'card')][1]"], 0) },
  "chat-input": {
    q: "view=dashboard",
    steps: [async (p) => { const i = p.getByPlaceholder(/Message/).first(); await i.click(); await i.pressSequentially(":pog", { delay: 40 }); await wait(500); }],
    crop: async (p) => box(p, ["[role=listbox]", "textarea, input[placeholder^='Message']"], 16),
  },
  palette: {
    q: "view=dashboard",
    steps: [async (p) => { await p.keyboard.press("Control+k"); await wait(300); await p.keyboard.type("web search", { delay: 30 }); await wait(400); }],
    crop: async (p) => box(p, ["[role=dialog]"], 16),
  },
  tooltip: {
    q: "view=moderation",
    keepPointer: true,
    steps: [async (p) => { await p.locator(".tip-btn").first().hover(); await wait(900); }],
    crop: async (p) => box(p, [".tip-btn >> nth=0", "#hb-tip"], 20),
  },
};

const only = process.argv.slice(2);
const server = await createServer({ root, logLevel: "error", server: { port: 1430, strictPort: false } });
await server.listen();
const base = server.resolvedUrls.local[0];
const browser = await chromium.launch({ channel: process.env.HB_BROWSER ?? "msedge", headless: true });
let failed = 0;
try {
  for (const [name, s] of Object.entries(SHOTS)) {
    if (only.length && !only.includes(name)) continue;
    const theme = s.q.includes("theme=light") ? "light" : "dark";
    const page = await browser.newPage({ viewport: { width: W, height: H }, colorScheme: theme });
    const errors = [];
    page.on("pageerror", (e) => errors.push(String(e)));
    try {
      await page.goto(`${base}?demo&${s.q.includes("theme=") ? s.q : `${s.q}&theme=dark`}`);
      await page.waitForSelector("aside nav button");
      await wait(1200); // feed, emotes and fonts settle
      for (const step of s.steps ?? []) await step(page);
      if (!s.keepPointer) await page.mouse.move(W - 2, H - 2);
      const file = join(out, `${name}.webp`);
      const clip = s.crop ? await s.crop(page) : undefined;
      await page.screenshot({ path: file, type: "webp", quality: 82, clip });
      console.log(`${name}.webp ${(statSync(file).size / 1024).toFixed(0)} KB${errors.length ? ` – page errors: ${errors.join("; ")}` : ""}`);
    } catch (e) {
      failed++;
      console.error(`${name}: ${e.message.split("\n")[0]}`);
    } finally {
      await page.close();
    }
  }
} finally {
  await browser.close();
  await server.close();
}
process.exit(failed ? 1 : 0);
