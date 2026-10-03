// Site chrome: the contents toggle, the search box, copy buttons, the
// table of contents, and the gallery's card / pipe switch.

(() => {
  const root = window.MALEVICH_ROOT ?? "./";

  // Contents toggle on narrow screens.
  const toggle = document.querySelector(".nav-toggle");
  const sidebar = document.getElementById("sidebar");
  if (toggle && sidebar) {
    toggle.addEventListener("click", () => {
      const open = sidebar.classList.toggle("open");
      toggle.setAttribute("aria-expanded", String(open));
    });
  }

  // Copy buttons on code blocks.
  for (const block of document.querySelectorAll(".code")) {
    const pre = block.querySelector("pre");
    if (!pre) continue;
    const button = document.createElement("button");
    button.type = "button";
    button.className = "copy";
    button.textContent = "copy";
    button.addEventListener("click", async () => {
      try {
        await navigator.clipboard.writeText(pre.textContent ?? "");
        button.textContent = "copied";
      } catch {
        button.textContent = "select it";
      }
      window.setTimeout(() => {
        button.textContent = "copy";
      }, 1400);
    });
    block.append(button);
  }

  // The table of contents follows the section in view.
  const toc = document.querySelector(".toc");
  if (toc && "IntersectionObserver" in window) {
    const links = new Map();
    for (const link of toc.querySelectorAll("a[href^='#']")) {
      links.set(decodeURIComponent(link.getAttribute("href").slice(1)), link);
    }
    const headings = [...links.keys()].map((id) => document.getElementById(id)).filter(Boolean);
    let current = null;
    const mark = (id) => {
      if (current) current.removeAttribute("aria-current");
      current = links.get(id) ?? null;
      if (current) current.setAttribute("aria-current", "true");
    };
    const observer = new IntersectionObserver(
      (entries) => {
        for (const entry of entries) {
          if (entry.isIntersecting) mark(entry.target.id);
        }
      },
      { rootMargin: "-10% 0px -80% 0px" },
    );
    for (const heading of headings) observer.observe(heading);
  }

  // Gallery: the card is the default. "What a pipe sees" reveals the bytes.
  const switcher = document.querySelector(".gallery-switch");
  if (switcher) {
    const buttons = [...switcher.querySelectorAll("button[data-view]")];
    const setView = (view) => {
      document.body.classList.toggle("gallery-pipes", view === "pipe");
      for (const button of buttons) {
        button.setAttribute("aria-pressed", String(button.dataset.view === view));
      }
    };
    for (const button of buttons) {
      button.addEventListener("click", () => setView(button.dataset.view));
    }
  }
  const galleryIndex = document.querySelector(".gallery-index");
  if (galleryIndex && "IntersectionObserver" in window) {
    const links = new Map();
    for (const link of galleryIndex.querySelectorAll("a[href^='#']")) {
      links.set(decodeURIComponent(link.getAttribute("href").slice(1)), link);
    }
    const sections = [...links.keys()]
      .map((id) => document.getElementById(id))
      .filter(Boolean);
    let current = null;
    const mark = (id) => {
      if (current) current.removeAttribute("aria-current");
      current = links.get(id) ?? null;
      if (current) current.setAttribute("aria-current", "true");
    };
    const sectionObserver = new IntersectionObserver(
      (entries) => {
        const visible = entries
          .filter((entry) => entry.isIntersecting)
          .sort((a, b) => a.target.getBoundingClientRect().top - b.target.getBoundingClientRect().top);
        if (visible[0]) mark(visible[0].target.id);
      },
      { rootMargin: "-15% 0px -75% 0px" },
    );
    for (const heading of sections) sectionObserver.observe(heading);
    if (sections[0]) mark(sections[0].id);
  }

  // Search: a small index, substring matching, keyboard navigable.
  const input = document.getElementById("search");
  const results = document.getElementById("results");
  if (!input || !results) return;
  let index = null;
  let selected = -1;
  const load = async () => {
    if (index) return index;
    const response = await fetch(`${root}search.json`);
    index = await response.json();
    return index;
  };
  const escape = (text) =>
    text.replace(/[&<>"]/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;" })[c]);
  const snippet = (text, query) => {
    const at = text.toLowerCase().indexOf(query);
    if (at < 0) return text.slice(0, 110);
    const start = Math.max(0, at - 40);
    return (start > 0 ? "…" : "") + text.slice(start, start + 120) + "…";
  };
  const score = (entry, query) => {
    const title = entry.title.toLowerCase();
    if (title === query) return 100;
    if (title.startsWith(query)) return 60;
    if (title.includes(query)) return 40;
    const heading = entry.headings.find((h) => h.text.toLowerCase().includes(query));
    if (heading) return 25;
    if (entry.text.toLowerCase().includes(query)) return 10;
    return 0;
  };
  const render = async () => {
    const query = input.value.trim().toLowerCase();
    selected = -1;
    if (query.length < 2) {
      results.hidden = true;
      results.innerHTML = "";
      return;
    }
    const entries = await load();
    const hits = entries
      .map((entry) => ({ entry, score: score(entry, query) }))
      .filter((hit) => hit.score > 0)
      .sort((a, b) => b.score - a.score)
      .slice(0, 8);
    if (hits.length === 0) {
      results.innerHTML = `<p class="empty">Nothing matches “${escape(input.value.trim())}”.</p>`;
      results.hidden = false;
      return;
    }
    results.innerHTML = hits
      .map(({ entry }) => {
        const heading = entry.headings.find((h) => h.text.toLowerCase().includes(query));
        const anchor = heading ? `#${heading.id}` : "";
        const where = heading ? `${entry.section} › ${escape(entry.title)}` : entry.section;
        const label = heading ? escape(heading.text) : escape(entry.title);
        return `<a href="${root}${entry.url.replace(/^\//, "")}${anchor}"><span class="where">${where}</span>${label}<span class="snippet">${escape(snippet(entry.text, query))}</span></a>`;
      })
      .join("");
    results.hidden = false;
  };
  input.addEventListener("input", render);
  input.addEventListener("focus", render);
  input.addEventListener("keydown", (event) => {
    const items = [...results.querySelectorAll("a")];
    if (event.key === "Escape") {
      results.hidden = true;
      input.blur();
      return;
    }
    if (items.length === 0) return;
    if (event.key === "ArrowDown" || event.key === "ArrowUp") {
      event.preventDefault();
      items[selected]?.removeAttribute("aria-selected");
      selected = event.key === "ArrowDown" ? (selected + 1) % items.length : (selected - 1 + items.length) % items.length;
      items[selected].setAttribute("aria-selected", "true");
      items[selected].scrollIntoView({ block: "nearest" });
    }
    if (event.key === "Enter" && selected >= 0) {
      event.preventDefault();
      items[selected].click();
    }
  });
  document.addEventListener("click", (event) => {
    if (!results.contains(event.target) && event.target !== input) results.hidden = true;
  });
  document.addEventListener("keydown", (event) => {
    if (event.key === "/" && document.activeElement !== input && !/INPUT|TEXTAREA/.test(document.activeElement?.tagName ?? "")) {
      event.preventDefault();
      input.focus();
    }
  });
})();
