// Builds src-tauri/data/emoji.tsv from emojibase-data (MIT, based on Unicode CLDR):
// one line per emoji with English and German names, tags and shortcodes, in
// the official order. The Rust core searches it for ":" suggestions in chat.
//
//   node scripts/emoji/build.mjs
import { readFileSync, writeFileSync, mkdirSync } from "node:fs";
import { createRequire } from "node:module";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const require = createRequire(import.meta.url);
const root = join(dirname(fileURLToPath(import.meta.url)), "..", "..");
const load = (p) => JSON.parse(readFileSync(require.resolve(`emojibase-data/${p}`), "utf8"));

const en = load("en/compact.json");
const de = new Map(load("de/compact.json").map((e) => [e.hexcode, e]));
const enCodes = [load("en/shortcodes/github.json"), load("en/shortcodes/emojibase.json")];
const deCodes = [load("de/shortcodes/cldr.json")];
const version = JSON.parse(readFileSync(require.resolve("emojibase-data/package.json"), "utf8")).version;

const COMPONENT = 2; // skin tones and hair styles on their own
const codes = (maps, hex) => maps.flatMap((m) => [m[hex] ?? []].flat());
const clean = (s) => s.replace(/[\t\n]/g, " ").trim();
const words = (...parts) => [...new Set(parts.flat().filter(Boolean).map((w) => clean(String(w)).toLowerCase()))].join(" ");

const lines = [`# emojibase-data ${version} (MIT, https://emojibase.dev) – unicode\ten name\ten words\tde name\tde words`];
for (const e of en.sort((a, b) => (a.order ?? 1e9) - (b.order ?? 1e9))) {
  if (e.group === COMPONENT || e.order == null) continue;
  const d = de.get(e.hexcode);
  lines.push(
    [
      e.unicode,
      clean(e.label),
      words(codes(enCodes, e.hexcode), e.tags ?? [], e.emoticon ?? []),
      clean(d?.label ?? e.label),
      words(codes(deCodes, e.hexcode), d?.tags ?? []),
    ].join("\t"),
  );
}
mkdirSync(join(root, "src-tauri", "data"), { recursive: true });
const out = join(root, "src-tauri", "data", "emoji.tsv");
writeFileSync(out, lines.join("\n") + "\n");
console.log(`${lines.length - 1} emojis → ${out}`);
