// Builds the documentation website, in English (docs/site/pages/*.md) and
// German (docs/site/pages/de/*.md), into docs/site/dist, which GitHub Pages
// serves at https://zcarecrow.github.io/HighBazar/ (German under /de/).
//
//   node docs/site/build.mjs           build (fails on broken links or anchors)
//   node docs/site/build.mjs --serve   build and preview on http://localhost:4173/HighBazar/
//
// Both languages have the same pages under the same names, and links to anchors
// use the same ids (German headings carry the English id as {#id} where linked).
// Screenshots come from docs/screenshots and docs/screenshots/de (node
// docs/site/shots.mjs), icons from the app's own Nucleo set (src/icons/nucleo.ts)
// plus four brand icons in docs/site/icons. Nucleo icons are licensed to the
// HighBazar author and may not be extracted or reused (see README).
import { Marked } from "marked";
import { createHash } from "node:crypto";
import { cpSync, existsSync, mkdirSync, readdirSync, readFileSync, rmSync, statSync, writeFileSync } from "node:fs";
import { createServer } from "node:http";
import { dirname, extname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { nucleo } from "../../src/icons/nucleo.ts";
import { LANGS } from "./i18n.mjs";

const here = dirname(fileURLToPath(import.meta.url));
const root = join(here, "../..");
const dist = join(here, "dist");
const shotsDir = join(root, "docs/screenshots");
const pkg = JSON.parse(readFileSync(join(root, "package.json"), "utf8"));
const VERSION = pkg.version;
const SITE = "https://zcarecrow.github.io/HighBazar/";
const REPO = "https://github.com/ZcarecroW/HighBazar";
const RELEASE = `${REPO}/releases/download/v${VERSION}`;

// Sidebar: the order of the pages per section (file names without .md); the
// section titles are in i18n.mjs.
const NAV = [
  ["introduction", "installation", "setup-wizard", "first-stream"],
  ["dashboard", "chat-input", "channels", "companion", "moderation", "translation", "stream-manager", "overlay", "settings"],
  ["ai-providers", "web-search", "costs"],
  ["twitch-glossary", "shortcuts", "privacy", "troubleshooting", "faq", "changelog", "building"],
];
const ORDER = NAV.flat();
const PROVIDERS = ["Claude", "OpenAI GPT", "Gemini", "Grok", "OpenRouter", "Perplexity", "DeepSeek", "Mistral", "Groq", "Together AI", "Ollama", "LM Studio", "Claude Code CLI", "Codex CLI"];

// ---------------------------------------------------------------- helpers

const esc = (s) => String(s).replace(/[&<>"']/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" })[c]);
const strip = (html) => html.replace(/<[^>]+>/g, " ").replace(/&nbsp;/g, " ").replace(/&amp;/g, "&").replace(/&lt;/g, "<").replace(/&gt;/g, ">").replace(/&quot;/g, '"').replace(/&#39;/g, "'").replace(/\s+/g, " ").trim();
const slugify = (s) => strip(s).toLowerCase().replace(/ß/g, "ss").normalize("NFKD").replace(/[̀-ͯ]/g, "").replace(/[^a-z0-9]+/g, "-").replace(/^-|-$/g, "");
const hash = (s) => createHash("sha256").update(s).digest("hex").slice(0, 10);
const linkTo = (link) => link.replace(/^([\w-]+)/, "$1/"); // "faq#x" -> "faq/#x"

/** Inline SVG of an app icon (two-tone parts get class nc-c2). */
function icon(name, size = 20, cls = "ico") {
  const hit = nucleo[name];
  if (!hit) throw new Error(`unknown icon "${name}"`);
  return `<svg class="${cls}" viewBox="${hit[0]}" width="${size}" height="${size}" aria-hidden="true">${hit[1]}</svg>`;
}
const brand = Object.fromEntries(
  readdirSync(join(here, "icons")).map((f) => [f.replace(/\.svg$/, ""), readFileSync(join(here, "icons", f), "utf8").replace(/<svg /, '<svg class="ico" aria-hidden="true" ').replace(/ width="24" height="24"/, ' width="20" height="20"')]),
);

/** Width and height of a WebP file (VP8, VP8L, VP8X). */
function webpSize(file) {
  const b = readFileSync(file);
  const kind = b.toString("ascii", 12, 16);
  if (kind === "VP8X") return [1 + b.readUIntLE(24, 3), 1 + b.readUIntLE(27, 3)];
  if (kind === "VP8L") {
    const v = b.readUInt32LE(21);
    return [(v & 0x3fff) + 1, ((v >> 14) & 0x3fff) + 1];
  }
  if (kind === "VP8 ") return [b.readUInt16LE(26) & 0x3fff, b.readUInt16LE(28) & 0x3fff];
  throw new Error(`${file}: not a WebP file`);
}

/** `---\nkey: value\n---` front matter. */
function frontMatter(src, file) {
  const m = src.match(/^---\r?\n([\s\S]*?)\r?\n---\r?\n/);
  if (!m) throw new Error(`${file}: missing front matter`);
  const meta = {};
  for (const line of m[1].split(/\r?\n/)) {
    const i = line.indexOf(":");
    if (i > 0) meta[line.slice(0, i).trim()] = line.slice(i + 1).trim();
  }
  for (const k of ["title", "description", "icon"]) if (!meta[k]) throw new Error(`${file}: front matter needs "${k}"`);
  return { meta, body: src.slice(m[0].length) };
}

// GitHub-style callouts: "> [!TIP]" blocks become boxes with an icon.
const CALLOUT_ICONS = { NOTE: "Info", TIP: "Sparkles", IMPORTANT: "Bookmark", WARNING: "TriangleAlert", CAUTION: "ShieldAlert" };
function callouts(md, t) {
  return md.replace(/^> \[!(NOTE|TIP|IMPORTANT|WARNING|CAUTION)\][^\n]*\n((?:>[^\n]*\n?)*)/gm, (_, kind, body) => {
    const inner = body.replace(/^> ?/gm, "");
    return `<div class="callout ${kind.toLowerCase()}"><p class="callout-title">${icon(CALLOUT_ICONS[kind], 18)}${t.callouts[kind]}</p>\n\n${inner}\n</div>\n\n`;
  });
}

// ---------------------------------------------------------------- pages

const sites = {}; // lang -> { t, pages: Map, usedImages: Set }
for (const [lang, t] of Object.entries(LANGS)) {
  const dir = join(here, "pages", lang === "en" ? "" : lang);
  const pages = new Map();
  for (const f of readdirSync(dir).filter((f) => f.endsWith(".md"))) {
    const slug = f.replace(/\.md$/, "");
    const { meta, body } = frontMatter(readFileSync(join(dir, f), "utf8"), `${lang}/${f}`);
    pages.set(slug, { slug, lang, ...meta, md: body });
  }
  // The changelog is the repository's CHANGELOG.md (in English) without its title.
  pages.set("changelog", { slug: "changelog", lang, title: t.changelog.title, description: t.changelog.description, icon: "Clock", md: t.changelog.note + readFileSync(join(root, "CHANGELOG.md"), "utf8").replace(/^# Changelog\s*/, "") });
  for (const s of ORDER) if (!pages.has(s)) throw new Error(`${lang}: NAV lists "${s}" but its page is missing in ${dir}`);
  for (const s of pages.keys()) if (!ORDER.includes(s)) throw new Error(`${lang}: page "${s}" is not in NAV`);
  sites[lang] = { t, pages, usedImages: new Set(), links: [] };
}
const sectionOf = (slug, t) => t.nav[NAV.findIndex((s) => s.includes(slug))];
const shotFile = (lang, name) => join(shotsDir, lang === "en" ? "" : lang, name);
const shotUrl = (lang, name) => `images/${lang === "en" ? "" : `${lang}/`}${name}`;

function render(page, site) {
  const { t } = site;
  const toc = [];
  const ids = new Set();
  const marked = new Marked({ gfm: true });
  marked.use({
    renderer: {
      heading({ tokens, depth, text }) {
        const html = this.parser.parseInline(tokens);
        const custom = text.match(/\{#([\w-]+)\}\s*$/);
        const id = custom ? custom[1] : slugify(text) || "section";
        let unique = id;
        for (let n = 2; ids.has(unique); n++) unique = `${id}-${n}`;
        ids.add(unique);
        const label = html.replace(/\s*\{#[\w-]+\}\s*$/, "");
        if (depth === 2 || depth === 3) toc.push({ depth, id: unique, text: strip(label) });
        return `<h${depth} id="${unique}"><a class="anchor" href="#${unique}" aria-label="${esc(t.anchor)}">#</a>${label}</h${depth}>\n`;
      },
      image({ href, title, text }) {
        if (/^https?:/.test(href)) return `<img src="${esc(href)}" alt="${esc(text)}" loading="lazy">`;
        const file = shotFile(page.lang, href);
        if (!existsSync(file)) throw new Error(`${page.lang}/${page.slug}: missing screenshot ${href} (run node docs/site/shots.mjs)`);
        site.usedImages.add(href);
        const [w, h] = webpSize(file);
        const src = `${page.rel}${shotUrl(page.lang, href)}`;
        return `<figure class="shot${w < 1100 ? " small" : ""}"><a href="${esc(src)}" data-lightbox><img src="${esc(src)}" alt="${esc(text)}" width="${w}" height="${h}" loading="lazy" decoding="async"></a>${title || text ? `<figcaption>${esc(title || text)}</figcaption>` : ""}</figure>`;
      },
      link({ href, title, tokens }) {
        const inner = this.parser.parseInline(tokens);
        if (/^(https?:|mailto:)/.test(href)) return `<a href="${esc(href)}"${title ? ` title="${esc(title)}"` : ""} rel="noopener">${inner}</a>`;
        const m = href.match(/^([\w-]*)(?:\.md)?(?:#([\w-]+))?$/);
        if (!m) throw new Error(`${page.lang}/${page.slug}: unsupported link "${href}"`);
        const target = m[1] || page.slug;
        site.links.push([page.slug, target, m[2] ?? ""]);
        const url = (target === page.slug ? "" : `../${target === "index" ? "" : `${target}/`}`) + (m[2] ? `#${m[2]}` : "");
        return `<a href="${url || "#"}">${inner}</a>`;
      },
      table(token) {
        const head = token.header.map((c) => `<th${c.align ? ` style="text-align:${c.align}"` : ""}>${this.parser.parseInline(c.tokens)}</th>`).join("");
        const rows = token.rows.map((r) => `<tr>${r.map((c) => `<td${c.align ? ` style="text-align:${c.align}"` : ""}>${this.parser.parseInline(c.tokens)}</td>`).join("")}</tr>`).join("\n");
        return `<div class="table-wrap"><table><thead><tr>${head}</tr></thead><tbody>${rows}</tbody></table></div>\n`;
      },
      code({ text, lang }) {
        return `<pre${lang ? ` data-lang="${esc(lang)}"` : ""}><code>${esc(text)}</code></pre>\n`;
      },
    },
  });
  // {icon:Name} anywhere in the text; {#id} outside headings marks a link target (e.g. a table row).
  const md = callouts(page.md, t)
    .replace(/\{icon:(\w+)\}/g, (_, n) => icon(n, 18, "ico inline"))
    .split("\n")
    .map((l) => (/^#{1,6} /.test(l) ? l : l.replace(/\s*\{#([\w-]+)\}/g, (_, id) => (ids.add(id), `<span id="${id}" class="target"></span>`))))
    .join("\n");
  page.html = marked.parse(md);
  page.toc = toc;
  page.ids = ids;
}

for (const site of Object.values(sites)) {
  for (const p of site.pages.values()) {
    p.rel = site.t.dir ? "../../" : "../";
    render(p, site);
  }
  for (const [from, to, anchor] of site.links) checkLink(site, from, to, anchor);
}

function checkLink(site, from, to, anchor) {
  if (to === "index") return;
  const t = site.pages.get(to);
  if (!t) throw new Error(`${site.t.code}/${from}: link to missing page "${to}"`);
  if (anchor && !t.ids.has(anchor)) throw new Error(`${site.t.code}/${from}: link to missing anchor "${to}#${anchor}"`);
}

// ---------------------------------------------------------------- layout

const asset = {}; // name -> file name with content hash

/** URL of the same page in a language, relative to a page at depth `rel`. */
const alt = (rel, lang, slug) => `${rel}${LANGS[lang].dir}${slug ? `${slug}/` : ""}`;

function head({ t, title, description, rel, slug, image = "dashboard.webp" }) {
  const canonical = `${SITE}${t.dir}${slug ? `${slug}/` : ""}`;
  const alternates = Object.values(LANGS).map((l) => `<link rel="alternate" hreflang="${l.code}" href="${SITE}${l.dir}${slug ? `${slug}/` : ""}">`).join("\n");
  return `<!doctype html>
<html lang="${t.code}">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>${esc(title)}</title>
<meta name="description" content="${esc(description)}">
<link rel="canonical" href="${canonical}">
${alternates}
<link rel="alternate" hreflang="x-default" href="${SITE}${slug ? `${slug}/` : ""}">
<meta name="theme-color" content="#14101c">
<meta property="og:type" content="website">
<meta property="og:site_name" content="HighBazar">
<meta property="og:locale" content="${t.code === "de" ? "de_DE" : "en_US"}">
<meta property="og:title" content="${esc(title)}">
<meta property="og:description" content="${esc(description)}">
<meta property="og:url" content="${canonical}">
<meta property="og:image" content="${SITE}${shotUrl(t.code, image)}">
<meta name="twitter:card" content="summary_large_image">
<link rel="icon" href="${rel}favicon.svg" type="image/svg+xml">
<link rel="apple-touch-icon" href="${rel}apple-touch-icon.png">
<script>document.documentElement.classList.add("js");try{var t=localStorage.getItem("hb-docs-theme");if(t)document.documentElement.dataset.theme=t}catch(e){}</script>
<link rel="stylesheet" href="${rel}${asset.css}">
<script src="${rel}${asset.js}" defer></script>
</head>`;
}

function topbar(t, rel, slug) {
  const home = `${rel}${t.dir}`;
  const other = Object.values(LANGS).find((l) => l.code !== t.code);
  return `<a class="skip" href="#main">${esc(t.skip)}</a>
<header class="topbar">
  <button class="icon-btn menu-btn" type="button" aria-label="${esc(t.menu)}" aria-expanded="false" data-menu>${icon("Layers", 20)}</button>
  <a class="brand" href="${home || "./"}"><img src="${rel}favicon.svg" alt="" width="30" height="30"><span>HighBazar</span><span class="brand-tag">${esc(t.tag)}</span></a>
  <button class="search-btn" type="button" data-search>${icon("Search", 16)}<span>${esc(t.search)}</span><kbd>Ctrl K</kbd></button>
  <nav class="top-links" aria-label="Site">
    <a href="${home}introduction/" class="hide-xs"${slug && slug !== "404" ? ' aria-current="true"' : ""}>${esc(t.guide)}</a>
    <a href="${home}changelog/" class="hide-sm">v${VERSION}</a>
    <a class="lang-switch" href="${alt(rel, other.code, slug === "404" ? "" : slug)}" hreflang="${other.code}" lang="${other.code}" title="${esc(t.otherLabel)}" data-lang="${other.code}">${icon("Languages", 17)}<span>${other.code.toUpperCase()}</span></a>
    <a class="icon-btn hide-xs" href="${REPO}" aria-label="${esc(t.github)}" rel="noopener">${brand["logo-github"]}</a>
    <button class="icon-btn" type="button" data-theme-toggle aria-label="${esc(t.themeToggle)}">${icon("Moon", 18, "ico when-light")}${icon("Sun", 18, "ico when-dark")}</button>
    <a class="btn btn-primary btn-sm hide-xs" href="${home}installation/">${icon("Download", 16)}${esc(t.download)}</a>
  </nav>
</header>
<div class="lang-hint" hidden data-hint-for="${other.code}"><span>${esc(t.hint)}</span><a href="${alt(rel, other.code, slug === "404" ? "" : slug)}" data-lang="${other.code}">${esc(other.name)} →</a><button class="icon-btn" type="button" aria-label="${esc(t.close)}" data-hint-close>${icon("X", 16)}</button></div>`;
}

function footer(t, rel) {
  const home = `${rel}${t.dir}`;
  const fl = t.footerLinks;
  return `<footer class="site-footer">
  <div class="footer-inner">
    <div class="footer-brand"><img src="${rel}favicon.svg" alt="" width="36" height="36"><div><strong>HighBazar</strong><p>${esc(t.footerBlurb)}</p></div></div>
    <nav aria-label="Footer">
      <div><p>${esc(t.footerDocs)}</p><a href="${home}introduction/">${fl.introduction}</a><a href="${home}installation/">${fl.installation}</a><a href="${home}first-stream/">${fl["first-stream"]}</a><a href="${home}faq/">${fl.faq}</a></div>
      <div><p>${esc(t.footerProject)}</p><a href="${REPO}" rel="noopener">GitHub</a><a href="${REPO}/releases/latest" rel="noopener">${fl.releases}</a><a href="${REPO}/issues" rel="noopener">${fl.issues}</a><a href="${home}changelog/">${fl.changelog}</a></div>
    </nav>
  </div>
  <p class="fine">${t.fine}</p>
</footer>
<dialog class="search" aria-label="${esc(t.searchLabel)}">
  <div class="search-box">${icon("Search", 18)}<input type="search" placeholder="${esc(t.searchPlaceholder)}" autocomplete="off" spellcheck="false" aria-label="${esc(t.search)}"><kbd>Esc</kbd></div>
  <ul class="search-results" role="listbox"></ul>
  <p class="search-hint"><span><kbd>↑</kbd><kbd>↓</kbd> ${esc(t.choose)}</span><span><kbd>Enter</kbd> ${esc(t.open)}</span></p>
</dialog>
<div class="lightbox" hidden><img alt=""><button class="icon-btn" type="button" aria-label="${esc(t.close)}">${icon("X", 22)}</button></div>`;
}

function sidebar(site, current, rel) {
  const home = `${rel}${site.t.dir}`;
  return NAV.map((slugs, i) => `<div class="nav-group"><p>${esc(site.t.nav[i])}</p>${slugs.map((s) => {
    const p = site.pages.get(s);
    return `<a href="${home}${s}/"${s === current ? ' aria-current="page"' : ""}>${icon(p.icon, 18)}<span>${esc(p.title)}</span></a>`;
  }).join("")}</div>`).join("");
}

function docPage(site, p) {
  const { t } = site;
  const rel = p.rel;
  const i = ORDER.indexOf(p.slug);
  const prev = site.pages.get(ORDER[i - 1]);
  const next = site.pages.get(ORDER[i + 1]);
  const toc = p.toc.length > 1 ? `<aside class="toc" aria-label="${esc(t.onThisPage)}"><p>${esc(t.onThisPage)}</p><ul>${p.toc.map((h) => `<li class="d${h.depth}"><a href="#${h.id}">${esc(h.text)}</a></li>`).join("")}</ul></aside>` : `<aside class="toc"></aside>`;
  const source = p.slug === "changelog" ? `${REPO}/blob/main/CHANGELOG.md` : `${REPO}/edit/main/docs/site/pages/${t.dir}${p.slug}.md`;
  return `${head({ t, title: `${p.title} · ${t.docs}`, description: p.description, rel, slug: p.slug })}
<body class="doc">
${topbar(t, rel, p.slug)}
<div class="layout">
  <aside class="sidebar" aria-label="${esc(t.sidebar)}"><nav>${sidebar(site, p.slug, rel)}</nav></aside>
  <main id="main" class="content">
    <header class="page-head">
      <p class="eyebrow">${esc(sectionOf(p.slug, t))}</p>
      <h1><span class="page-icon">${icon(p.icon, 26)}</span>${esc(p.title)}</h1>
      <p class="lead">${esc(p.description)}</p>
    </header>
    <article class="prose">${p.html}</article>
    <nav class="pager" aria-label="${esc(t.morePages)}">
      ${prev ? `<a class="prev" href="../${prev.slug}/"><span>${esc(t.prev)}</span>${esc(prev.title)}</a>` : "<span></span>"}
      ${next ? `<a class="next" href="../${next.slug}/"><span>${esc(t.next)}</span>${esc(next.title)}</a>` : "<span></span>"}
    </nav>
    <p class="edit"><a href="${source}" rel="noopener">${icon("Pencil", 15)}${esc(t.improve)}</a></p>
  </main>
  ${toc}
</div>
${footer(t, rel)}
</body>
</html>
`;
}

// ---------------------------------------------------------------- home page

function homePage(site) {
  const { t } = site;
  const h = t.home;
  const lang = t.code;
  const rel = t.dir ? "../" : "";
  const home = ""; // links on the home page are relative to it
  for (const [, , , link] of h.features) checkHome(site, link);
  for (const [, , link] of [...h.steps, ...h.faq]) checkHome(site, link);
  const img = (name) => {
    site.usedImages.add(name);
    const [w, ht] = webpSize(shotFile(lang, name));
    return { src: `${rel}${shotUrl(lang, name)}`, w, h: ht };
  };
  const dash = img("dashboard.webp");
  const tabs = h.showcase.map(([, name], i) => `<button type="button" role="tab" aria-selected="${i === 0}" data-show="${i}">${esc(name)}</button>`).join("");
  const panes = h.showcase.map(([file, name, text], i) => {
    const s = img(file);
    return `<figure class="pane" ${i ? "hidden" : ""} data-pane="${i}"><a href="${s.src}" data-lightbox><img src="${s.src}" alt="${esc(name)}" width="${s.w}" height="${s.h}" loading="lazy" decoding="async"></a><figcaption>${esc(text)}</figcaption></figure>`;
  }).join("");
  const exe = `${RELEASE}/HighBazar_${VERSION}_x64-setup.exe`;
  const dmg = `${RELEASE}/HighBazar_${VERSION}_universal.dmg`;
  const appimage = `${RELEASE}/HighBazar_${VERSION}_amd64.AppImage`;
  const deb = `<span data-href="${RELEASE}/HighBazar_${VERSION}_amd64.deb">.deb</span>`;
  const rpm = `<span data-href="${RELEASE}/HighBazar-${VERSION}-1.x86_64.rpm">.rpm</span>`;
  const os = [[brand.microsoft, exe], [brand.apple, dmg], [brand.linux, appimage]].map(([logo, href], i) => {
    const [name, text, button] = h.os[i];
    return `<a class="os-card reveal" href="${href}">${logo}<h3>${name}</h3><p>${text.replace("{deb}", deb).replace("{rpm}", rpm)}</p><span class="btn btn-primary">${icon("Download", 16)}${esc(button)}</span></a>`;
  }).join("\n      ");
  return `${head({ t, title: h.title, description: h.description, rel, slug: "" })}
<body class="home">
${topbar(t, rel, "")}
<main id="main">
  <section class="hero">
    <div class="hero-glow" aria-hidden="true"></div>
    <div class="hero-text reveal">
      <p class="pill">${icon("Sparkles", 16)}${esc(h.pill.replace("{v}", VERSION))} <a href="${home}changelog/">${esc(h.whatsNew)}</a></p>
      <h1>${h.h1}</h1>
      <p class="hero-lead">${esc(h.lead)}</p>
      <div class="cta">
        <a class="btn btn-primary btn-lg" href="${exe}" data-download data-mac="${esc(h.downloadMac)}" data-linux="${esc(h.downloadLinux)}">${brand.microsoft}<span>${esc(h.downloadWindows)}</span></a>
        <a class="btn btn-ghost btn-lg" href="${home}introduction/">${icon("Bookmark", 18)}${esc(h.readGuide)}</a>
      </div>
      <p class="hero-meta"><span>${brand.microsoft}Windows 10/11</span><span>${brand.apple}macOS 11+</span><span>${brand.linux}Linux</span><span>${icon("Heart", 15)}${h.free}</span></p>
    </div>
    <figure class="hero-shot reveal">
      <div class="window"><div class="window-bar"><i></i><i></i><i></i><span>${esc(h.windowTitle)}</span></div>
      <a href="${dash.src}" data-lightbox><img src="${dash.src}" alt="${esc(h.shotAlt)}" width="${dash.w}" height="${dash.h}" fetchpriority="high"></a></div>
    </figure>
  </section>

  <section class="band providers reveal" aria-label="${esc(h.providersLabel)}">
    <p>${esc(h.worksWith)}</p>
    <div class="marquee"><div>${[...PROVIDERS, ...PROVIDERS].map((p) => `<span>${esc(p)}</span>`).join("")}</div></div>
  </section>

  <section class="section">
    <div class="section-head reveal"><p class="eyebrow">${esc(h.featuresEyebrow)}</p><h2>${esc(h.featuresTitle)}</h2><p>${esc(h.featuresLead)}</p></div>
    <div class="features">${h.features.map(([ic, ti, d, link]) => `<a class="feature reveal" href="${home}${linkTo(link)}"><span class="feature-icon">${icon(ic, 26)}</span><h3>${esc(ti)}</h3><p>${esc(d)}</p></a>`).join("")}</div>
  </section>

  <section class="section showcase">
    <div class="section-head reveal"><p class="eyebrow">${esc(h.showEyebrow)}</p><h2>${esc(h.showTitle)}</h2><p>${esc(h.showLead)}</p></div>
    <div class="tabs reveal" role="tablist">${tabs}</div>
    <div class="panes reveal">${panes}</div>
  </section>

  <section class="section">
    <div class="section-head reveal"><p class="eyebrow">${esc(h.stepsEyebrow)}</p><h2>${esc(h.stepsTitle)}</h2></div>
    <ol class="steps">${h.steps.map(([ti, d, link], i) => `<li class="reveal"><span class="step-n">${i + 1}</span><h3>${esc(ti)}</h3><p>${esc(d)}</p><a href="${home}${linkTo(link)}">${esc(h.learnMore)}</a></li>`).join("")}</ol>
  </section>

  <section class="section download" id="download">
    <div class="section-head reveal"><p class="eyebrow">${esc(h.downloadEyebrow)}</p><h2>HighBazar ${VERSION}</h2><p>${h.downloadLead}</p></div>
    <div class="os-cards">
      ${os}
    </div>
    <p class="all-files reveal"><a href="${REPO}/releases/latest" rel="noopener">${esc(h.allFiles)}</a></p>
  </section>

  <section class="section">
    <div class="section-head reveal"><p class="eyebrow">${esc(h.faqEyebrow)}</p><h2>${esc(h.faqTitle)}</h2></div>
    <div class="faq-grid">${h.faq.map(([q, a, link]) => `<a class="faq-card reveal" href="${home}${linkTo(link)}"><h3>${esc(q)}</h3><p>${esc(a)}</p></a>`).join("")}</div>
  </section>

  <section class="final reveal">
    <h2>${esc(h.finalTitle)}</h2>
    <p>${esc(h.finalLead)}</p>
    <div class="cta"><a class="btn btn-primary btn-lg" href="#download">${icon("Download", 18)}${esc(h.finalDownload)}</a><a class="btn btn-ghost btn-lg" href="${home}setup-wizard/">${icon("Wand", 18)}${esc(h.finalSetup)}</a></div>
  </section>
</main>
${footer(t, rel)}
</body>
</html>
`;
}

function checkHome(site, link) {
  const [slug, anchor] = link.split("#");
  checkLink(site, "home", slug, anchor ?? "");
}

// One 404 page for both languages (GitHub Pages serves /404.html for every
// missing path); it follows the visitor's language.
function notFound() {
  const rel = "/HighBazar/";
  const [en, de] = [LANGS.en, LANGS.de];
  return `${head({ t: en, title: `${en.notFound.title} · ${en.docs}`, description: en.notFound.text, rel, slug: "" }).replace('<link rel="canonical"', '<meta name="robots" content="noindex">\n<link rel="canonical"')}
<body class="doc">
${topbar(en, rel, "404")}
<main id="main" class="not-found">
  <h1>404</h1>
  <p>${esc(en.notFound.text)}<br><span lang="de">${esc(de.notFound.text)}</span></p>
  <div class="cta"><a class="btn btn-primary" href="${rel}">${icon("ArrowLeft", 16)}${esc(en.notFound.home)}</a><a class="btn btn-ghost" href="${rel}de/" lang="de">${icon("ArrowLeft", 16)}${esc(de.notFound.home)}</a><button class="btn btn-ghost" type="button" data-search>${icon("Search", 16)}${esc(en.search)}</button></div>
</main>
${footer(en, rel)}
</body>
</html>
`;
}

// ---------------------------------------------------------------- write

const t0 = performance.now();
rmSync(dist, { recursive: true, force: true });
mkdirSync(join(dist, "images"), { recursive: true });

const css = readFileSync(join(here, "theme/site.css"), "utf8");
const js = readFileSync(join(here, "theme/site.js"), "utf8");
asset.css = `site.${hash(css)}.css`;
asset.js = `site.${hash(js)}.js`;
writeFileSync(join(dist, asset.css), css);
writeFileSync(join(dist, asset.js), js);
cpSync(join(root, "app-icon.svg"), join(dist, "favicon.svg"));
cpSync(join(root, "src-tauri/icons/128x128@2x.png"), join(dist, "apple-touch-icon.png"));

let entries = 0;
for (const [lang, site] of Object.entries(sites)) {
  const base = join(dist, site.t.dir);
  const search = [];
  for (const p of site.pages.values()) {
    mkdirSync(join(base, p.slug), { recursive: true });
    writeFileSync(join(base, p.slug, "index.html"), docPage(site, p));
    // One search entry per section: the text up to the next h2/h3.
    for (const part of p.html.split(/(?=<h[23] id=")/)) {
      const m = part.match(/^<h[23] id="([^"]+)">(.*?)<\/h[23]>/s);
      const text = strip(m ? part.slice(m[0].length) : part).slice(0, 1200);
      if (!text && !m) continue;
      search.push({ p: p.title, s: sectionOf(p.slug, site.t), h: m ? strip(m[2]).replace(/^#\s*/, "") : "", u: `${site.t.dir}${p.slug}/${m ? `#${m[1]}` : ""}`, t: text });
    }
  }
  writeFileSync(join(base, "index.html"), homePage(site));
  writeFileSync(join(base, "search.json"), JSON.stringify(search));
  entries += search.length;
  mkdirSync(join(dist, "images", lang === "en" ? "" : lang), { recursive: true });
  for (const img of site.usedImages) cpSync(shotFile(lang, img), join(dist, shotUrl(lang, img)));
  const unused = readdirSync(join(shotsDir, lang === "en" ? "" : lang)).filter((f) => f.endsWith(".webp") && !site.usedImages.has(f));
  const readme = readFileSync(join(root, lang === "en" ? "README.md" : `README.${lang}.md`), "utf8");
  const reallyUnused = unused.filter((f) => !readme.includes(f));
  if (reallyUnused.length) console.warn(`${lang}: unused screenshots: ${reallyUnused.join(", ")}`);
}
writeFileSync(join(dist, "404.html"), notFound());
const urls = Object.values(LANGS).flatMap((l) => ["", ...ORDER].map((s) => `<url><loc>${SITE}${l.dir}${s ? `${s}/` : ""}</loc>${Object.values(LANGS).map((a) => `<xhtml:link rel="alternate" hreflang="${a.code}" href="${SITE}${a.dir}${s ? `${s}/` : ""}"/>`).join("")}</url>`));
writeFileSync(join(dist, "sitemap.xml"), `<?xml version="1.0" encoding="UTF-8"?>\n<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9" xmlns:xhtml="http://www.w3.org/1999/xhtml">\n${urls.join("\n")}\n</urlset>\n`);
writeFileSync(join(dist, "robots.txt"), `User-agent: *\nAllow: /\nSitemap: ${SITE}sitemap.xml\n`);
writeFileSync(join(dist, ".nojekyll"), "");

let bytes = 0;
const walk = (d) => readdirSync(d).forEach((f) => (statSync(join(d, f)).isDirectory() ? walk(join(d, f)) : (bytes += statSync(join(d, f)).size)));
walk(dist);
const pageCount = Object.values(sites).reduce((n, s) => n + s.pages.size + 1, 0);
console.log(`${pageCount} pages in ${Object.keys(sites).length} languages, ${entries} search entries, ${(bytes / 1024).toFixed(0)} KB in ${(performance.now() - t0).toFixed(0)} ms → ${dist}`);

if (process.argv.includes("--serve")) {
  const types = { ".html": "text/html; charset=utf-8", ".css": "text/css", ".js": "text/javascript", ".json": "application/json", ".svg": "image/svg+xml", ".webp": "image/webp", ".png": "image/png", ".xml": "application/xml", ".txt": "text/plain" };
  createServer((req, res) => {
    const path = decodeURIComponent(new URL(req.url, "http://x").pathname);
    if (!path.startsWith("/HighBazar/")) return res.writeHead(302, { location: "/HighBazar/" }).end();
    let file = join(dist, path.slice("/HighBazar/".length));
    if (existsSync(file) && statSync(file).isDirectory()) file = join(file, "index.html");
    if (!existsSync(file)) return res.writeHead(404, { "content-type": types[".html"] }).end(readFileSync(join(dist, "404.html")));
    res.writeHead(200, { "content-type": types[extname(file)] ?? "application/octet-stream" }).end(readFileSync(file));
  }).listen(4173, () => console.log("preview: http://localhost:4173/HighBazar/"));
}
