// Tests for src/lib/chatInput.ts – run with: node scripts/chatInput.test.ts
import assert from "node:assert/strict";
import { activeToken, applySuggestion, enterPicks, type Suggestion } from "../src/lib/chatInput.ts";

const s = (insert: string, kind: Suggestion["kind"] = "emote"): Suggestion => ({ insert, label: insert, kind, provider: "7tv", url: "", scope: "", favorite: false });
const tok = (t: string) => activeToken(t, t.length);

assert.deepEqual(tok(":pog"), { mode: "colon", query: "pog", start: 0 });
assert.deepEqual(tok("gg :7 cat"), { mode: "colon", query: "7 cat", start: 3 });
assert.equal(tok("Hinweis: das"), null, "a colon inside a word does not open suggestions");
assert.equal(tok("Hinweis:"), null);
assert.equal(tok("lol :)"), null, "smileys stay smileys (Enter must send them)");
// ":D" and ":P" open suggestions, but Enter sends them unless one was chosen.
assert.equal(enterPicks(tok(":D")!, false), false);
assert.equal(enterPicks(tok(":D")!, true), true);
assert.equal(enterPicks(tok(":po")!, false), true);
assert.equal(enterPicks(tok("@a")!, false), true);
assert.deepEqual(tok(":t pika"), { mode: "colon", query: "t pika", start: 0 });
assert.deepEqual(tok(":"), { mode: "colon", query: "", start: 0 });
assert.deepEqual(tok("hi @zca"), { mode: "mention", query: "zca", start: 3 });
assert.equal(tok("mail@host"), null);

let r = applySuggestion("gg :po", 3, 6, s("PogChamp"));
assert.deepEqual(r, { text: "gg PogChamp ", caret: 12 });
r = applySuggestion(":po wow", 0, 3, s("Pog"));
assert.deepEqual(r, { text: "Pog wow", caret: 4 });
r = applySuggestion("hi @ma", 3, 6, s("@Max", "user"));
assert.equal(r.text, "hi @Max ");
console.log("chat input tests passed");
