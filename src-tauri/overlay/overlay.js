// HighBazar chat overlay: renders the live chat sent by the app as
// Server-Sent Events. Plain JavaScript, no framework, few DOM nodes.
(() => {
  "use strict";
  const key = new URLSearchParams(location.search).get("k") || "";
  const feed = document.getElementById("feed");
  const chat = document.getElementById("chat");
  const statusEl = document.getElementById("status");
  const MAX_LINES = 80;

  let settings = { showBot: true, showEvents: true, showTranslations: true, hideAfterSecs: 0, background: 0.55, showEmotes: true, chatZoom: 1, chatScroll: { smooth: true, durationMs: 450, easing: "easeInOut" }, language: "auto", chatInput: false };
  let emotes = {};
  const translations = new Map();

  // ---------------------------------------------------------------- texts
  const EVENTS = {
    de: {
      follow: "{user} folgt jetzt", sub: "{user} hat abonniert {tier}", resub: "{user} hat für {months} Monate abonniert",
      gift: "{user} hat {recipient} ein Abo geschenkt", communityGift: "{user} hat der Community {count} Abos geschenkt",
      upgrade: "{user} hat ein geschenktes bzw. Prime-Abo in ein bezahltes umgewandelt", payForward: "{user} hat ein geschenktes Abo weitergeschenkt",
      watchStreak: "{user} hat {streak} Streams in Folge geschaut", bitsBadge: "{user} hat das {bitsTier}-Bits-Abzeichen freigeschaltet",
      charity: "{user} hat für die Spendenaktion gespendet", raid: "{user} raidet mit {viewers} Zuschauern",
    },
  };
  const lang = () => (settings.language === "auto" ? (navigator.language || "en").slice(0, 2) : settings.language);
  function eventText(item) {
    const tpl = item.key && EVENTS[lang()] && EVENTS[lang()][item.key];
    if (!tpl) return item.text;
    const p = item.params || {};
    return tpl.replace(/\{(\w+)\}/g, (_, k) => (p[k] ?? "")).replace(/\s+$/, "");
  }

  // ---------------------------------------------------------------- colors
  // Twitch name colors can be unreadable on a dark background: lift them.
  function readable(hex) {
    const m = /^#?([0-9a-f]{6})$/i.exec(hex || "");
    if (!m) return "#c4a5ff";
    let [r, g, b] = [0, 2, 4].map((i) => parseInt(m[1].slice(i, i + 2), 16));
    const lum = (0.2126 * r + 0.7152 * g + 0.0722 * b) / 255;
    if (lum < 0.45) {
      const f = (0.45 - lum) / (1 - lum);
      [r, g, b] = [r, g, b].map((c) => Math.round(c + (255 - c) * Math.min(0.75, f * 1.6)));
    }
    return `rgb(${r},${g},${b})`;
  }

  // ---------------------------------------------------------------- render
  function appendText(el, text, msgEmotes) {
    const parts = String(text).split(/(\s+)/);
    let buf = "";
    for (const p of parts) {
      const url = settings.showEmotes && p.trim() ? (msgEmotes && msgEmotes[p]) || emotes[p] : null;
      if (url) {
        if (buf) el.appendChild(document.createTextNode(buf));
        buf = "";
        const img = document.createElement("img");
        img.className = "emote";
        img.src = url;
        img.alt = p;
        img.title = p;
        img.decoding = "async";
        el.appendChild(img);
      } else buf += p;
    }
    if (buf) el.appendChild(document.createTextNode(buf));
  }

  function visible(item) {
    if (item.kind === "bot") return settings.showBot;
    if (item.kind === "event") return settings.showEvents;
    return item.kind === "chat";
  }

  function renderTranslation(line, t) {
    if (!settings.showTranslations || !t) return;
    let el = line.querySelector(".tr");
    if (!el) {
      el = document.createElement("span");
      el.className = "tr";
      line.appendChild(el);
    }
    el.textContent = "";
    const b = document.createElement("b");
    b.textContent = t.lang + ": ";
    el.appendChild(b);
    appendText(el, t.text, null);
  }

  function addLine(item, animate) {
    if (!visible(item)) return;
    const line = document.createElement("div");
    line.className = "line " + item.kind + (item.shared ? " shared" : "");
    line.dataset.mid = item.messageId || "";
    line.dataset.uid = item.userId || "";
    if (item.kind === "event") {
      line.textContent = eventText(item);
    } else {
      const name = document.createElement("span");
      name.className = "name";
      name.textContent = item.user || "HighBazar";
      name.style.color = item.kind === "bot" ? "#c4a5ff" : readable(item.color);
      const sep = document.createElement("span");
      sep.className = "sep";
      sep.textContent = ": ";
      const msg = document.createElement("span");
      appendText(msg, item.text, item.emotes);
      line.append(name, sep, msg);
      renderTranslation(line, translations.get(item.messageId));
    }
    if (animate && fadeMs() > 0) line.classList.add("in");
    chat.appendChild(line);
    while (chat.children.length > MAX_LINES) chat.firstElementChild.remove();
    if (settings.hideAfterSecs > 0) setTimeout(() => line.classList.add("gone"), settings.hideAfterSecs * 1000);
  }

  function applySettings() {
    applyInput();
    document.documentElement.style.setProperty("--bg", String(settings.background));
    document.documentElement.style.setProperty("--in-ms", fadeMs() + "ms");
    chat.style.zoom = String(settings.chatZoom || 1);
    document.documentElement.lang = lang();
  }
  const fadeMs = () => (settings.chatScroll && settings.chatScroll.smooth ? Math.min(400, settings.chatScroll.durationMs) : 0);

  // ---------------------------------------------------------------- glide
  // Same motion as the app's live feed (src/lib/smoothFollow.ts).
  const EASE = {
    easeInOut: (s) => (s < 0.5 ? 4 * s * s * s : 1 - Math.pow(-2 * s + 2, 3) / 2),
    easeOut: (s) => 1 - Math.pow(1 - s, 3),
    linear: (s) => s,
  };
  function glide(easing, dur, d0, v0, t) {
    dur = Math.max(1, dur);
    if (easing === "spring") {
      const w = 6 / dur;
      return (d0 + (v0 + w * d0) * t) * Math.exp(-w * t);
    }
    const s = Math.min(1, Math.max(0, t / dur));
    return d0 * (1 - (EASE[easing] || EASE.easeInOut)(s)) + v0 * dur * s * (1 - s) * (1 - s);
  }
  let raf = 0, d0 = 0, v0 = 0, t0 = 0, lastHeight = 0;
  const max = () => feed.scrollHeight - feed.clientHeight;
  const setDistance = (d) => (feed.scrollTop = Math.max(0, max() - d));
  function velocity(cfg) {
    const t = performance.now() - t0;
    return (glide(cfg.easing, cfg.durationMs, d0, v0, t + 8) - glide(cfg.easing, cfg.durationMs, d0, v0, t)) / 8;
  }
  function frame(now) {
    const cfg = settings.chatScroll;
    const t = now - t0;
    let d = glide(cfg.easing, cfg.durationMs, d0, v0, t);
    const done = cfg.easing === "spring" ? Math.abs(d) < 0.5 && t > 50 : t >= cfg.durationMs;
    if (done || d < 0) d = 0;
    setDistance(d);
    if (d === 0 && (done || cfg.easing !== "spring")) {
      raf = 0;
      return;
    }
    raf = requestAnimationFrame(frame);
  }
  function follow() {
    const h = feed.scrollHeight;
    if (h === lastHeight && !raf) return setDistance(0);
    lastHeight = h;
    const cfg = settings.chatScroll;
    if (!cfg || !cfg.smooth || matchMedia("(prefers-reduced-motion: reduce)").matches) {
      cancelAnimationFrame(raf);
      raf = 0;
      return setDistance(0);
    }
    let d = Math.max(0, max() - feed.scrollTop);
    d = Math.min(d, feed.clientHeight * 1.5);
    v0 = raf ? velocity(cfg) : 0;
    d0 = d;
    t0 = performance.now();
    setDistance(d);
    if (!raf) raf = requestAnimationFrame(frame);
  }
  new ResizeObserver(follow).observe(chat);
  new ResizeObserver(follow).observe(feed);

  // ---------------------------------------------------------------- events
  function reset(state) {
    settings = Object.assign(settings, state.settings || {});
    emotes = state.emotes || {};
    translations.clear();
    for (const t of state.translations || []) translations.set(t.messageId, t);
    applySettings();
    chat.textContent = "";
    for (const item of state.items || []) addLine(item, false);
    lastHeight = 0;
    cancelAnimationFrame(raf);
    raf = 0;
    setDistance(0);
  }

  // ---------------------------------------------------------------- chat input
  // Optional input to write in chat (Shift+Tab into the Steam overlay, reply,
  // Enter). Suggestions come from the app, so they rank exactly like there:
  // ":" emotes and emojis (":7 pog" = 7TV only), "@" people, Tab completes.
  const say = document.getElementById("say");
  const sayIn = document.getElementById("say-in");
  const sayBtn = document.getElementById("say-btn");
  const sug = document.getElementById("sug");
  const T = {
    de: { ph: "Nachricht … ( : Emotes, @ Leute, Tab ergänzt)", hint: "↑↓ wählen · Enter/Tab einfügen · :7 :b :f :t :e = Anbieter", off: "HighBazar: Bot läuft nicht" },
    en: { ph: "Message … ( : emotes, @ people, Tab completes)", hint: "↑↓ choose · Enter/Tab insert · :7 :b :f :t :e = provider", off: "HighBazar: the bot is not running" },
  };
  const tr = () => T[lang()] || T.en;
  const PROVIDER = { twitch: "Twitch", bttv: "BTTV", ffz: "FFZ", "7tv": "7TV", emoji: "Emoji" };
  let items = [];
  let activeIdx = 0;
  let seq = 0;
  let tabCycle = null;
  const sent = [];
  let histPos = -1;

  function activeToken(text, caret) {
    const before = text.slice(0, caret);
    const colon = before.match(/(^|\s):((?:[7bfteBFTE] )?[^\s:]*)$/);
    if (colon) return { mode: "colon", query: colon[2], start: caret - colon[2].length - 1 };
    const at = before.match(/(^|\s)@([\p{L}\p{N}_]*)$/u);
    if (at) return { mode: "mention", query: at[2], start: caret - at[2].length - 1 };
    return null;
  }
  async function suggest(q, mode, limit) {
    const url = `/suggest?k=${encodeURIComponent(key)}&m=${mode}&lang=${lang()}&q=${encodeURIComponent(q)}&n=${limit}`;
    try {
      const r = await fetch(url);
      return r.ok ? await r.json() : [];
    } catch {
      return [];
    }
  }
  function renderSug() {
    sug.textContent = "";
    sug.hidden = !items.length;
    sayIn.setAttribute("aria-expanded", String(items.length > 0));
    items.forEach((s, i) => {
      const li = document.createElement("li");
      li.setAttribute("role", "option");
      li.setAttribute("aria-selected", String(i === activeIdx));
      if (s.url) {
        const img = document.createElement("img");
        img.src = s.url;
        img.alt = "";
        li.append(img);
      } else {
        const b = document.createElement("span");
        b.className = s.kind === "emoji" ? "emoji" : "at";
        b.textContent = s.kind === "emoji" ? s.insert : "@";
        li.append(b);
      }
      const lbl = document.createElement("span");
      lbl.className = "lbl";
      lbl.textContent = s.kind === "emoji" ? s.label : s.insert;
      li.append(lbl);
      if (s.kind !== "user") {
        const p = document.createElement("span");
        p.className = "prov";
        p.textContent = s.scope === "sub" ? (lang() === "de" ? "Abo" : "Sub") : PROVIDER[s.provider] || "";
        li.append(p);
      }
      li.addEventListener("mousedown", (e) => {
        e.preventDefault();
        pick(s);
      });
      sug.append(li);
    });
    if (items.length) {
      const h = document.createElement("li");
      h.className = "hint";
      h.textContent = tr().hint;
      sug.append(h);
    }
  }
  async function refreshSug() {
    const tok = activeToken(sayIn.value, sayIn.selectionStart ?? sayIn.value.length);
    if (!tok) {
      items = [];
      renderSug();
      return;
    }
    const mine = ++seq;
    const r = await suggest(tok.query, tok.mode, 8);
    if (mine !== seq) return;
    items = r.slice(0, 8);
    activeIdx = 0;
    renderSug();
  }
  function pick(s) {
    const caret = sayIn.selectionStart ?? sayIn.value.length;
    const tok = activeToken(sayIn.value, caret);
    if (!tok) return;
    const after = sayIn.value.slice(caret).replace(/^\S*/, "");
    const ins = after.startsWith(" ") ? s.insert : s.insert + " ";
    sayIn.value = sayIn.value.slice(0, tok.start) + ins + after;
    const pos = tok.start + ins.length + (after.startsWith(" ") ? 1 : 0);
    sayIn.setSelectionRange(pos, pos);
    items = [];
    renderSug();
  }
  async function tabComplete(back) {
    if (!tabCycle) {
      const caret = sayIn.selectionStart ?? sayIn.value.length;
      const m = sayIn.value.slice(0, caret).match(/(\S+)$/);
      if (!m) return;
      const list = await suggest(m[1].replace(/^@/, ""), "tab", 20);
      if (!list.length) return;
      tabCycle = { start: caret - m[1].length, end: caret, list, i: back ? list.length - 1 : 0 };
    } else {
      const n = tabCycle.list.length;
      tabCycle.i = (tabCycle.i + (back ? n - 1 : 1)) % n;
    }
    const ins = tabCycle.list[tabCycle.i].insert + " ";
    sayIn.value = sayIn.value.slice(0, tabCycle.start) + ins + sayIn.value.slice(tabCycle.end);
    tabCycle.end = tabCycle.start + ins.length;
    sayIn.setSelectionRange(tabCycle.end, tabCycle.end);
  }
  function flash(text) {
    statusEl.textContent = text;
    statusEl.hidden = false;
    clearTimeout(flash.t);
    flash.t = setTimeout(() => (statusEl.hidden = true), 6000);
  }
  async function send() {
    const text = sayIn.value.trim();
    if (!text || sayBtn.disabled) return;
    sayBtn.disabled = true;
    try {
      const r = await fetch(`/send?k=${encodeURIComponent(key)}`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ text }),
      });
      const j = await r.json().catch(() => ({}));
      if (j.ok) {
        sent.push(text);
        if (sent.length > 30) sent.shift();
        sayIn.value = "";
        items = [];
        renderSug();
      } else {
        flash(j.error || tr().off);
      }
    } catch {
      flash(tr().off);
    } finally {
      sayBtn.disabled = false;
      histPos = -1;
      sayIn.focus();
    }
  }
  sayIn.addEventListener("input", () => {
    tabCycle = null;
    histPos = -1;
    refreshSug();
  });
  sayIn.addEventListener("blur", () => {
    items = [];
    renderSug();
  });
  sayIn.addEventListener("keydown", (e) => {
    const open = items.length > 0;
    if (open && (e.key === "ArrowDown" || e.key === "ArrowUp")) {
      e.preventDefault();
      activeIdx = (activeIdx + (e.key === "ArrowDown" ? 1 : items.length - 1)) % items.length;
      renderSug();
    } else if (open && (e.key === "Enter" || e.key === "Tab")) {
      e.preventDefault();
      pick(items[activeIdx]);
    } else if (open && e.key === "Escape") {
      items = [];
      renderSug();
    } else if (e.key === "Tab") {
      e.preventDefault();
      tabComplete(e.shiftKey);
    } else if (e.key === "ArrowUp" && sent.length && (!sayIn.value || histPos >= 0)) {
      e.preventDefault();
      histPos = Math.min(histPos + 1, sent.length - 1);
      sayIn.value = sent[sent.length - 1 - histPos];
    } else if (e.key === "ArrowDown" && histPos >= 0) {
      e.preventDefault();
      histPos--;
      sayIn.value = histPos >= 0 ? sent[sent.length - 1 - histPos] : "";
    } else if (e.key !== "Shift") {
      tabCycle = null;
    }
  });
  say.addEventListener("submit", (e) => {
    e.preventDefault();
    if (!items.length) send();
  });
  // Opening the overlay should let you type right away.
  window.addEventListener("focus", () => !say.hidden && sayIn.focus());
  function applyInput() {
    const on = !!settings.chatInput;
    say.hidden = !on;
    document.body.classList.toggle("typing", on);
    sayIn.placeholder = tr().ph;
    if (on && document.hasFocus()) sayIn.focus();
  }

  function connect() {
    const es = new EventSource("/events?k=" + encodeURIComponent(key));
    const on = (name, fn) => es.addEventListener(name, (e) => fn(JSON.parse(e.data)));
    on("state", (s) => {
      statusEl.hidden = true;
      reset(s);
    });
    on("msg", (item) => addLine(item, true));
    on("tr", (t) => {
      translations.set(t.messageId, t);
      if (translations.size > 300) translations.delete(translations.keys().next().value);
      const line = chat.querySelector(`.line[data-mid="${CSS.escape(t.messageId)}"]`);
      if (line) renderTranslation(line, t);
    });
    on("remove", (r) => {
      for (const l of [...chat.children]) {
        if ((r.messageId && l.dataset.mid === r.messageId) || (r.userId && l.dataset.uid === r.userId && l.classList.contains("chat"))) l.remove();
      }
    });
    on("emotes", (e) => (emotes = e || {}));
    on("settings", (s) => {
      const rerender = s.showBot !== settings.showBot || s.showEvents !== settings.showEvents || s.showEmotes !== settings.showEmotes || s.showTranslations !== settings.showTranslations;
      Object.assign(settings, s);
      applySettings();
      if (rerender) {
        es.close();
        connect();
      }
    });
    on("clear", () => (chat.textContent = ""));
    es.onerror = () => {
      statusEl.textContent = lang() === "de" ? "Keine Verbindung zu HighBazar – läuft die App?" : "No connection to HighBazar – is the app running?";
      statusEl.hidden = false;
    };
  }
  connect();
})();
