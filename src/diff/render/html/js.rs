//! JavaScript for generated diff pages, one concern per script.
//!
//! Scripts are static: no user-controlled data is ever interpolated into them.
//! They only reach the DOM through class/id selectors the renderers emit, which
//! keeps generated pages XSS-safe by construction.

/// Default theme before first paint: baked choice, else stored choice, else OS.
pub(super) const THEME_INIT_JS: &str = r#"
(function () {
  var t = document.documentElement.dataset.theme;
  if (!t) {
    var stored = localStorage.getItem("rustdiff-theme");
    t = (stored === "dark" || stored === "light")
      ? stored
      : (window.matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light");
    document.documentElement.dataset.theme = t;
  }
})();
"#;

/// Jump to the next/previous change row (`n`/`p`/`j`/`k` keys or toolbar
/// buttons). Change rows are `tr.add`, `tr.del` (unified/numbered) and
/// `tr.chg` (side-by-side).
pub(super) const NAVIGATION_JS: &str = r#"
(function () {
  var changes = Array.prototype.slice.call(
    document.querySelectorAll("tr.add, tr.del, tr.chg")
  );
  function jump(dir) {
    if (!changes.length) return;
    var target = null;
    var i;
    if (dir > 0) {
      for (i = 0; i < changes.length; i++) {
        if (changes[i].getBoundingClientRect().top > 0) { target = changes[i]; break; }
      }
    } else {
      for (i = changes.length - 1; i >= 0; i--) {
        if (changes[i].getBoundingClientRect().top < 0) { target = changes[i]; break; }
      }
    }
    if (!target) target = dir > 0 ? changes[0] : changes[changes.length - 1];
    target.scrollIntoView({ block: "center" });
  }
  document.getElementById("next-change").addEventListener("click", function () { jump(1); });
  document.getElementById("prev-change").addEventListener("click", function () { jump(-1); });
  document.addEventListener("keydown", function (e) {
    if (e.target.tagName === "INPUT" || e.target.tagName === "TEXTAREA") return;
    if (e.key === "n" || e.key === "j") jump(1);
    else if (e.key === "p" || e.key === "k") jump(-1);
  });
})();
"#;

/// Expand a collapsed unchanged-region gap row, revealing the hidden context
/// rows that follow it.
pub(super) const COLLAPSE_JS: &str = r#"
document.body.addEventListener("click", function (e) {
  if (e.target.classList.contains("expand")) {
    var gap = e.target.closest("tr.gap");
    var row = gap.nextElementSibling;
    while (row && row.classList.contains("collapsed")) {
      row.classList.remove("collapsed");
      row = row.nextElementSibling;
    }
    gap.classList.add("hidden");
    e.preventDefault();
  }
});
"#;

/// Toggle line wrapping (off = horizontal scroll), persisting the choice.
pub(super) const WRAP_JS: &str = r#"
(function () {
  var btn = document.getElementById("wrap-toggle");
  function apply(off) {
    document.body.classList.toggle("wrap-off", off);
    btn.textContent = off ? "Wrap on" : "Wrap off";
  }
  apply(localStorage.getItem("rustdiff-wrap") === "off");
  btn.addEventListener("click", function () {
    var off = !document.body.classList.contains("wrap-off");
    localStorage.setItem("rustdiff-wrap", off ? "off" : "on");
    apply(off);
  });
})();
"#;

/// Toggle the color theme at view time, persisting the choice.
pub(super) const THEME_TOGGLE_JS: &str = r#"
(function () {
  var root = document.documentElement;
  var btn = document.getElementById("theme-toggle");
  function apply(t) {
    root.dataset.theme = t;
    btn.textContent = t === "dark" ? "Light theme" : "Dark theme";
  }
  apply(root.dataset.theme || "dark");
  btn.addEventListener("click", function () {
    var next = root.dataset.theme === "dark" ? "light" : "dark";
    localStorage.setItem("rustdiff-theme", next);
    apply(next);
  });
})();
"#;
