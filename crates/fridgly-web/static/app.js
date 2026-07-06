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

// ---- Barcode scanning ----
//
// Uses the browser-native BarcodeDetector API (widely available on mobile) to
// read a barcode from the rear camera, then looks it up via /products/:barcode
// and pre-fills the add sheet. A manual entry field is always available as a
// fallback for browsers without BarcodeDetector or camera access.

let scanStream = null;
let scanActive = false;
let barcodeDetector = null;

function scanPanel() {
    return document.getElementById("scan-sheet");
}

// Read a translated status message stored as a data-msg-* attribute on the
// panel (keeps copy server-side/localised). key "notfound" -> data-msg-notfound.
function scanMsg(key) {
    const panel = scanPanel();
    return (panel && panel.dataset["msg" + key]) || "";
}

function setScanStatus(text) {
    const el = document.getElementById("scan-status");
    if (el) el.textContent = text;
}

function scannerIsOpen() {
    const panel = scanPanel();
    return panel && panel.classList.contains("open");
}

async function openScanner() {
    scanPanel().classList.add("open");
    document.getElementById("scan-overlay").classList.add("open");
    const input = document.getElementById("scan-input");
    if (input) input.value = "";
    await startCamera();
}

function closeScanner() {
    stopCamera();
    const panel = scanPanel();
    if (panel) panel.classList.remove("open");
    document.getElementById("scan-overlay").classList.remove("open");
}

async function startCamera() {
    const video = document.getElementById("scan-video");
    if (!("BarcodeDetector" in window) || !navigator.mediaDevices) {
        setScanStatus(scanMsg("Unsupported"));
        return;
    }
    try {
        if (!barcodeDetector) {
            barcodeDetector = new BarcodeDetector({
                formats: ["ean_13", "ean_8", "upc_a", "upc_e", "code_128"],
            });
        }
        scanStream = await navigator.mediaDevices.getUserMedia({
            video: { facingMode: "environment" },
        });
        video.srcObject = scanStream;
        await video.play();
        scanActive = true;
        setScanStatus(scanMsg("Scanning"));
        requestAnimationFrame(detectLoop);
    } catch (_) {
        stopCamera();
        setScanStatus(scanMsg("Unsupported"));
    }
}

async function detectLoop() {
    const video = document.getElementById("scan-video");
    if (!scanActive || !barcodeDetector || !video) return;
    try {
        const codes = await barcodeDetector.detect(video);
        const raw = codes && codes.length ? codes[0].rawValue : null;
        if (raw) {
            stopCamera();
            lookupBarcode(raw);
            return;
        }
    } catch (_) {
        /* Detection can throw transiently before the first frame; keep going. */
    }
    if (scanActive) requestAnimationFrame(detectLoop);
}

function stopCamera() {
    scanActive = false;
    if (scanStream) {
        scanStream.getTracks().forEach((track) => track.stop());
        scanStream = null;
    }
    const video = document.getElementById("scan-video");
    if (video) video.srcObject = null;
}

// Resume the camera a moment after a failed lookup so the user can rescan,
// unless they've since closed the panel.
function resumeScanning() {
    if (!("BarcodeDetector" in window)) return;
    setTimeout(() => {
        if (scannerIsOpen() && !scanActive) startCamera();
    }, 1500);
}

function submitManualBarcode(event) {
    event.preventDefault();
    const input = document.getElementById("scan-input");
    const code = input ? input.value.trim() : "";
    if (code) {
        stopCamera();
        lookupBarcode(code);
    }
}

async function lookupBarcode(barcode) {
    setScanStatus(scanMsg("Looking"));
    try {
        const resp = await fetch("/products/" + encodeURIComponent(barcode), {
            headers: { Accept: "application/json" },
        });
        if (resp.status === 404 || resp.status === 400) {
            setScanStatus(scanMsg("Notfound"));
            resumeScanning();
            return;
        }
        if (!resp.ok) {
            setScanStatus(scanMsg("Error"));
            resumeScanning();
            return;
        }
        applyProduct(await resp.json());
    } catch (_) {
        setScanStatus(scanMsg("Error"));
        resumeScanning();
    }
}

// Split an Open Food Facts pack size (e.g. "500 g", "1,5 L") into a numeric
// amount and a unit. Falls back to putting the whole string in the unit when it
// doesn't start with a number.
function parsePackSize(raw) {
    if (!raw) return { amount: "", unit: "" };
    const text = String(raw).trim();
    const match = text.match(/^(\d+(?:[.,]\d+)?)\s*(.*)$/);
    if (match) return { amount: match[1].replace(",", "."), unit: match[2].trim() };
    return { amount: "", unit: text };
}

// Prefill the add sheet from a resolved product and hand off to it.
function applyProduct(product) {
    closeScanner();
    openSheet();
    const name = document.getElementById("add-name");
    const qty = document.querySelector("#add-form .qty-input");
    const unit = document.querySelector("#add-form .unit-input");
    const category = document.getElementById("add-category");
    const size = parsePackSize(product.quantity);
    if (name) name.value = product.name || "";
    if (qty) qty.value = size.amount || "1";
    if (unit && size.unit) unit.value = size.unit;
    if (category) category.value = product.category || "";
    if (name) setTimeout(() => name.focus(), 60);
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

// Meal-ideas screen: selecting a meal-type chip records it in the hidden
// `meal_type` input that htmx sends with the "Suggest meals" request.
function pickMeal(btn) {
    const group = btn.closest("[data-meal-group]");
    if (group) {
        group.querySelectorAll(".chip").forEach((c) => c.classList.remove("active"));
    }
    btn.classList.add("active");
    const hidden = document.getElementById("meal-type");
    if (hidden) hidden.value = btn.dataset.meal;
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
