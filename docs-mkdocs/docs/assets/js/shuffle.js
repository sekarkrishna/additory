// shuffle.js — Shuffle component for additory docs
// Randomizes code examples across 8 domain datasets
(function () {
  "use strict";

  var scenarios = null;
  var currentScenarioId = null;

  // Resolve a dotted path with optional array indexing, e.g. "target.rows[0].id"
  function resolvePath(obj, path) {
    return path.split(".").reduce(function (acc, part) {
      if (acc == null) return undefined;
      var match = part.match(/^(\w+)\[(\d+)\]$/);
      if (match) {
        var arr = acc[match[1]];
        return arr ? arr[parseInt(match[2], 10)] : undefined;
      }
      return acc[part];
    }, obj);
  }

  // Replace {{placeholder}} tokens in a template string
  function renderTemplate(template, scenario) {
    return template.replace(/\{\{(.+?)\}\}/g, function (_, path) {
      var value = resolvePath(scenario, path.trim());
      return value !== undefined ? String(value) : "undefined";
    });
  }

  // Pick a random scenario different from the excluded id
  function pickRandom(exclude) {
    var candidates = scenarios.filter(function (s) {
      return s.id !== exclude;
    });
    return candidates[Math.floor(Math.random() * candidates.length)];
  }

  // Apply a random scenario to all shuffle containers on the page (or a single container)
  function applyRandomScenario(container) {
    if (!scenarios || scenarios.length === 0) return;

    var scenario = pickRandom(currentScenarioId);
    currentScenarioId = scenario.id;

    var scope = container || document;

    // Find all code elements with data-shuffle-template inside shuffle containers
    var codeBlocks = scope.querySelectorAll(
      ".shuffle-container [data-shuffle-template]"
    );

    codeBlocks.forEach(function (block) {
      var template = block.getAttribute("data-shuffle-template");
      if (template) {
        block.textContent = renderTemplate(template, scenario);
      }
    });

    // Update domain labels
    var labels = scope.querySelectorAll(".shuffle-domain");
    labels.forEach(function (label) {
      label.textContent = scenario.name;
    });
  }

  // Wire up all shuffle buttons on the page
  function initShuffleButtons() {
    document.querySelectorAll(".shuffle-btn").forEach(function (btn) {
      btn.addEventListener("click", function () {
        var container = btn.closest(".shuffle-container");
        applyRandomScenario(container || undefined);
      });
    });
  }

  // Bootstrap on DOMContentLoaded
  document.addEventListener("DOMContentLoaded", function () {
    // Determine base URL — works both on localhost and deployed site
    var base = document.querySelector('meta[name="base_url"]');
    var baseUrl = base ? base.getAttribute("content") : "";
    // Strip trailing slash
    if (baseUrl.endsWith("/")) baseUrl = baseUrl.slice(0, -1);

    var url = baseUrl + "/assets/data/scenarios.json";

    fetch(url)
      .then(function (resp) {
        if (!resp.ok) throw new Error("HTTP " + resp.status);
        return resp.json();
      })
      .then(function (data) {
        scenarios = data.scenarios;
        initShuffleButtons();
        // Only apply initial scenario if there are shuffle containers on this page
        if (document.querySelector(".shuffle-container")) {
          applyRandomScenario();
        }
      })
      .catch(function (e) {
        console.warn("Shuffle: Could not load scenarios.json", e);
      });
  });

  // Expose for manual triggering
  window.additoryShuffle = { applyRandomScenario: applyRandomScenario };
})();
