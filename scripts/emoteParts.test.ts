// Tests for src/lib/emoteParts.ts – run with: node scripts/emoteParts.test.ts
import assert from "node:assert/strict";
import { emoteParts } from "../src/lib/emoteParts.ts";

const known = { catJAM: "u/cat", RainTime: "u/rain", SteerR: "u/steer", Kappa: "u/kappa" };
const zw = new Set(["RainTime", "SteerR"]);
const parts = (text: string, local?: Record<string, string>) => emoteParts(text, local, known, zw);

assert.deepEqual(parts("hello chat"), [{ text: "hello chat", url: "" }]);
assert.deepEqual(parts("gg catJAM nice"), [
  { text: "gg ", url: "" },
  { text: "catJAM", url: "u/cat" },
  { text: " nice", url: "" },
]);

// A zero-width emote right after an emote is stacked on it (the space between disappears).
assert.deepEqual(parts("catJAM RainTime SteerR !"), [
  {
    text: "catJAM",
    url: "u/cat",
    over: [
      { text: "RainTime", url: "u/rain" },
      { text: "SteerR", url: "u/steer" },
    ],
  },
  { text: " !", url: "" },
]);

// On its own, or after a word, it stays a normal emote.
assert.deepEqual(parts("RainTime"), [{ text: "RainTime", url: "u/rain" }]);
assert.deepEqual(parts("wow RainTime"), [
  { text: "wow ", url: "" },
  { text: "RainTime", url: "u/rain" },
]);

// Native Twitch emotes of the message win, and a native emote is never stacked.
assert.deepEqual(parts("Kappa", { Kappa: "native/25" }), [{ text: "Kappa", url: "native/25" }]);

// Words that are names of object properties are plain text.
assert.deepEqual(parts("constructor toString __proto__", {}), [{ text: "constructor toString __proto__", url: "" }]);

console.log("emote parts tests passed");
