// Fridgly front-end interactions: add-sheet, quantity stepper, expiry chips.
// Kept intentionally small — htmx drives all the data mutations.

function openSheet() {
    document.getElementById("add-sheet").classList.add("open");
    document.getElementById("sheet-overlay").classList.add("open");
    const name = document.getElementById("add-name");
    if (name) setTimeout(() => name.focus(), 50);
}

function closeSheet() {
    document.getElementById("add-sheet").classList.remove("open");
    document.getElementById("sheet-overlay").classList.remove("open");
}

// Increment/decrement the quantity when it is a plain integer; leave free-form
// values (e.g. "1L", "2 bags") untouched.
function stepQty(btn, delta) {
    const input = btn.parentElement.querySelector(".qty-input");
    if (!input) return;
    const value = input.value.trim();
    if (/^\d+$/.test(value)) {
        input.value = Math.max(0, parseInt(value, 10) + delta);
    } else if (value === "") {
        input.value = Math.max(0, delta);
    }
}

function chipGroup(el) {
    return el.closest("[data-expiry-group]");
}

function dateInputFor(el) {
    return el.closest("form").querySelector(".date-input");
}

function clearActive(group) {
    group.querySelectorAll(".chip").forEach((c) => c.classList.remove("active"));
}

// Preset chips (Today / +3d / +1wk / +1mo): set the hidden date and hide the picker.
function pickExpiry(btn) {
    const group = chipGroup(btn);
    const date = dateInputFor(btn);
    const days = parseInt(btn.dataset.days, 10);
    const d = new Date();
    d.setDate(d.getDate() + days);
    date.value = d.toISOString().slice(0, 10);
    clearActive(group);
    btn.classList.add("active");
    date.classList.add("hidden");
}

// "pick date" chip: reveal the native date picker.
function pickDate(btn) {
    const group = chipGroup(btn);
    const date = dateInputFor(btn);
    clearActive(group);
    btn.classList.add("active");
    date.classList.remove("hidden");
    date.focus();
    if (typeof date.showPicker === "function") {
        try {
            date.showPicker();
        } catch (_) {
            /* showPicker may throw if not user-activated; ignore */
        }
    }
}

// Reset the add-sheet controls after a successful submit.
function resetSheetControls() {
    document.querySelectorAll("#add-form .chip").forEach((c) => c.classList.remove("active"));
    const date = document.querySelector("#add-form .date-input");
    if (date) {
        date.value = "";
        date.classList.add("hidden");
    }
    const qty = document.querySelector("#add-form .qty-input");
    if (qty) qty.value = "1";
}

// Let keyboard users open the edit form on a row (role="button").
document.addEventListener("keydown", (e) => {
    const target = e.target;
    if (
        target &&
        target.classList &&
        target.classList.contains("row-body") &&
        (e.key === "Enter" || e.key === " ")
    ) {
        e.preventDefault();
        target.click();
    }
});
