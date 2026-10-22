// Test of the wrapper exe in a fresh WebView2 profile, window off-screen: offline (all requests local), new project +
// brush stroke in History, Ctrl+wheel and touchpad pinch zoom the canvas (not the page), a dropped file opens, Print's
// popup opens, downloads are allowed, Ctrl+R stays in the page, the save/open pickers exist.
// Usage: npm test   (or: node scripts/test-app.mjs). Needs a built exe (cargo build --release in src-tauri).
import { spawn, spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { createRequire } from "node:module";
const require = createRequire(import.meta.url);
const { chromium } = require("playwright-core");
const root = path.join(path.dirname(fileURLToPath(import.meta.url)), "../src-tauri");
const port = 9351;
const profile = path.join(root, "target/wv2-test");
const dl = path.join(root, "target/test-dl");
fs.rmSync(profile, { recursive: true, force: true });
fs.rmSync(dl, { recursive: true, force: true });
fs.mkdirSync(dl, { recursive: true });
const p = spawn(path.join(root, "target/release/photopea-offline.exe"), [], {
  env: { ...process.env, WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS: `--remote-debugging-port=${port}`, WEBVIEW2_USER_DATA_FOLDER: profile, PHOTOPEA_TEST_OFFSCREEN: "1" },
  stdio: "ignore",
});
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
const results = [];
const check = (name, pass, detail = "") => { results.push({ name, pass }); console.log(`${pass ? "ok  " : "FAIL"} ${name}${detail ? "  (" + detail + ")" : ""}`); };
try {
  let browser = null;
  for (let i = 0; i < 100 && !browser; i++) { try { browser = await chromium.connectOverCDP(`http://127.0.0.1:${port}`); } catch { await sleep(300); } }
  let page = null;
  for (let i = 0; i < 100 && !page; i++) { for (const c of browser.contexts()) for (const pg of c.pages()) if (pg.url().includes("8887")) page = pg; if (!page) await sleep(300); }
  const errs = [];
  page.on("pageerror", (e) => errs.push(String(e).slice(0, 200)));
  const np = page.getByText("New Project", { exact: true }).first();
  await np.waitFor({ timeout: 60000 });
  await sleep(1000);
  await page.evaluate(() => { window.__marker = 1; });
  await np.click();
  await sleep(1500);
  await page.locator("button", { hasText: /^Create$/ }).first().click();
  await sleep(2000);
  const box = await page.evaluate(() => { let best = null; for (const c of document.querySelectorAll("canvas")) { const b = c.getBoundingClientRect(); if (!best || b.width * b.height > best.w * best.h) best = { x: b.left, y: b.top, w: b.width, h: b.height }; } return best; });
  const cx = box.x + box.w / 2, cy = box.y + box.h / 2;
  const leafTexts = () => page.evaluate(() => [...document.querySelectorAll("div, span, button")].filter((e) => e.children.length == 0 && e.getBoundingClientRect().width > 0).map((e) => e.textContent.trim()));

  // brush stroke
  await page.keyboard.press("b");
  await sleep(300);
  await page.mouse.move(cx - 100, cy);
  await page.mouse.down();
  for (let k = 0; k <= 20; k++) { await page.mouse.move(cx - 100 + k * 10, cy + Math.sin(k / 3) * 30); await sleep(15); }
  await page.mouse.up();
  await sleep(800);
  check("brush stroke appears in History", (await leafTexts()).some((t) => /^Brush/.test(t)));
  // history keeps 50 steps: after 55 more strokes, "Open" is gone and 50 strokes remain
  for (let k = 0; k < 55; k++) {
    await page.mouse.move(cx - 200, cy - 150 + k * 5);
    await page.mouse.down();
    await page.mouse.move(cx - 100, cy - 150 + k * 5, { steps: 3 });
    await page.mouse.up();
    await sleep(60);
  }
  await sleep(800);
  const hist = await page.evaluate(() => [...document.querySelectorAll("div, span")].filter((e) => e.children.length == 0 && /^(Open|Brush Tool)$/.test(e.textContent.trim())).map((e) => e.textContent.trim()));
  check("History keeps 50 steps", hist.length == 50 && !hist.includes("Open"), `${hist.length} entries, Open ${hist.includes("Open") ? "kept" : "dropped"}`);

  // zoom: the canvas must change while the page itself stays at scale 1
  const pageZoom = () => page.evaluate(() => [window.visualViewport.scale, window.devicePixelRatio, window.innerWidth]);
  const z0 = await pageZoom();
  const cdp = await page.context().newCDPSession(page);
  let s0 = await page.screenshot();
  await page.mouse.move(cx, cy);
  await cdp.send("Input.dispatchMouseEvent", { type: "mouseWheel", x: cx, y: cy, deltaX: 0, deltaY: -240, modifiers: 2 });
  await sleep(800);
  let s1 = await page.screenshot();
  check("Ctrl+wheel zooms the canvas, not the page", !s0.equals(s1) && JSON.stringify(await pageZoom()) == JSON.stringify(z0), JSON.stringify(await pageZoom()));
  s0 = s1;
  await cdp.send("Input.synthesizePinchGesture", { x: cx, y: cy, scaleFactor: 0.5, gestureSourceType: "mouse" });
  await sleep(800);
  s1 = await page.screenshot();
  check("touchpad pinch zooms the canvas, not the page", !s0.equals(s1) && JSON.stringify(await pageZoom()) == JSON.stringify(z0), JSON.stringify(await pageZoom()));

  // drop a PNG on the canvas (renderer-level drag events; the OS-level path is what disable_drag_drop_handler frees)
  const png = path.join(root, "icons/128x128.png");
  for (const type of ["dragEnter", "dragOver", "drop"]) await cdp.send("Input.dispatchDragEvent", { type, x: cx, y: cy, data: { items: [], files: [png], dragOperationsMask: 1 } });
  await sleep(2500);
  // dropped on an open document, Photopea places the file as a layer
  check("dropped file is placed", (await leafTexts()).some((t) => /^Placing Smart Object$/.test(t)) && (await leafTexts()).includes("128x128"));

  // Ctrl+R toggles rulers inside Photopea and must not reload the page
  await page.keyboard.press("Escape");
  await page.keyboard.press("Control+r");
  await sleep(800);
  check("Ctrl+R stays in the page (no reload)", (await page.evaluate(() => window.__marker).catch(() => 0)) === 1);

  // Print's popup (window.open() + document.write); the window is closed right away
  const popup = await page.evaluate(() => { const w = window.open(); if (!w) return "blocked"; try { w.document.write("<p>x</p>"); const t = w.document.body.textContent; w.close(); return t; } catch (e) { return String(e); } });
  check("Print popup opens (window.open())", popup == "x", popup);
  const ext = await page.evaluate(() => { const w = window.open("https://example.com/"); if (w) w.close(); return w ? "opened" : "blocked"; });
  check("external popup blocked", ext == "blocked", ext);

  // downloads (exports use <a download>): sent to a test folder for this run
  const bcdp = await browser.newBrowserCDPSession();
  await bcdp.send("Browser.setDownloadBehavior", { behavior: "allow", downloadPath: dl.replace(/\//g, "\\"), eventsEnabled: true });
  const done = new Promise((r) => { bcdp.on("Browser.downloadProgress", (e) => { if (e.state != "inProgress") r(e.state); }); setTimeout(() => r("timeout"), 10000); });
  await page.evaluate(() => { const a = document.createElement("a"); a.href = URL.createObjectURL(new Blob([new Uint8Array([1, 2, 3])])); a.download = "test.bin"; document.body.appendChild(a); a.click(); a.remove(); });
  const st = await done;
  await sleep(300);
  check("download completes", st == "completed" && fs.existsSync(path.join(dl, "test.bin")), st);

  check("save/open pickers available (Save as PSD, File > Open)", await page.evaluate(() => typeof window.showSaveFilePicker == "function" && typeof window.showOpenFilePicker == "function"));
  check("clipboard API present", await page.evaluate(() => !!(navigator.clipboard && navigator.clipboard.read && window.ClipboardItem)));

  // offline
  const hosts = new Set();
  for (const u of await page.evaluate(() => performance.getEntriesByType("resource").map((e) => e.name).concat([location.href]))) { try { const x = new URL(u); hosts.add(x.host || x.protocol); } catch {} }
  const external = [...hosts].filter((h) => h != "127.0.0.1:8887" && !/^(data|blob):$/.test(h));
  check("all requests local", external.length == 0, external.join(" "));
  await page.screenshot({ path: path.join(root, "target/test_app.png") });
  console.log("page errors:", errs.length, errs.slice(0, 3));
  await browser.close();
} catch (e) {
  results.push({ name: "exception", pass: false });
  console.log("EXCEPTION", e);
} finally {
  spawnSync("taskkill", ["/pid", String(p.pid), "/T", "/F"], { stdio: "ignore" });
}
console.log(results.every((r) => r.pass) ? "PASS" : "FAIL");
