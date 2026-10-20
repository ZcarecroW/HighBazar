// Compiles every UI text with vue-i18n's own message compiler, so a stray "@",
// "{" or "|" fails here instead of in the released app – run: node scripts/i18n.test.ts
import { baseCompile } from "@intlify/message-compiler";
import de from "../src/i18n/de.ts";
import en from "../src/i18n/en.ts";

let n = 0;
const bad: string[] = [];
function walk(o: Record<string, unknown>, lang: string, path = "") {
  for (const [k, v] of Object.entries(o)) {
    const p = path ? `${path}.${k}` : k;
    if (v && typeof v === "object") walk(v as Record<string, unknown>, lang, p);
    else if (typeof v === "string") {
      n++;
      baseCompile(v, { onError: (e) => bad.push(`${lang} ${p}: ${e.message} – ${JSON.stringify(v).slice(0, 80)}`) });
    }
  }
}
walk(de, "de");
walk(en, "en");
if (bad.length) {
  console.error(bad.join("\n"));
  process.exit(1);
}
console.log(`${n} texts compile`);
