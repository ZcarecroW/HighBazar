// Fuzzy search used everywhere in the app: tolerant of case, accents/umlauts,
// gaps ("elr" → "Elden Ring") and small typos ("minecarft" → "Minecraft").
// Several words must each match somewhere ("ring night" → "Elden Ring Nightreign").

/** [start, end) ranges in the original text, for highlighting. */
export type Ranges = [number, number][];

export interface FuzzyResult<T> {
  item: T;
  score: number;
  /** Highlight ranges per searched field (same order as the keys). */
  ranges: Ranges[];
}

interface Folded {
  text: string;
  /** Index in the original string for each folded character. */
  map: number[];
}

const MARKS = /\p{M}/gu;

/** Lower case without accents (ä → a, ß → ss), keeping a map back to the original. */
export function fold(s: string): Folded {
  let text = "";
  const map: number[] = [];
  let i = 0;
  for (const ch of s) {
    let f = ch.toLowerCase();
    if (f === "ß") f = "ss";
    else f = f.normalize("NFD").replace(MARKS, "");
    for (let k = 0; k < f.length; k++) map.push(i);
    text += f;
    i += ch.length;
  }
  map.push(i);
  return { text, map };
}

const isWordChar = (c: string | undefined) => !!c && /[\p{L}\p{N}]/u.test(c);
/** Start of a word: after a non-letter or at a lower→upper case change (camelCase). */
function wordStarts(original: string, f: Folded): boolean[] {
  const starts: boolean[] = [];
  for (let k = 0; k < f.text.length; k++) {
    const prev = f.text[k - 1];
    const o = original[f.map[k]!] ?? "";
    const op = original[f.map[k]! - 1] ?? "";
    starts.push(k === 0 || !isWordChar(prev) || (o !== o.toLowerCase() && op === op.toLowerCase() && isWordChar(op)));
  }
  return starts;
}

function toRanges(f: Folded, idx: number[]): Ranges {
  const out: Ranges = [];
  for (const k of idx) {
    const a = f.map[k]!;
    const b = f.map[k + 1] ?? a + 1;
    const last = out[out.length - 1];
    if (last && last[1] >= a) last[1] = Math.max(last[1], b);
    else out.push([a, Math.max(b, a + 1)]);
  }
  return out;
}

/** Edit distance with transpositions (optimal string alignment), stops above `max`. */
function distance(a: string, b: string, max: number): number {
  if (Math.abs(a.length - b.length) > max) return max + 1;
  const d: number[][] = Array.from({ length: a.length + 1 }, (_, i) => [i]);
  for (let j = 1; j <= b.length; j++) d[0]![j] = j;
  for (let i = 1; i <= a.length; i++) {
    let rowMin = Infinity;
    for (let j = 1; j <= b.length; j++) {
      const cost = a[i - 1] === b[j - 1] ? 0 : 1;
      let v = Math.min(d[i - 1]![j]! + 1, d[i]![j - 1]! + 1, d[i - 1]![j - 1]! + cost);
      if (i > 1 && j > 1 && a[i - 1] === b[j - 2] && a[i - 2] === b[j - 1]) v = Math.min(v, d[i - 2]![j - 2]! + 1);
      d[i]![j] = v;
      rowMin = Math.min(rowMin, v);
    }
    if (rowMin > max) return max + 1;
  }
  return d[a.length]![b.length]!;
}

/** Scores one search term against one text; null = no match. */
function scoreTerm(term: string, f: Folded, starts: boolean[]): { score: number; idx: number[] } | null {
  const t = f.text;
  if (!term || !t) return null;
  // 1) Substring – best at the very start, then at a word start.
  let best: { score: number; idx: number[] } | null = null;
  for (let at = t.indexOf(term); at !== -1; at = t.indexOf(term, at + 1)) {
    const s = 100 + (at === 0 ? 40 : starts[at] ? 25 : 0) + (term.length / t.length) * 20 - Math.min(at, 40) * 0.2;
    if (!best || s > best.score) best = { score: s, idx: Array.from({ length: term.length }, (_, k) => at + k) };
  }
  if (best) return best;
  // 2) Letters in order ("elr" → Elden Ring); word starts and runs count more.
  if (term.length >= 2) {
    const firsts: number[] = [];
    for (let k = t.indexOf(term[0]!); k !== -1 && firsts.length < 12; k = t.indexOf(term[0]!, k + 1)) firsts.push(k);
    for (const first of firsts) {
      const idx = [first];
      let s = starts[first] ? 10 : 2;
      let pos = first;
      for (let q = 1; q < term.length && idx.length === q; q++) {
        // Prefer the next word start that has the letter, else the nearest one.
        let next = -1;
        for (let k = pos + 1; k < t.length; k++) {
          if (t[k] !== term[q]) continue;
          if (k === pos + 1 || starts[k]) {
            next = k;
            break;
          }
          if (next === -1) next = k;
        }
        // Letters scattered across a long text are chance, not a match.
        if (next === -1 || (next !== pos + 1 && !starts[next] && next - pos > 4)) break;
        s += next === pos + 1 ? 7 : starts[next] ? 8 : 1 - (next - pos - 1) * 0.4;
        idx.push(next);
        pos = next;
      }
      if (idx.length === term.length) {
        const sc = 40 + s - (first === 0 ? 0 : 3);
        if (!best || sc > best.score) best = { score: sc, idx };
      }
    }
    if (best) return best;
  }
  // 3) A small typo in a word ("minecarft", "fortnte").
  if (term.length >= 4) {
    const max = term.length >= 8 ? 2 : 1;
    const re = /[\p{L}\p{N}]+/gu;
    for (const m of t.matchAll(re)) {
      const word = m[0];
      // The whole word (one letter more or less), or its start while still typing.
      const lens = Math.abs(word.length - term.length) <= max ? [word.length] : [];
      if (word.length > term.length) lens.push(term.length);
      for (const len of lens) {
        if (len < 3) continue;
        const d = distance(term, word.slice(0, len), max);
        if (d <= max) {
          const s = 25 - d * 6 + (m.index === 0 ? 5 : 0);
          if (!best || s > best.score) best = { score: s, idx: Array.from({ length: len }, (_, k) => m.index! + k) };
        }
      }
    }
  }
  return best;
}

interface Prepared {
  original: string;
  f: Folded;
  starts: boolean[];
}
const cache = new Map<string, Prepared>();
function prepare(s: string): Prepared {
  let p = cache.get(s);
  if (!p) {
    const f = fold(s);
    p = { original: s, f, starts: wordStarts(s, f) };
    if (cache.size > 5000) cache.clear();
    cache.set(s, p);
  }
  return p;
}

/** Splits a query into folded terms. */
export function terms(query: string): string[] {
  return fold(query.trim()).text.split(/\s+/).filter(Boolean);
}

/**
 * Matches a query against the fields of one item. Every term has to match a
 * field; earlier fields weigh more (or pass `weights`).
 */
export function fuzzyMatch(query: string | string[], fields: (string | null | undefined)[], weights?: number[]): { score: number; ranges: Ranges[] } | null {
  const ts = Array.isArray(query) ? query : terms(query);
  const ranges: Ranges[] = fields.map(() => []);
  if (!ts.length) return { score: 0, ranges };
  const prepared = fields.map((x) => (x ? prepare(x) : null));
  let total = 0;
  for (const term of ts) {
    let best: { score: number; field: number; idx: number[] } | null = null;
    prepared.forEach((p, i) => {
      if (!p) return;
      const r = scoreTerm(term, p.f, p.starts);
      if (!r) return;
      const w = weights?.[i] ?? Math.max(0.55, 1 - i * 0.15);
      const s = r.score * w;
      if (!best || s > best.score) best = { score: s, field: i, idx: r.idx };
    });
    if (!best) return null;
    const b = best as { score: number; field: number; idx: number[] };
    total += b.score;
    ranges[b.field]!.push(...toRanges(prepared[b.field]!.f, b.idx));
  }
  return { score: total, ranges: ranges.map(merge) };
}

function merge(r: Ranges): Ranges {
  const s = [...r].sort((a, b) => a[0] - b[0]);
  const out: Ranges = [];
  for (const x of s) {
    const last = out[out.length - 1];
    if (last && x[0] <= last[1]) last[1] = Math.max(last[1], x[1]);
    else out.push([x[0], x[1]]);
  }
  return out;
}

/**
 * Filters and sorts items by a query (best first; ties keep their order).
 * An empty query returns all items unscored.
 */
export function fuzzySearch<T>(
  items: readonly T[],
  query: string,
  keys: (item: T) => (string | null | undefined)[],
  opts: { limit?: number; weights?: number[] } = {},
): FuzzyResult<T>[] {
  const ts = terms(query);
  if (!ts.length) {
    const all = items.map((item) => ({ item, score: 0, ranges: keys(item).map(() => [] as Ranges) }));
    return opts.limit ? all.slice(0, opts.limit) : all;
  }
  const out: (FuzzyResult<T> & { i: number })[] = [];
  items.forEach((item, i) => {
    const m = fuzzyMatch(ts, keys(item), opts.weights);
    if (m) out.push({ item, score: m.score, ranges: m.ranges, i });
  });
  out.sort((a, b) => b.score - a.score || a.i - b.i);
  // Far weaker hits next to good ones are noise (e.g. a typo match in a long description).
  // When even the best hit is only a typo match, the rest must come close to it.
  const top = out[0]?.score ?? 0;
  const floor = top * (top < 60 ? 0.6 : 0.3);
  const res = out.filter((r) => r.score >= floor).map(({ item, score, ranges }) => ({ item, score, ranges }));
  return opts.limit ? res.slice(0, opts.limit) : res;
}

/** Text split into plain and highlighted parts. */
export function segments(text: string, ranges: Ranges | undefined): { text: string; hit: boolean }[] {
  if (!ranges?.length) return [{ text, hit: false }];
  const out: { text: string; hit: boolean }[] = [];
  let pos = 0;
  for (const [a, b] of ranges) {
    if (a > pos) out.push({ text: text.slice(pos, a), hit: false });
    out.push({ text: text.slice(a, b), hit: true });
    pos = b;
  }
  if (pos < text.length) out.push({ text: text.slice(pos), hit: false });
  return out;
}
