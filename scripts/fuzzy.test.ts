// Tests for src/lib/fuzzy.ts – run with: node scripts/fuzzy.test.ts
import assert from "node:assert/strict";
import { fold, fuzzyMatch, fuzzySearch, segments } from "../src/lib/fuzzy.ts";

let n = 0;
function test(name: string, fn: () => void) {
  fn();
  n++;
  console.log("ok -", name);
}

const games = ["Minecraft", "Elden Ring Nightreign", "Elden Ring", "Fortnite", "Just Chatting", "League of Legends", "Rocket League", "Pokémon Legends: Z-A", "Straßenbahn Simulator"];
const top = (q: string) => fuzzySearch(games, q, (g) => [g])[0]?.item;

test("folds case, accents and ß", () => {
  assert.equal(fold("Pokémon Straße").text, "pokemon strasse");
  // Each folded char maps back to its original position.
  assert.deepEqual(fold("ß").map, [0, 0, 1]);
});

test("prefix beats a match in the middle", () => {
  assert.equal(top("league"), "League of Legends");
  assert.equal(top("leg"), "League of Legends");
});

test("letters in order and word starts", () => {
  assert.equal(top("ern"), "Elden Ring Nightreign");
  assert.equal(top("jc"), "Just Chatting");
  assert.equal(top("lol"), "League of Legends");
});

test("several words must all match", () => {
  assert.equal(top("ring night"), "Elden Ring Nightreign");
  assert.equal(fuzzySearch(games, "ring fortnite", (g) => [g]).length, 0);
});

test("typos are forgiven", () => {
  assert.equal(top("minecarft"), "Minecraft");
  assert.equal(top("fortnte"), "Fortnite");
  assert.equal(top("pokemon"), "Pokémon Legends: Z-A");
  assert.equal(top("strassenbahn"), "Straßenbahn Simulator");
  assert.equal(top("moderaton"), undefined);
  const r = fuzzySearch(["Gemini 3.8 Flash", "Nutzt dein angemeldetes Claude Code CLI"], "gemni", (x) => [x]);
  assert.deepEqual(r.map((x) => x.item), ["Gemini 3.8 Flash"]);
  const p = fuzzySearch(
    [["Ollama (lokal)", "Ollama (local) · llama3.2", "ollama Nutzt das Modell, das in Ollama läuft."], ["Codex CLI (ChatGPT-Abo)", "Codex CLI · gpt-6-luna", "codexCli Nutzt dein lokal installiertes Codex CLI"]],
    "olama",
    (x) => x,
  );
  assert.deepEqual(p.map((x) => x.item[0]), ["Ollama (lokal)"]);
});

test("no match returns nothing", () => {
  assert.equal(fuzzySearch(games, "zzzz", (g) => [g]).length, 0);
});

test("empty query keeps everything in order", () => {
  assert.deepEqual(fuzzySearch(games, "  ", (g) => [g]).map((r) => r.item), games);
});

test("earlier fields weigh more", () => {
  const users = [
    { name: "nightbot", title: "Moderation bot" },
    { name: "zcarecrow", title: "Nightreign runs tonight" },
  ];
  const r = fuzzySearch(users, "night", (u) => [u.name, u.title]);
  assert.equal(r[0]!.item.name, "nightbot");
  assert.equal(r.length, 2);
});

test("highlight ranges map to the original text", () => {
  const m = fuzzyMatch("pok", ["Pokémon"])!;
  assert.deepEqual(m.ranges[0], [[0, 3]]);
  const s = segments("Straßenbahn", fuzzyMatch("strasse", ["Straßenbahn"])!.ranges[0]);
  assert.deepEqual(s, [{ text: "Straße", hit: true }, { text: "nbahn", hit: false }]);
});

test("camelCase starts count as word starts", () => {
  const r = fuzzySearch(["xQcOW", "summit1g"], "qc", (u) => [u]);
  assert.equal(r[0]!.item, "xQcOW");
});



test("scattered letters and far weaker hits are dropped", () => {
  const items = ["Jetzt in den Infobereich (Tray)", "Ausgewogen – greift bei klaren Regelverstößen ein und warnt beim ersten Mal"];
  assert.deepEqual(fuzzySearch(items, "tray", (x) => [x]).map((r) => r.item), [items[0]]);
  assert.equal(fuzzySearch(["streamer ready"], "tray", (x) => [x]).length, 0);
});

console.log(`${n} fuzzy tests passed`);
