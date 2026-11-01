// Builds the documentation website from docs/site/pages/*.md into
// docs/site/dist, which GitHub Pages serves at https://zcarecrow.github.io/HighBazar/.
//
//   node docs/site/build.mjs           build (fails on broken links or anchors)
//   node docs/site/build.mjs --serve   build and preview on http://localhost:4173/HighBazar/
//
// Screenshots come from docs/screenshots (node docs/site/shots.mjs), icons from
// the app's own Nucleo set (src/icons/nucleo.ts) plus four brand icons in
// docs/site/icons. Nucleo icons are licensed to the HighBazar author and may
// not be extracted or reused (see README).
import { Marked } from "marked";
import { createHash } from "node:crypto";
import { cpSync, existsSync, mkdirSync, readdirSync, readFileSync, rmSync, statSync, writeFileSync } from "node:fs";
import { createServer } from "node:http";
import { dirname, extname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { nucleo } from "../../src/icons/nucleo.ts";

const here = dirname(fileURLToPath(import.meta.url));
const root = join(here, "../..");
const dist = join(here, "dist");
const shots = join(root, "docs/screenshots");
const pkg = JSON.parse(readFileSync(join(root, "package.json"), "utf8"));
const VERSION = pkg.version;
const SITE = "https://zcarecrow.github.io/HighBazar/";
const REPO = "https://github.com/ZcarecroW/HighBazar";
const RELEASE = `${REPO}/releases/download/v${VERSION}`;

// Sidebar: sections and the order of their pages (file names without .md).
const NAV = [
  ["Getting started", ["introduction", "installation", "setup-wizard", "first-stream"]],
  ["Using HighBazar", ["dashboard", "chat-input", "channels", "companion", "moderation", "translation", "stream-manager", "overlay", "settings"]],
  ["AI", ["ai-providers", "web-search", "costs"]],
  ["Reference", ["twitch-glossary", "shortcuts", "privacy", "troubleshooting", "faq", "changelog", "building"]],
];

// ---------------------------------------------------------------- helpers

const esc = (s) => String(s).replace(/[&<>"']/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" })[c]);
const strip = (html) => html.replace(/<[^>]+>/g, " ").replace(/&nbsp;/g, " ").replace(/&amp;/g, "&").replace(/&lt;/g, "<").replace(/&gt;/g, ">").replace(/&quot;/g, '"').replace(/&#39;/g, "'").replace(/\s+/g, " ").trim();
const slugify = (s) => strip(s).toLowerCase().normalize("NFKD").replace(/[̀-ͯ]/g, "").replace(/[^a-z0-9]+/g, "-").replace(/^-|-$/g, "");
const hash = (s) => createHash("sha256").update(s).digest("hex").slice(0, 10);

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
const CALLOUTS = { NOTE: ["Note", "Info"], TIP: ["Tip", "Sparkles"], IMPORTANT: ["Important", "Bookmark"], WARNING: ["Warning", "TriangleAlert"], CAUTION: ["Caution", "ShieldAlert"] };
function callouts(md) {
  return md.replace(/^> \[!(NOTE|TIP|IMPORTANT|WARNING|CAUTION)\][^\n]*\n((?:>[^\n]*\n?)*)/gm, (_, kind, body) => {
    const [label, ic] = CALLOUTS[kind];
    const inner = body.replace(/^> ?/gm, "");
    return `<div class="callout ${kind.toLowerCase()}"><p class="callout-title">${icon(ic, 18)}${label}</p>\n\n${inner}\n</div>\n\n`;
  });
}

// ---------------------------------------------------------------- pages

const pages = new Map(); // slug -> page
for (const f of readdirSync(join(here, "pages")).filter((f) => f.endsWith(".md"))) {
  const slug = f.replace(/\.md$/, "");
  const { meta, body } = frontMatter(readFileSync(join(here, "pages", f), "utf8"), f);
  pages.set(slug, { slug, ...meta, md: body });
}
// The changelog page is the repository's CHANGELOG.md without its title.
pages.set("changelog", {
  slug: "changelog",
  title: "Changelog",
  description: "Every HighBazar release and what changed in it.",
  icon: "Clock",
  md: readFileSync(join(root, "CHANGELOG.md"), "utf8").replace(/^# Changelog\s*/, ""),
});

const order = NAV.flatMap(([, slugs]) => slugs);
for (const s of order) if (!pages.has(s)) throw new Error(`NAV lists "${s}" but docs/site/pages/${s}.md is missing`);
for (const s of pages.keys()) if (!order.includes(s)) throw new Error(`docs/site/pages/${s}.md is not in NAV`);
const sectionOf = (slug) => NAV.find(([, s]) => s.includes(slug))[0];

const usedImages = new Set();
const links = []; // [fromSlug, targetSlug, anchor]

function render(page) {
  const toc = [];
  const ids = new Set();
  const marked = new Marked({ gfm: true });
  marked.use({
    renderer: {
      heading({ tokens, depth, text }) {
        const html = this.parser.parseInline(tokens);
        let id = slugify(text.replace(/\{#([\w-]+)\}\s*$/, "")) || "section";
        const custom = text.match(/\{#([\w-]+)\}\s*$/);
        if (custom) id = custom[1];
        let unique = id;
        for (let n = 2; ids.has(unique); n++) unique = `${id}-${n}`;
        ids.add(unique);
        const label = html.replace(/\s*\{#[\w-]+\}\s*$/, "");
        if (depth === 2 || depth === 3) toc.push({ depth, id: unique, text: strip(label) });
        return `<h${depth} id="${unique}"><a class="anchor" href="#${unique}" aria-label="Link to this section">#</a>${label}</h${depth}>\n`;
      },
      image({ href, title, text }) {
        if (/^https?:/.test(href)) return `<img src="${esc(href)}" alt="${esc(text)}" loading="lazy">`;
        const file = join(shots, href);
        if (!existsSync(file)) throw new Error(`${page.slug}: missing screenshot ${href} (run node docs/site/shots.mjs)`);
        usedImages.add(href);
        const [w, h] = webpSize(file);
        const small = w < 1100 ? " small" : "";
        return `<figure class="shot${small}"><a href="../images/${esc(href)}" data-lightbox><img src="../images/${esc(href)}" alt="${esc(text)}" width="${w}" height="${h}" loading="lazy" decoding="async"></a>${title || text ? `<figcaption>${esc(title || text)}</figcaption>` : ""}</figure>`;
      },
      link({ href, title, tokens }) {
        const inner = this.parser.parseInline(tokens);
        if (/^(https?:|mailto:)/.test(href)) return `<a href="${esc(href)}"${title ? ` title="${esc(title)}"` : ""} rel="noopener">${inner}</a>`;
        const m = href.match(/^([\w-]*)(?:\.md)?(?:#([\w-]+))?$/);
        if (!m) throw new Error(`${page.slug}: unsupported link "${href}"`);
        const target = m[1] || page.slug;
        links.push([page.slug, target, m[2] ?? ""]);
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
  const md = callouts(page.md)
    .replace(/\{icon:(\w+)\}/g, (_, n) => icon(n, 18, "ico inline"))
    .split("\n")
    .map((l) => (/^#{1,6} /.test(l) ? l : l.replace(/\s*\{#([\w-]+)\}/g, (_, id) => (ids.add(id), `<span id="${id}" class="target"></span>`))))
    .join("\n");
  page.html = marked.parse(md);
  page.toc = toc;
  page.ids = ids;
}
for (const p of pages.values()) render(p);

for (const [from, to, anchor] of links) {
  const t = to === "index" ? null : pages.get(to);
  if (to !== "index" && !t) throw new Error(`${from}: link to missing page "${to}"`);
  if (anchor && t && !t.ids.has(anchor)) throw new Error(`${from}: link to missing anchor "${to}#${anchor}"`);
}

function checkHomeLink(link) {
  const [slug, anchor] = link.split("#");
  const p = pages.get(slug);
  if (!p) throw new Error(`home: link to missing page "${slug}"`);
  if (anchor && !p.ids.has(anchor)) throw new Error(`home: link to missing anchor "${link}"`);
}

// ---------------------------------------------------------------- layout

const asset = {}; // name -> url with content hash
function head({ title, description, rel, canonical, image = "dashboard.webp" }) {
  return `<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>${esc(title)}</title>
<meta name="description" content="${esc(description)}">
<link rel="canonical" href="${canonical}">
<meta name="theme-color" content="#14101c">
<meta property="og:type" content="website">
<meta property="og:site_name" content="HighBazar">
<meta property="og:title" content="${esc(title)}">
<meta property="og:description" content="${esc(description)}">
<meta property="og:url" content="${canonical}">
<meta property="og:image" content="${SITE}images/${image}">
<meta name="twitter:card" content="summary_large_image">
<link rel="icon" href="${rel}favicon.svg" type="image/svg+xml">
<link rel="apple-touch-icon" href="${rel}apple-touch-icon.png">
<script>document.documentElement.classList.add("js");try{var t=localStorage.getItem("hb-docs-theme");if(t)document.documentElement.dataset.theme=t}catch(e){}</script>
<link rel="stylesheet" href="${rel}${asset.css}">
<script src="${rel}${asset.js}" defer></script>
</head>`;
}

function topbar(rel, current) {
  return `<a class="skip" href="#main">Skip to content</a>
<header class="topbar">
  <button class="icon-btn menu-btn" type="button" aria-label="Open the menu" aria-expanded="false" data-menu>${icon("Layers", 20)}</button>
  <a class="brand" href="${rel || "./"}"><img src="${rel}favicon.svg" alt="" width="30" height="30"><span>HighBazar</span><span class="brand-tag">Docs</span></a>
  <button class="search-btn" type="button" data-search>${icon("Search", 16)}<span>Search the docs</span><kbd>Ctrl K</kbd></button>
  <nav class="top-links" aria-label="Site">
    <a href="${rel}introduction/"${current && current !== "home" ? ' aria-current="true"' : ""}>Guide</a>
    <a href="${rel}changelog/" class="hide-sm">v${VERSION}</a>
    <a class="icon-btn" href="${REPO}" aria-label="HighBazar on GitHub" rel="noopener">${brand["logo-github"]}</a>
    <button class="icon-btn" type="button" data-theme-toggle aria-label="Switch between dark and light">${icon("Moon", 18, "ico when-light")}${icon("Sun", 18, "ico when-dark")}</button>
    <a class="btn btn-primary btn-sm hide-xs" href="${rel}installation/">${icon("Download", 16)}Download</a>
  </nav>
</header>`;
}

function footer(rel) {
  return `<footer class="site-footer">
  <div class="footer-inner">
    <div class="footer-brand"><img src="${rel}favicon.svg" alt="" width="36" height="36"><div><strong>HighBazar</strong><p>An AI companion and fair moderator for your Twitch chat. Free and open source (MIT).</p></div></div>
    <nav aria-label="Footer">
      <div><p>Docs</p><a href="${rel}introduction/">Introduction</a><a href="${rel}installation/">Installation</a><a href="${rel}first-stream/">First stream</a><a href="${rel}faq/">FAQ</a></div>
      <div><p>Project</p><a href="${REPO}" rel="noopener">GitHub</a><a href="${REPO}/releases/latest" rel="noopener">Releases</a><a href="${REPO}/issues" rel="noopener">Report a problem</a><a href="${rel}changelog/">Changelog</a></div>
    </nav>
  </div>
  <p class="fine">HighBazar is not affiliated with Twitch, Anthropic, OpenAI, Google or any other provider. Icons: <a href="https://nucleoapp.com" rel="noopener">Nucleo</a>, licensed to the author for HighBazar only.</p>
</footer>
<dialog class="search" aria-label="Search the documentation">
  <div class="search-box">${icon("Search", 18)}<input type="search" placeholder="Search settings, features, Twitch terms …" autocomplete="off" spellcheck="false" aria-label="Search"><kbd>Esc</kbd></div>
  <ul class="search-results" role="listbox"></ul>
  <p class="search-hint"><span><kbd>↑</kbd><kbd>↓</kbd> choose</span><span><kbd>Enter</kbd> open</span></p>
</dialog>
<div class="lightbox" hidden><img alt=""><button class="icon-btn" type="button" aria-label="Close">${icon("X", 22)}</button></div>`;
}

function sidebar(current, rel) {
  return NAV.map(([section, slugs]) => `<div class="nav-group"><p>${section}</p>${slugs.map((s) => {
    const p = pages.get(s);
    return `<a href="${rel}${s}/"${s === current ? ' aria-current="page"' : ""}>${icon(p.icon, 18)}<span>${esc(p.navTitle ?? p.title)}</span></a>`;
  }).join("")}</div>`).join("");
}

function docPage(p) {
  const rel = "../";
  const i = order.indexOf(p.slug);
  const prev = pages.get(order[i - 1]);
  const next = pages.get(order[i + 1]);
  const toc = p.toc.length > 1 ? `<aside class="toc" aria-label="On this page"><p>On this page</p><ul>${p.toc.map((h) => `<li class="d${h.depth}"><a href="#${h.id}">${esc(h.text)}</a></li>`).join("")}</ul></aside>` : `<aside class="toc"></aside>`;
  const source = p.slug === "changelog" ? `${REPO}/blob/main/CHANGELOG.md` : `${REPO}/edit/main/docs/site/pages/${p.slug}.md`;
  return `${head({ title: `${p.title} · HighBazar docs`, description: p.description, rel, canonical: `${SITE}${p.slug}/` })}
<body class="doc">
${topbar(rel, p.slug)}
<div class="layout">
  <aside class="sidebar" aria-label="Documentation"><nav>${sidebar(p.slug, rel)}</nav></aside>
  <main id="main" class="content">
    <header class="page-head">
      <p class="eyebrow">${esc(sectionOf(p.slug))}</p>
      <h1><span class="page-icon">${icon(p.icon, 26)}</span>${esc(p.title)}</h1>
      <p class="lead">${esc(p.description)}</p>
    </header>
    <article class="prose">${p.html}</article>
    <nav class="pager" aria-label="More pages">
      ${prev ? `<a class="prev" href="${rel}${prev.slug}/"><span>Previous</span>${esc(prev.title)}</a>` : "<span></span>"}
      ${next ? `<a class="next" href="${rel}${next.slug}/"><span>Next</span>${esc(next.title)}</a>` : "<span></span>"}
    </nav>
    <p class="edit"><a href="${source}" rel="noopener">${icon("Pencil", 15)}Improve this page on GitHub</a></p>
  </main>
  ${toc}
</div>
${footer(rel)}
</body>
</html>
`;
}

// ---------------------------------------------------------------- home page

const FEATURES = [
  ["Bot", "Chats like a regular", "Every few minutes something fun, interesting or engaging that fits your chat and your game – adaptive, so it holds back when chat is busy.", "companion"],
  ["Heart", "Welcomes and celebrates", "Answers when addressed, greets first-time chatters, thanks follows, subs, gifts, raids and cheers in your stream's language.", "companion#reactions"],
  ["Brain", "Knows your game", "Researches every new category with Wikipedia and a web search: facts, tips and the latest patch – and looks up current answers when chat asks.", "web-search"],
  ["ShieldCheck", "Moderates fairly", "Instant local rules plus AI that tells banter from real hate, with escalation, suggestions you approve, a full log and undo.", "moderation"],
  ["Languages", "Translates your chat", "Messages in languages you don't speak are translated right below them; rule breaks get a note with the real meaning for your mods.", "translation"],
  ["MonitorPlay", "Stream manager", "Title, category, chat modes, raids, shoutouts, clips, polls, predictions, channel points, AutoMod and ads – without opening Twitch.", "stream-manager"],
  ["Gamepad2", "Made for gaming PCs", "Rust core, about 9 MB in the background, lower process priority – and a chat overlay for the Steam overlay or OBS.", "overlay"],
  ["Sparkles", "Any AI you like", "Claude, GPT, Gemini, Grok, OpenRouter, Perplexity, DeepSeek, local models with Ollama or LM Studio – or your Claude / ChatGPT subscription.", "ai-providers"],
];
const SHOWCASE = [
  ["dashboard.webp", "Dashboard", "Live feed with emotes, bot messages, events and moderation – plus stats, quick controls and a chat simulator."],
  ["companion.webp", "Companion", "Personality, timing, topics, reactions, commands and game research – seven built-in styles to start from."],
  ["moderation.webp", "Moderation", "Modes, AI check, categories with actions for clear and borderline cases, repeat offenders and instant rules."],
  ["ai.webp", "AI", "Profiles for every provider, model list from the provider, prices, web search and a one-click test."],
  ["stream-expert.webp", "Stream", "Simple, Advanced or Expert: from title and tags up to polls, rewards, AutoMod and ad automation."],
];
const STEPS = [
  ["Download", "Install HighBazar on Windows, macOS or Linux. It's free and updates itself.", "installation"],
  ["Connect", "Log in to Twitch with a short code, pick an AI and paste a key – the setup wizard guides you.", "setup-wizard"],
  ["Try it safely", "Test mode and the chat simulator show how the bot reacts – nothing is sent to Twitch.", "first-stream"],
  ["Go live", "Start the bot. It chats, welcomes, researches and moderates while you play.", "dashboard"],
];
const PROVIDERS = ["Claude", "OpenAI GPT", "Gemini", "Grok", "OpenRouter", "Perplexity", "DeepSeek", "Mistral", "Groq", "Together AI", "Ollama", "LM Studio", "Claude Code CLI", "Codex CLI"];
const FAQ_TEASER = [
  ["Does the bot pretend to be me?", "No. Messages from your account carry a prefix like [HighBazar], and the AI is told never to speak in your name. You can also connect a separate bot account.", "faq#does-the-bot-pretend-to-be-me"],
  ["What does it cost?", "HighBazar is free. AI providers bill you directly – with a small model a stream hour costs a few cents, local models nothing. A daily budget caps it.", "costs"],
  ["Will it spam my chat?", "No: timed messages are spread out, pause when chat is busy or nobody answers, and only run while you're live. Replies have cooldowns.", "faq#will-it-spam-my-chat"],
  ["Is my data safe?", "Everything stays on your PC. Keys and Twitch logins live in your system's credential store; chat only goes to the AI you chose.", "privacy"],
];

function homePage() {
  const rel = "";
  for (const [, , , link] of FEATURES) checkHomeLink(link);
  for (const [, , link] of [...STEPS, ...FAQ_TEASER]) checkHomeLink(link);
  const [dw, dh] = webpSize(join(shots, "dashboard.webp"));
  usedImages.add("dashboard.webp");
  const show = SHOWCASE.map(([img, name], i) => `<button type="button" role="tab" aria-selected="${i === 0}" data-show="${i}">${esc(name)}</button>`).join("");
  const panes = SHOWCASE.map(([img, name, text], i) => {
    usedImages.add(img);
    const [w, h] = webpSize(join(shots, img));
    return `<figure class="pane" ${i ? "hidden" : ""} data-pane="${i}"><a href="images/${img}" data-lightbox><img src="images/${img}" alt="${esc(name)}" width="${w}" height="${h}" loading="lazy" decoding="async"></a><figcaption>${esc(text)}</figcaption></figure>`;
  }).join("");
  return `${head({ title: "HighBazar – AI companion and fair moderator for your Twitch chat", description: "HighBazar chats like a friendly regular, welcomes newcomers, knows your game, translates and moderates fairly – with the AI of your choice. Free for Windows, macOS and Linux.", rel, canonical: SITE })}
<body class="home">
${topbar(rel, "home")}
<main id="main">
  <section class="hero">
    <div class="hero-glow" aria-hidden="true"></div>
    <div class="hero-text reveal">
      <p class="pill">${icon("Sparkles", 16)}New in ${VERSION}: web search that actually gets used <a href="changelog/">What's new →</a></p>
      <h1>Your Twitch chat,<br><span class="grad">never quiet again.</span></h1>
      <p class="hero-lead">HighBazar is a free desktop app that brings life into your chat: it chats like a friendly regular, welcomes newcomers, knows your game – and keeps chat safe with fair, context-aware moderation.</p>
      <div class="cta">
        <a class="btn btn-primary btn-lg" href="${RELEASE}/HighBazar_${VERSION}_x64-setup.exe" data-download>${brand.microsoft}<span>Download for Windows</span></a>
        <a class="btn btn-ghost btn-lg" href="introduction/">${icon("Bookmark", 18)}Read the guide</a>
      </div>
      <p class="hero-meta"><span>${brand.microsoft}Windows 10/11</span><span>${brand.apple}macOS 11+</span><span>${brand.linux}Linux</span><span>${icon("Heart", 15)}Free &amp; open source</span></p>
    </div>
    <figure class="hero-shot reveal">
      <div class="window"><div class="window-bar"><i></i><i></i><i></i><span>HighBazar – Dashboard</span></div>
      <a href="images/dashboard.webp" data-lightbox><img src="images/dashboard.webp" alt="The HighBazar dashboard with the live feed, stats and the chat simulator" width="${dw}" height="${dh}" fetchpriority="high"></a></div>
    </figure>
  </section>

  <section class="band providers reveal" aria-label="Supported AI providers">
    <p>Works with</p>
    <div class="marquee"><div>${[...PROVIDERS, ...PROVIDERS].map((p) => `<span>${esc(p)}</span>`).join("")}</div></div>
  </section>

  <section class="section">
    <div class="section-head reveal"><p class="eyebrow">Features</p><h2>Everything a lively, safe chat needs</h2><p>Built for small streamers who want a lively chat from the first viewer on – and it scales to big channels with adaptive pacing, batching and budgets.</p></div>
    <div class="features">${FEATURES.map(([ic, t, d, link]) => `<a class="feature reveal" href="${link.replace(/^([\w-]+)/, "$1/")}"><span class="feature-icon">${icon(ic, 26)}</span><h3>${esc(t)}</h3><p>${esc(d)}</p></a>`).join("")}</div>
  </section>

  <section class="section showcase">
    <div class="section-head reveal"><p class="eyebrow">A look inside</p><h2>Calm, clear, explained everywhere</h2><p>Ten color themes in light and dark, a small ⓘ next to every setting that explains it in plain words – including the Twitch terms newcomers stumble over.</p></div>
    <div class="tabs reveal" role="tablist">${show}</div>
    <div class="panes reveal">${panes}</div>
  </section>

  <section class="section">
    <div class="section-head reveal"><p class="eyebrow">Get started</p><h2>Live in about ten minutes</h2></div>
    <ol class="steps">${STEPS.map(([t, d, link], i) => `<li class="reveal"><span class="step-n">${i + 1}</span><h3>${esc(t)}</h3><p>${esc(d)}</p><a href="${link}/">Learn more →</a></li>`).join("")}</ol>
  </section>

  <section class="section download" id="download">
    <div class="section-head reveal"><p class="eyebrow">Download</p><h2>HighBazar ${VERSION}</h2><p>Installs in seconds and updates itself. Not signed with a paid certificate yet – the <a href="installation/">installation guide</a> shows how to open it the first time.</p></div>
    <div class="os-cards">
      <a class="os-card reveal" href="${RELEASE}/HighBazar_${VERSION}_x64-setup.exe">${brand.microsoft}<h3>Windows</h3><p>Windows 10 and 11 · 64-bit<br>No admin rights needed</p><span class="btn btn-primary">${icon("Download", 16)}Installer (.exe)</span></a>
      <a class="os-card reveal" href="${RELEASE}/HighBazar_${VERSION}_universal.dmg">${brand.apple}<h3>macOS</h3><p>macOS 11 or newer<br>Apple Silicon and Intel</p><span class="btn btn-primary">${icon("Download", 16)}Disk image (.dmg)</span></a>
      <a class="os-card reveal" href="${RELEASE}/HighBazar_${VERSION}_amd64.AppImage">${brand.linux}<h3>Linux</h3><p>AppImage with auto-updates<br><span class="os-alt">also <span data-href="${RELEASE}/HighBazar_${VERSION}_amd64.deb">.deb</span> and <span data-href="${RELEASE}/HighBazar-${VERSION}-1.x86_64.rpm">.rpm</span></span></p><span class="btn btn-primary">${icon("Download", 16)}AppImage</span></a>
    </div>
    <p class="all-files reveal"><a href="${REPO}/releases/latest" rel="noopener">All files and release notes on GitHub →</a></p>
  </section>

  <section class="section">
    <div class="section-head reveal"><p class="eyebrow">Questions</p><h2>Good to know</h2></div>
    <div class="faq-grid">${FAQ_TEASER.map(([q, a, link]) => `<a class="faq-card reveal" href="${link.replace(/^([\w-]+)/, "$1/")}"><h3>${esc(q)}</h3><p>${esc(a)}</p></a>`).join("")}</div>
  </section>

  <section class="final reveal">
    <h2>Ready to give your chat a friend?</h2>
    <p>Free, open source, and set up in minutes.</p>
    <div class="cta"><a class="btn btn-primary btn-lg" href="#download">${icon("Download", 18)}Download HighBazar</a><a class="btn btn-ghost btn-lg" href="setup-wizard/">${icon("Wand", 18)}See the setup</a></div>
  </section>
</main>
${footer(rel)}
</body>
</html>
`;
}

function notFound() {
  const rel = "/HighBazar/";
  return `${head({ title: "Page not found · HighBazar docs", description: "This page doesn't exist.", rel, canonical: `${SITE}404.html` })}
<body class="doc">
${topbar(rel, "404")}
<main id="main" class="not-found"><h1>404</h1><p>This page doesn't exist – maybe it moved.</p><div class="cta"><a class="btn btn-primary" href="${rel}">${icon("ArrowLeft", 16)}Home</a><button class="btn btn-ghost" type="button" data-search>${icon("Search", 16)}Search the docs</button></div></main>
${footer(rel)}
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

const search = [];
for (const p of pages.values()) {
  mkdirSync(join(dist, p.slug), { recursive: true });
  writeFileSync(join(dist, p.slug, "index.html"), docPage(p));
  // One search entry per section: the text up to the next h2/h3.
  const parts = p.html.split(/(?=<h[23] id=")/);
  for (const part of parts) {
    const m = part.match(/^<h[23] id="([^"]+)">(.*?)<\/h[23]>/s);
    const text = strip(m ? part.slice(m[0].length) : part).slice(0, 1200);
    if (!text && !m) continue;
    search.push({ p: p.title, s: sectionOf(p.slug), h: m ? strip(m[2]).replace(/^#\s*/, "") : "", u: `${p.slug}/${m ? `#${m[1]}` : ""}`, t: text });
  }
}
writeFileSync(join(dist, "index.html"), homePage());
writeFileSync(join(dist, "404.html"), notFound());
writeFileSync(join(dist, "search.json"), JSON.stringify(search));
for (const img of usedImages) cpSync(join(shots, img), join(dist, "images", img));
writeFileSync(join(dist, "sitemap.xml"), `<?xml version="1.0" encoding="UTF-8"?>\n<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">\n<url><loc>${SITE}</loc></url>\n${order.map((s) => `<url><loc>${SITE}${s}/</loc></url>`).join("\n")}\n</urlset>\n`);
writeFileSync(join(dist, "robots.txt"), `User-agent: *\nAllow: /\nSitemap: ${SITE}sitemap.xml\n`);
writeFileSync(join(dist, ".nojekyll"), "");

const unused = readdirSync(shots).filter((f) => f.endsWith(".webp") && !usedImages.has(f) && !readFileSync(join(root, "README.md"), "utf8").includes(f));
if (unused.length) console.warn(`unused screenshots: ${unused.join(", ")}`);
let bytes = 0;
const walk = (d) => readdirSync(d).forEach((f) => (statSync(join(d, f)).isDirectory() ? walk(join(d, f)) : (bytes += statSync(join(d, f)).size)));
walk(dist);
console.log(`${pages.size + 1} pages, ${search.length} search entries, ${usedImages.size} images, ${(bytes / 1024).toFixed(0)} KB in ${(performance.now() - t0).toFixed(0)} ms → ${dist}`);

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
