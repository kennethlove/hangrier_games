(function() {
  "use strict";

  // ── Feed Tab Filtering ──────────────────────────────────────────
  var feedTabs = document.querySelectorAll(".feed-tab");
  var feedScroll = document.getElementById("feedScroll");

  // Restore saved filter from localStorage
  var savedFilter = localStorage.getItem("feedFilter") || "all";
  feedTabs.forEach(function(tab) {
    if (tab.textContent.trim().toLowerCase() === savedFilter) {
      tab.classList.add("active");
    } else {
      tab.classList.remove("active");
    }
  });

  // Apply filter on load
  if (feedScroll) {
    var filter = savedFilter === "all" ? "" : savedFilter;
    var cards = feedScroll.querySelectorAll(".event-card");
    cards.forEach(function(card) {
      if (!filter || card.classList.contains(filter)) {
        card.style.display = "";
      } else {
        card.style.display = "none";
      }
    });
  }

  feedTabs.forEach(function(tab) {
    tab.addEventListener("click", function() {
      feedTabs.forEach(function(t) { t.classList.remove("active"); });
      tab.classList.add("active");

      var filter = tab.textContent.trim().toLowerCase();
      localStorage.setItem("feedFilter", filter);
      if (filter === "all") filter = "";

      var cards = feedScroll.querySelectorAll(".event-card");
      cards.forEach(function(card) {
        if (!filter || card.classList.contains(filter)) {
          card.style.display = "";
        } else {
          card.style.display = "none";
        }
      });
    });
  });

  // ── Hex Map Zoom / Pan ──────────────────────────────────────────
  var mapContainer = document.querySelector(".map-container");
  var hexMap = mapContainer ? mapContainer.querySelector(".hex-map") : null;

  if (hexMap) {
    var scale = 1, tx = 0, ty = 0;
    var dragging = false, startX = 0, startY = 0;

    function applyTransform() {
      hexMap.style.transform = "translate(" + tx + "px," + ty + "px) scale(" + scale + ")";
      hexMap.style.transformOrigin = "center center";
    }

    mapContainer.addEventListener("wheel", function(e) {
      e.preventDefault();
      var delta = e.deltaY > 0 ? 0.9 : 1.1;
      scale = Math.min(4, Math.max(0.5, scale * delta));
      applyTransform();
    }, { passive: false });

    mapContainer.addEventListener("mousedown", function(e) {
      if (e.button !== 0) return;
      dragging = true;
      startX = e.clientX - tx;
      startY = e.clientY - ty;
      mapContainer.style.cursor = "grabbing";
    });

    window.addEventListener("mousemove", function(e) {
      if (!dragging) return;
      tx = e.clientX - startX;
      ty = e.clientY - startY;
      applyTransform();
    });

    window.addEventListener("mouseup", function() {
      dragging = false;
      mapContainer.style.cursor = "grab";
    });

    mapContainer.addEventListener("dblclick", function() {
      scale = 1; tx = 0; ty = 0;
      applyTransform();
    });

    // Touch support
    var lastTouchDist = 0;
    mapContainer.addEventListener("touchstart", function(e) {
      if (e.touches.length === 1) {
        dragging = true;
        startX = e.touches[0].clientX - tx;
        startY = e.touches[0].clientY - ty;
      } else if (e.touches.length === 2) {
        var dx = e.touches[0].clientX - e.touches[1].clientX;
        var dy = e.touches[0].clientY - e.touches[1].clientY;
        lastTouchDist = Math.sqrt(dx * dx + dy * dy);
      }
    }, { passive: true });

    mapContainer.addEventListener("touchmove", function(e) {
      if (e.touches.length === 1 && dragging) {
        tx = e.touches[0].clientX - startX;
        ty = e.touches[0].clientY - startY;
        applyTransform();
      } else if (e.touches.length === 2) {
        var dx = e.touches[0].clientX - e.touches[1].clientX;
        var dy = e.touches[0].clientY - e.touches[1].clientY;
        var dist = Math.sqrt(dx * dx + dy * dy);
        if (lastTouchDist > 0) {
          scale = Math.min(4, Math.max(0.5, scale * (dist / lastTouchDist)));
          applyTransform();
        }
        lastTouchDist = dist;
      }
    }, { passive: true });

    mapContainer.addEventListener("touchend", function() {
      dragging = false;
      lastTouchDist = 0;
    });
  }

  // ── Roster Sort ─────────────────────────────────────────────────
  var rosterScroll = document.querySelector(".roster-scroll");
  if (rosterScroll) {
    var sortBtn = document.querySelector(".roster-sort");
    var sortModes = ["district", "health"];

    // Save original HTML (with group wrappers) for restoring district view
    var originalRosterHTML = rosterScroll.innerHTML;

    // Restore saved sort mode from localStorage
    var savedMode = localStorage.getItem("rosterSort") || "district";
    var sortIdx = sortModes.indexOf(savedMode);
    if (sortIdx < 0) sortIdx = 0;
    if (sortBtn) sortBtn.textContent = sortModes[sortIdx].toUpperCase();

    if (sortBtn) {
      sortBtn.addEventListener("click", function() {
        sortIdx = (sortIdx + 1) % sortModes.length;
        var mode = sortModes[sortIdx];
        sortBtn.textContent = mode.toUpperCase();
        localStorage.setItem("rosterSort", mode);

        if (mode === "district") {
          // Restore original server-rendered HTML (with district group wrappers)
          rosterScroll.innerHTML = originalRosterHTML;
        } else {
          // Flatten: extract roster-rows from group wrappers, discard headers
          var groups = rosterScroll.querySelectorAll(".district-group, .alliance-group");
          groups.forEach(function(g) {
            var rows = g.querySelectorAll(".roster-row");
            rows.forEach(function(r) { rosterScroll.appendChild(r); });
            g.remove();
          });

          var rows = Array.from(rosterScroll.querySelectorAll(".roster-row"));
          rows.sort(function(a, b) {
            if (mode === "health") {
              var aW = a.querySelector(".roster-health-fill");
              var bW = b.querySelector(".roster-health-fill");
              var aPct = aW ? parseInt(aW.style.width) || 0 : 0;
              var bPct = bW ? parseInt(bW.style.width) || 0 : 0;
              return bPct - aPct;
            }
            return 0;
          });
          rows.forEach(function(r) { rosterScroll.appendChild(r); });
        }
      });
    }
  }
})();
