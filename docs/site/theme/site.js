// HighBazar documentation: theme toggle, search (Ctrl+K), mobile menu, table
// of contents highlighting, lightbox, copy buttons, home page tabs and the
// download button for the visitor's system. No dependencies.
(() => {
  const $ = (s, el = document) => el.querySelector(s);
  const $$ = (s, el = document) => [...el.querySelectorAll(s)];
  const root = document.documentElement;
  const base = new URL(document.querySelector('link[rel="icon"]').getAttribute("href"), location.href).href.replace(/favicon\.svg$/, "");
  const de = root.lang === "de";
  const T = de
    ? { loading: "Lädt …", failed: "Der Suchindex konnte nicht geladen werden.", start: "Tippe, um alle Seiten zu durchsuchen – z. B. „Raid“, „Websuche“, „Budget“.", none: (q) => `Nichts gefunden für „${q}“.`, copy: "Kopieren" }
    : { loading: "Loading …", failed: "The search index could not be loaded.", start: "Type to search all pages – e.g. “raid”, “web search”, “budget”.", none: (q) => `Nothing found for “${q}”.`, copy: "Copy" };

  // ------------------------------------------------------------ language
  // A visitor who picks a language keeps it; one whose browser prefers the
  // other language gets a small hint (once, until they choose or close it).
  let chosen = null;
  try { chosen = localStorage.getItem("hb-docs-lang"); } catch {}
  const remember = (v) => { try { localStorage.setItem("hb-docs-lang", v); } catch {} };
  $$("[data-lang]").forEach((a) => a.addEventListener("click", () => remember(a.dataset.lang)));
  const hint = $(".lang-hint");
  if (hint && !chosen && (navigator.languages ?? [navigator.language]).some((l) => l?.toLowerCase().startsWith(hint.dataset.hintFor)) && !(navigator.language ?? "").toLowerCase().startsWith(root.lang)) {
    hint.hidden = false;
    $("[data-hint-close]", hint)?.addEventListener("click", () => { hint.hidden = true; remember(root.lang); });
  }

  // ------------------------------------------------------------ theme
  const dark = () => (root.dataset.theme ?? (matchMedia("(prefers-color-scheme: light)").matches ? "light" : "dark")) === "dark";
  $$("[data-theme-toggle]").forEach((b) =>
    b.addEventListener("click", () => {
      root.dataset.theme = dark() ? "light" : "dark";
      try { localStorage.setItem("hb-docs-theme", root.dataset.theme); } catch {}
    }),
  );

  // ------------------------------------------------------------ mobile menu
  const menu = $("[data-menu]");
  menu?.addEventListener("click", () => {
    const open = document.body.classList.toggle("nav-open");
    menu.setAttribute("aria-expanded", String(open));
  });
  document.addEventListener("click", (e) => {
    if (document.body.classList.contains("nav-open") && !e.target.closest(".sidebar, [data-menu]")) {
      document.body.classList.remove("nav-open");
      menu?.setAttribute("aria-expanded", "false");
    }
  });
  $(".sidebar [aria-current]")?.scrollIntoView({ block: "center" });

  // ------------------------------------------------------------ search
  const dlg = $("dialog.search");
  const input = $("input", dlg);
  const list = $(".search-results", dlg);
  let index = null;
  let hits = [];
  let sel = 0;
  const fold = (s) => s.toLowerCase().normalize("NFD").replace(/[̀-ͯ]/g, "");
  const escHtml = (s) => s.replace(/[&<>"]/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;" })[c]);

  async function openSearch() {
    if (!dlg.open) dlg.showModal();
    input.select();
    if (!index) {
      list.innerHTML = `<li class="empty">${T.loading}</li>`;
      try {
        index = (await (await fetch(base + (de ? "de/" : "") + "search.json")).json()).map((e) => ({ ...e, ft: fold(e.p), fh: fold(e.h), fx: fold(e.t) }));
      } catch {
        list.innerHTML = `<li class="empty">${T.failed}</li>`;
        return;
      }
    }
    run();
  }

  function score(e, words) {
    let total = 0;
    for (const w of words) {
      let s = 0;
      const wordStart = (t) => t.startsWith(w) || t.includes(" " + w) || t.includes("-" + w);
      if (e.fh.includes(w)) s += wordStart(e.fh) ? 12 : 7;
      if (e.ft.includes(w)) s += wordStart(e.ft) ? 8 : 4;
      if (e.fx.includes(w)) {
        // Sections that talk about the term more often rank higher.
        const count = e.fx.split(w).length - 1;
        s += (wordStart(e.fx) ? 3 : 1.5) + Math.min(count - 1, 5) * 0.8;
      }
      if (!s) return 0;
      total += s;
    }
    return total + (e.h ? 0 : 0.5);
  }

  function snippet(text, words) {
    const f = fold(text);
    let at = -1;
    for (const w of words) {
      at = f.indexOf(w);
      if (at >= 0) break;
    }
    const start = Math.max(0, at - 60);
    let s = (start ? "… " : "") + text.slice(start, start + 180) + (text.length > start + 180 ? " …" : "");
    s = escHtml(s);
    for (const w of words) {
      if (w.length < 2) continue;
      s = s.replace(new RegExp(`(${w.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")})`, "ig"), "<mark>$1</mark>");
    }
    return s;
  }

  function run() {
    const q = fold(input.value.trim());
    const words = q.split(/\s+/).filter(Boolean);
    if (!words.length) {
      list.innerHTML = `<li class="empty">${T.start}</li>`;
      hits = [];
      return;
    }
    hits = index.map((e) => [score(e, words), e]).filter(([s]) => s > 0).sort((a, b) => b[0] - a[0]).slice(0, 14).map(([, e]) => e);
    sel = 0;
    list.innerHTML = hits.length
      ? hits.map((e, i) => `<li><a href="${base}${e.u}" role="option" aria-selected="${i === 0}"><span class="r-title">${escHtml(e.h || e.p)}${e.h ? ` <small>· ${escHtml(e.p)}</small>` : ` <small>· ${escHtml(e.s)}</small>`}</span><span class="r-text">${snippet(e.t, words)}</span></a></li>`).join("")
      : `<li class="empty">${T.none(escHtml(input.value.trim()))}</li>`;
  }

  function move(d) {
    const items = $$("a", list);
    if (!items.length) return;
    items[sel]?.setAttribute("aria-selected", "false");
    sel = (sel + d + items.length) % items.length;
    items[sel].setAttribute("aria-selected", "true");
    items[sel].scrollIntoView({ block: "nearest" });
  }

  input?.addEventListener("input", () => index && run());
  input?.addEventListener("keydown", (e) => {
    // A search field would only clear itself on the first Escape.
    if (e.key === "Escape") (e.preventDefault(), dlg.close());
    else if (e.key === "ArrowDown") (e.preventDefault(), move(1));
    else if (e.key === "ArrowUp") (e.preventDefault(), move(-1));
    else if (e.key === "Enter") {
      const a = $$("a", list)[sel];
      if (a) (e.preventDefault(), (location.href = a.href), dlg.close());
    }
  });
  list?.addEventListener("click", (e) => e.target.closest("a") && dlg.close());
  dlg?.addEventListener("click", (e) => e.target === dlg && dlg.close());
  $$("[data-search]").forEach((b) => b.addEventListener("click", openSearch));
  document.addEventListener("keydown", (e) => {
    const typing = /^(INPUT|TEXTAREA|SELECT)$/.test(document.activeElement?.tagName ?? "");
    if ((e.key === "k" && (e.ctrlKey || e.metaKey)) || (e.key === "/" && !typing)) {
      e.preventDefault();
      openSearch();
    }
  });
  if (/Mac|iPhone|iPad/.test(navigator.platform)) $$(".search-btn kbd").forEach((k) => (k.textContent = "⌘ K"));

  // ------------------------------------------------------------ table of contents
  const tocLinks = $$(".toc a");
  if (tocLinks.length) {
    const byId = new Map(tocLinks.map((a) => [decodeURIComponent(a.hash.slice(1)), a]));
    const heads = [...byId.keys()].map((id) => document.getElementById(id)).filter(Boolean);
    const mark = () => {
      let current = heads[0];
      for (const h of heads) if (h.getBoundingClientRect().top < 120) current = h;
      tocLinks.forEach((a) => a.classList.toggle("active", byId.get(current?.id) === a));
    };
    let ticking = false;
    addEventListener("scroll", () => { if (!ticking) { ticking = true; requestAnimationFrame(() => { mark(); ticking = false; }); } }, { passive: true });
    mark();
  }

  // ------------------------------------------------------------ copy buttons
  $$(".prose pre").forEach((pre) => {
    const b = document.createElement("button");
    b.className = "copy";
    b.type = "button";
    b.setAttribute("aria-label", T.copy);
    b.innerHTML = '<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true"><rect x="8" y="8" width="13" height="13" rx="2"/><path d="M16 8V5a2 2 0 0 0-2-2H5a2 2 0 0 0-2 2v9a2 2 0 0 0 2 2h3"/></svg>';
    b.addEventListener("click", async () => {
      try {
        await navigator.clipboard.writeText($("code", pre).textContent);
        b.classList.add("done");
        setTimeout(() => b.classList.remove("done"), 1400);
      } catch {}
    });
    pre.append(b);
  });

  // ------------------------------------------------------------ lightbox
  const box = $(".lightbox");
  const boxImg = box && $("img", box);
  document.addEventListener("click", (e) => {
    const a = e.target.closest("a[data-lightbox]");
    if (!a || !box || e.ctrlKey || e.metaKey) return;
    e.preventDefault();
    boxImg.src = a.href;
    boxImg.alt = $("img", a)?.alt ?? "";
    box.hidden = false;
  });
  box?.addEventListener("click", () => (box.hidden = true));
  document.addEventListener("keydown", (e) => e.key === "Escape" && box && (box.hidden = true));

  // ------------------------------------------------------------ home page
  $$("[data-show]").forEach((tab) =>
    tab.addEventListener("click", () => {
      $$("[data-show]").forEach((t) => t.setAttribute("aria-selected", String(t === tab)));
      $$("[data-pane]").forEach((p) => (p.hidden = p.dataset.pane !== tab.dataset.show));
    }),
  );
  $$(".os-alt [data-href]").forEach((s) => s.addEventListener("click", (e) => (e.preventDefault(), (location.href = s.dataset.href))));
  const dl = $("[data-download]");
  if (dl) {
    const p = (navigator.userAgentData?.platform || navigator.platform || "").toLowerCase();
    const cards = $$(".os-card");
    const pick = /mac/.test(p) ? 1 : /linux|x11/.test(p) && !/android/.test(navigator.userAgent.toLowerCase()) ? 2 : -1;
    if (pick > 0 && cards[pick]) {
      dl.href = cards[pick].href;
      dl.querySelector("span").textContent = pick === 1 ? dl.dataset.mac : dl.dataset.linux;
      dl.querySelector("svg").outerHTML = cards[pick].querySelector("svg").outerHTML;
    }
  }
  if ("IntersectionObserver" in window) {
    const io = new IntersectionObserver((entries) => entries.forEach((en) => en.isIntersecting && (en.target.classList.add("in"), io.unobserve(en.target))), { rootMargin: "0px 0px -8% 0px" });
    $$(".reveal").forEach((el) => io.observe(el));
  } else {
    $$(".reveal").forEach((el) => el.classList.add("in"));
  }
})();
