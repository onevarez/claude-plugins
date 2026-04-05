// So Cursor Tracker — logs interaction events to console for retrieval.
//
// Uses console.log with a [SO_CURSOR] prefix so events can be filtered
// from Playwright's console output via browser_console_messages.
//
// Mouse moves throttled to ~10Hz. Clicks include DOM path. Scrolls immediate.

(() => {
  "use strict";

  const PREFIX = "[SO_CURSOR]";
  const MOVE_THROTTLE_MS = 100;

  const startTime = Date.now();
  let lastMoveTime = 0;

  function domPath(el) {
    const parts = [];
    while (el && el !== document.body && parts.length < 4) {
      let tag = el.tagName?.toLowerCase() || "";
      if (el.id) tag += "#" + el.id;
      else if (el.className && typeof el.className === "string")
        tag += "." + el.className.split(" ")[0];
      if (tag) parts.unshift(tag);
      el = el.parentElement;
    }
    return parts.join(" > ");
  }

  document.addEventListener(
    "mousemove",
    (e) => {
      const now = Date.now();
      if (now - lastMoveTime < MOVE_THROTTLE_MS) return;
      lastMoveTime = now;
      console.log(
        PREFIX,
        JSON.stringify({ t: now - startTime, type: "move", x: e.clientX, y: e.clientY })
      );
    },
    { passive: true }
  );

  document.addEventListener(
    "click",
    (e) => {
      console.log(
        PREFIX,
        JSON.stringify({
          t: Date.now() - startTime,
          type: "click",
          x: e.clientX,
          y: e.clientY,
          target: e.target?.tagName || "",
          path: domPath(e.target),
        })
      );
    },
    { passive: true }
  );

  document.addEventListener(
    "scroll",
    () => {
      console.log(
        PREFIX,
        JSON.stringify({
          t: Date.now() - startTime,
          type: "scroll",
          x: 0,
          y: 0,
          scrollX: Math.round(window.scrollX),
          scrollY: Math.round(window.scrollY),
        })
      );
    },
    { passive: true, capture: true }
  );

  console.log(PREFIX, JSON.stringify({ t: 0, type: "init", url: location.href }));
})();
