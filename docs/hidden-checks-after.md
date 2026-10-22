# Hidden checks in the Photopea bundle

A record of every expiry, licence check, kill switch and anti-tamper trap found in Photopea's main code bundle, with where each one is and what it does.

- **Current file:** `www/code/pp/pp1792573386.js`, the patched mirror that the browser version and the exe both load.
- **Original file:** `upstream/pp1792573386.js`, Photopea's unmodified bundle.
- **Patch script:** `scripts/patch-bundle.js` rebuilds the current file from the original. "Op" names below refer to its edits.

Line numbers are for the files as they were on 2026-10-22. The bundle's file name is itself a timestamp: `1792573386` is late October 2026.

## Summary

| # | What | Effect | Status in the current file |
|---|------|--------|----------------------------|
| 1 | Document expiry | From **31 May 2027 14:47 UTC**, no document can be opened or created: New Project, Open and drag-drop silently do nothing | **Removed** (round 7, op `expiry.open`) |
| 2 | Developer-mode port check | `:8887` in the address skips checks 3–5 | Present, harmless: its only remaining use is the local SoundFont path in Jampea. Since round 7 removed 3–5, the mirror no longer depends on it |
| 3 | Serial-key licence | Other hosts need a paid, expiring `serial_key` | **Removed** (round 7, ops `licence.azp`, `licence.hash`, `licence.clock`) |
| 4 | Input lockout | Unlicensed: every click and key press is ignored | **Removed** (round 7, ops `lockout.set`, `lockout.enforce`) |
| 5 | Delayed sabotage | Unlicensed: the UI breaks 9.5–19.5 s after start | **Removed** (round 7, ops `sabotage.timer`, `sabotage.fn`) |
| 6 | Random "coin flips" | About 1 in 10 million: garbled strings, crash, or an expiry in 2005 | **Removed** (ops `ob.*`) |
| 7 | Ad-blocker / "modified code" detector | Refused to open documents | **Removed** (op `ads.open.check`, `FN ads.dW.On`) |
| 8 | Anti-mirror page | On listed sites, replaced the whole page with a "go to photopea.com" notice | **Removed** (op `net.antimirror.page`) |
| 9 | Promo list expiry | Hid expired promo entries in the sidebar | **Removed** (op `side.pd.expiry`); harmless |
| 10 | Time-of-day restrictions for free accounts | Sketch/XD → PSD only in the first 15 min of every 2 hours; saving inside Figma/Framer blocked on odd minutes | **Removed** (round 5); the first was already off on `:8887` |

Things that look similar but are harmless (verified, left alone):
- the April 1st "cucumber" logo (current line 13885)
- the holiday snow on the home screen, 21 Dec–9 Jan (current line 13871)
- the export date written into EXIF data (current line 12647)

## How the code hides things

Photopea scrambles sensitive names so they can't be found by searching for them.

**String decoders**, in the current file at lines 15–21 (`iK.kF`, `iK.wT`) and 12680–12682 (`H.kF`, `H.wT`):
```js
H.kF=function(h){var R="";for(var l=0;l<h.length;l++){var a=h.charCodeAt(l);if(a<32||126<a)throw"e"; R+=String.fromCharCode(32+(a-32+17+3*l)%95)}return R};
H.wT=function(h){var R=""; for(var l=0;l<h.length;l++){var a=h.charCodeAt(l);a-=32;a+=95e4;a-=17+3*l;a=a%95;R+=String.fromCharCode(32+a)}return R};
```
Each character is shifted by an amount that grows with its position. For example, `H.kF("3M]K")` = `"Date"`, `H.wT("t$&!'31")` = `"confirm"`, and `iK.kF(")$!}y")` = `":8887"`. Constants like `iK.fi`, `iK.A4` and `iK.Ui` are pre-decoded strings (`"location"`, `"href"`, `"document"` …), defined at the top of the file (lines ~19–35).

**Global object**, current lines 12679 and 12705: `H.Bo` is set to `document` first and then to `window`, so `H.Bo["location"]` reads as `window.location`.

**Disguised numbers:** dates are written as arithmetic with π or bit shifts, and delays as powers of π. Searching for a date or a timestamp therefore finds nothing.

In the snippets below, **Raw** is the exact text in the file and **Decoded** is the same code with the scrambled strings replaced.

## When the checks run

`kU` is the main application object, created once when the page loads. All lines below are in the current file `www/code/pp/pp1792573386.js`, and every snippet is copied exactly from it.

| When | Line | Exact code | Checks involved |
|---|---|---|---|
| Page load: the `kU` constructor schedules startup | 17457 | `dw.II(this.aDF.bind(this));` | — |
| …where `dw.II` is a 1 ms timer | 11 | `dw.II=function(h){setTimeout(h,1)};` | — |
| 1 ms after load: `kU.prototype.aDF` starts the sabotage timer | 17462 | `H.Bo[iK.a2y](this.a2f.bind(this),Math.pow(Math.PI,8)+H.Bo[iK.anc][iK.aBP]()*1e4);` | 5 |
| …and sets the input-lockout flag | 17464 | `if(!this.k2()){var e=H.azp();if(e==0)this.aHv=!0;if(e==2)this.o.Iq=!1;` | 2 → 3 → 4 |
| Every user action: the base class registers the central action handler | 17421 | `this.Q(O.E.b,this.ayh,this);` | — |
| …whose first statement is the lockout | 17520 | `kU.prototype.ayh=function(h){if(this.aHv){h.data=0;` | 4 |
| Every document opened or created: `kU.prototype.IP` adds it to the editor | 17696 | `kU.prototype.IP=function(h){var R=this.o,a=this.i1.indexOf(h);` | 1 |
| 9.5–19.5 s after load | 17795 | `kU.prototype.a2f=function(h){if(this.k2())return 1;` | 2 → 3 → 5 |
| Every window resize: the listener | 17419 | `window.addEventListener("resize",this.wc.bind(this),!1);` | — |
| …which calls the layout function `C8`, the one 5 replaces | 17423–17424 | `bL.prototype.wc=function(h){var R=window.innerWidth,a=window.innerHeight;`<br>`this.C8(R,a)};` | 5 |

(The table and the snippets below describe the checks as they were; since round 7 the code they point to no longer exists in the current file — the original line references still apply to `upstream/pp1792573386.js`.)

---

## 1. Document expiry (removed)

It has two parts: a hidden clock and the check that uses it.

### 1a. The clock: `iK.Pm`

| File | Line |
|------|------|
| current | 17 |
| original | 50 |

Current, raw:
```js
iK.Pm=function(){var h=H.Bo[H.kF("3M]K")];h=h[H.kF("][`")];return Math.floor(h()/1e3)};
```
Decoded:
```js
iK.Pm=function(){var h=H.Bo["Date"];h=h["now"];return Math.floor(h()/1e3)};   // = current time in seconds
```
Original, before op `ob.pm` removed the coin flips:
```js
iK.Pm=function(){var h=H.Bo[H.kF("3M"+(H.Ob(26)?"]K":"fa"))];h=h[H.kF("]["+(!H.Ob(28)?",":"`"))];return Math.floor(h()/1e3)};
```
`iK.Pm` is used in exactly two places: the expiry (1b) and the serial-key check (3).

### 1b. The check: runs every time a document is opened or created

| File | Lines |
|------|-------|
| current | 17698–17699 |
| original | 18427–18428 |

Current:
```js
var U=h.Z.length,n=iK.Pm(),f=Math.PI*(8e3<<14)+20*7e7;
if(n>f){var U=h.Z.length;h.Z=[];return U}
```
Original:
```js
var U=h.Z.length,n=iK.Pm(),f=Math.PI*(8e3<<14)+(H.Ob(31)?20:10)*7e7;
if(n>f){var U=h.Z.length;h.Z=[];return U}
```
**How it works, step by step:**

1. These lines are inside `kU.prototype.IP`, which starts at current line 17696:
   ```js
   kU.prototype.IP=function(h){var R=this.o,a=this.i1.indexOf(h);
   ```
   Every route that puts a document into the editor goes through `IP`. Its four call sites in the current file:

   | Line | Exact code | Route |
   |---|---|---|
   | 17486 | `n.name="song.mid";ic.RT.Pc(U,n);this.IP(n)}` | a song passed in the URL (`#m…`) |
   | 17565 | `if(s.d6){var bN=s.d6;bN.R3={a7u:V.add.SoLd.Idnt.v,uv:y};this.IP(bN)}` | opening a smart object's contents |
   | 17577 | `else this.IP(o)}` | a file loaded from a URL |
   | 17582 | `if(n==O.m.lf){if(!this.K8.mT()\|\|h.G6)this.IP(h.data.cm)}` | the "load document" message, used by New Project, File › Open, drag-drop and others |

   For example, the New Project dialog (`iV`, current line 15241) sends that message when you press Create:
   ```js
   var a=new b8(O.E.b,!0);a.data={S:O.m.lf,cm:R};this.D(a);this.close();
   ```
2. `IP` first does some bookkeeping on the document `h`: if it's already open it switches to its tab, otherwise it records its source and open time. Then it reaches the check (lines 17698–17699, quoted above).
3. `n` is the current time in seconds from the hidden clock (1a).
4. `f` is the cutoff. 8000 << 14 is 131,072,000, so `f` = π × 131,072,000 + 20 × 70,000,000 ≈ **1,811,774,832** seconds since 1970 = **2027-05-31 14:47:12 UTC**.
5. If `n > f`, it empties the document's layer list (`h.Z=[]`) and **returns before the document is added to the editor**. The line that adds the tab comes later in `IP`, at current line 17701, so it never runs:
   ```js
   this.i1.push(h);this.cD.j2(new kf(h));
   ```
   The return value (the old layer count) is ignored by every caller.

**What you see after the cutoff**, verified by running the current file in a separate test browser with its clock set to 1 June 2027:
- New Project › Create opens nothing; the home screen stays.
- File › Open of a PNG opens nothing.
- There's no message; one script error (`TypeError: Cannot read properties of undefined (reading 'm')`) appears in the console because later code expects the document to exist.
- With the clock at 22 October 2026, the same steps open both documents normally.

**Your files are not at risk:** nothing ever opens, so nothing can be saved over a file. The editor simply becomes unusable for documents.

**The original's coin flip:** `H.Ob(31)` picked `10` instead of `20` about once in tens of millions of document opens, which moved the cutoff to **2005-03-25**, so that one open would silently fail. Op `ob.expiry` fixed it to `20`.

**Status:** removed (round 7, op `expiry.open`). The cutoff block in `IP` is gone, so documents keep opening after 31 May 2027.

---

## 2. Developer-mode port check (present, harmless)

| File | Line |
|------|------|
| current | 17500 |
| original | 18189 |

Current, raw:
```js
kU.prototype.k2=function(){return 4<H.Bo[iK.fi][iK.A4][iK.aAM](iK.kF(")$!}y"))};
```
Decoded:
```js
kU.prototype.k2=function(){return 4<H.Bo["location"]["href"]["indexOf"](":8887")};
```
Original, which also treated being embedded in Figma or Framer as developer mode (removed in round 5, op `embed.k2`):
```js
kU.prototype.k2=function(){if(dW.bP())return!0;return 4<H.Bo[iK.fi][iK.A4][iK.aAM](iK.kF(")$!}y"))};
```
It returns true when the address contains `:8887`. Checks 4 and 5 used to call `k2()` first and do nothing when it's true; since round 7 removed them, `k2()` no longer gates anything. **Its only remaining call is the Jampea SoundFont path** (below), which is harmless — **the mirror no longer depends on port 8887.**

Every call to `k2()`:

| Current line | Original line | Code | Purpose | Status |
|---|---|---|---|---|
| 17456 | 18120 | `if(gE==2)this.rm.bg({url:this.k2()?"rsrc/sf2/MagicSF.zip":dw.n7("rsrc/sf2/MagicSF.zip")})` | Jampea only: local SoundFont path in developer mode | harmless, kept |
| — | 18122 | `if(this.k2())return;if(gE==0){dw.event("web_PPea",.01);…` | skip referrer/platform analytics in developer mode | removed with the analytics (`stats.referrer`) |
| 17464 | 18135 | `if(!this.k2()){var e=H.azp();if(e==0)this.aHv=!0;…` | input lockout, check 4 | removed (`lockout.set`) |
| — | 18191 | `X=!this._J\|\|this.k2()\|\|!this.o.Iq\|\|dw.he()\|\|H.zR()=="vectorpea.com"\|\|…` | ad sidebar: no ads in developer mode / premium | removed with the ad layout (`ads.layout`) |
| — | 18360 | `if(this.k2())e=!1;` in `kU.prototype.aMh` | Sketch/XD time limit off in developer mode (item 10) | removed (`premium.aMh`) |
| 17795 | 18525 | `kU.prototype.a2f=function(h){if(this.k2())return 1;…` | delayed sabotage, check 5 | kept |

(`ak2` at current lines 15694 and 15702 is an unrelated function, `jZ.mY.prototype.ak2`, whose name merely contains "k2".)

---

## 3. Serial-key licence (removed)

Three functions make it up.

### 3a. `H.azp`: licence status

Returns 1 on photopea.com, 2 with a valid key, and 0 otherwise.

| File | Lines |
|------|-------|
| current | 12655–12659 |
| original | 12825–12829 |

Raw:
```js
H.azp=function(){var h=H.zR();if(h=="")return 0;
if(h!=H.kF("_TXZRPB;d7@;")&&h!=iK.kF("eQLZRRM?8a4=8")&&h!=iK.kF("YMVVHAj=FA")){var R=H.Bo[iK.fi][iK.A4],a=R.indexOf(String.fromCharCode(35)),e;
if(a==-1)return 0;try{e=JSON.parse(H.Bo[iK.a6H](R.slice(a+1)))}catch(lr){return 0}var U=e[H.kF("bQ[ODL<E<M")];
if(U==null||U.length<<2!=64)return 0;var n=iK.Pm(),f=parseInt(U.slice(3*4).split("").reverse().join(""),16)<<16;
if(f<n||U!=iK.aKz(f,h))return 0;return 2}return 1};
```
Decoded:
```js
H.azp=function(){var h=H.zR();if(h=="")return 0;
if(h!="photopea.com"&&h!="vectorpea.com"&&h!="jampea.com"){var R=H.Bo["location"]["href"],a=R.indexOf(String.fromCharCode(35)),e;
if(a==-1)return 0;try{e=JSON.parse(H.Bo["decodeURIComponent"](R.slice(a+1)))}catch(lr){return 0}var U=e["serial_key"];
if(U==null||U.length<<2!=64)return 0;var n=iK.Pm(),f=parseInt(U.slice(3*4).split("").reverse().join(""),16)<<16;
if(f<n||U!=iK.aKz(f,h))return 0;return 2}return 1};
```
- On any other domain, the address must contain `#{"serial_key":"<16 chars>"}`.
- The last 4 characters, reversed and read as hex and shifted `<<16`, are the key's **expiry time**. If that time has passed (`f<n`), the key is invalid.
- The key must also match a hash of that expiry and the domain (3c).

### 3b. `H.zR`: the site's domain

| File | Lines |
|------|-------|
| current | 12659–12660 |
| original | 12829–12830 |

Decoded:
```js
H.zR=function(){var h=H.Bo["location"]["hostname"],R=String.fromCharCode(46),a=h.split(R);
if(a.length<2)return"";var e=a.pop();e=a.pop()+R+e;return e};
```
It returns the last two parts of the hostname (e.g. `photopea.com`). For `127.0.0.1` that is `"0.1"`.

Every call to `H.zR()`:

| Current line | Original line | Where | Status |
|---|---|---|---|
| 12655 | 12825 | `H.azp` (3a) | kept |
| — | 12830–12834 | `H.gl` (AI permission: `(R=="photopea.com"\|\|R=="0.1")&&…`) and `H.aK$`, which sent `"…/papi/event.php?g=2&id=d_"+H.zR()` to photopea.com | both removed (`ai.gl`, `net.report.domain`) |
| — | 18191 (×2) | ad sidebar: `H.zR()=="vectorpea.com"\|\|H.zR()=="jampea.com"` | removed (`ads.layout`) |

### 3c. `iK.aKz`: key hash

| File | Lines |
|------|-------|
| current | 15–16 |
| original | 48–49 |

Decoded:
```js
iK.aKz=function(h,R){if(typeof R!="string")throw"e";var a=H.Bo["sha1"];
if(a==null)return"XYP";var e=h.toString(16).slice(0,4),U=e+R+"aQYRDCB";return a(U).slice(5,17)+e.split("").reverse().join("")};
```
The key is SHA-1 of (expiry + domain + `"aQYRDCB"`), with the reversed expiry appended.

**How a key is checked, step by step:**

1. `H.zR()` takes the domain from the address bar: the last two parts of the hostname.
2. On `photopea.com`, `vectorpea.com` or `jampea.com`, `azp` returns **1** (official site) and nothing else is checked.
3. Anywhere else it looks for `#` in the address. `String.fromCharCode(35)` is `#`, written that way so a search for `"#"` won't find it. It parses everything after the `#` as JSON and reads `serial_key`. No `#`, invalid JSON or no key → **0**.
4. The key must be 16 characters (`U.length<<2!=64` is a disguised `length != 16`).
5. **Expiry:** the last 4 characters (`U.slice(12)`) are reversed and read as a hex number, then shifted left 16 bits (× 65,536). That gives the expiry time in seconds, with a resolution of about 18 hours. If it's earlier than now (`f<n`) → **0**.
6. **Signature:** `iK.aKz(f, domain)` rebuilds the key: `hex4` is the first 4 hex digits of `f` (the same 4 digits, unreversed), and the key is `sha1(hex4 + domain + "aQYRDCB")` characters 5–16 followed by `hex4` reversed. If that doesn't equal the given key → **0**. `H.Bo["sha1"]` is `window.sha1`, a SHA-1 library function defined in the extension bundle (`www/code/ext/ext1791885294.js` line 3544: `h.sha1=y`). If it's missing, the expected key becomes `"XYP"`, which never matches.
7. Otherwise → **2** (valid licence). The caller then also sets `this.o.Iq=!1`, which turns ads off for licensed copies.

**Status:** removed (round 7, ops `licence.azp` = `H.azp`, `licence.hash` = `iK.aKz`, `licence.clock` = `iK.Pm`). All three functions were deleted once their callers were gone.

---

## 4. Input lockout (removed)

`aHv` appears exactly three times:

| Current line | Original line | Code | Role |
|---|---|---|---|
| 17444 | 18107 | `…EA:{},Iq:!0,aHv:!1,compact:!1,…` | default value (off) in the app's settings object |
| 17464 | 18135 | `if(e==0)this.aHv=!0;` | set at startup (4a) |
| 17520 | 18217 | `if(this.aHv){h.data=0;return h.k}` | enforced on every action (4b) |

### 4a. The flag is set at startup, inside `kU.prototype.aDF`

| File | Lines |
|------|-------|
| current | 17464–17465 |
| original | 18135–18136 |

Current:
```js
if(!this.k2()){var e=H.azp();if(e==0)this.aHv=!0;if(e==2)this.o.Iq=!1;
}
```
Original, which also reported unlicensed domains to Photopea about once in 500 starts (removed in an earlier round, ops `net.report.domain` and `net.report.call`):
```js
if(!this.k2()){var e=H.azp();if(e==0)this.aHv=!0;if(e==2)this.o.Iq=!1;
if(e!=1&&Math.random()<.002)H.aK$()}
```

### 4b. The flag is enforced on every user action, at the top of `kU.prototype.ayh`

| File | Lines |
|------|-------|
| current | 17520–17521 |
| original | 18217–18218 |

```js
kU.prototype.ayh=function(h){if(this.aHv){h.data=0;
return h.k}
```
With no `:8887` and no licence, `aHv` is true, and every menu command, tool action and key press is thrown away silently.

**How it works, step by step:**

1. One millisecond after load, `aDF` runs: `if(!this.k2())…`. On `:8887` the whole block is skipped and `aHv` keeps its default `false`.
2. Otherwise it calls the licence check: `e=H.azp()`. Result 0 (no valid licence, not photopea.com) → `this.aHv=!0`.
3. Photopea's UI sends every command as a message (a `b8` object with a `data` payload) up to the application object. All of them arrive in `kU.prototype.ayh`, registered as the message handler in the base class at current line 17421:
   ```js
   this.Q(O.E.b,this.ayh,this);
   ```
4. The first statement of `ayh` is the lockout. When `aHv` is true it blanks the message (`h.data=0`) and returns before any handling.
5. **Result:** the page draws normally, but buttons, menus and shortcuts do nothing, and nothing is reported.

**Verified:** loading the current file in a separate test browser without `:8887` in the address, the home screen appeared, but clicking New Project did not open its dialog.

**Status:** removed (round 7, ops `lockout.set`, `lockout.enforce`). The startup block that set `aHv` and the first statement of `ayh` are gone. The inert default `aHv:!1` in the settings object is the only trace left.

---

## 5. Delayed sabotage (removed)

### 5a. The timer, at the start of `kU.prototype.aDF`

| File | Line |
|------|------|
| current | 17462 |
| original | 18133 |

Raw:
```js
H.Bo[iK.a2y](this.a2f.bind(this),Math.pow(Math.PI,8)+H.Bo[iK.anc][iK.aBP]()*1e4);
```
Decoded:
```js
H.Bo["setTimeout"](this.a2f.bind(this),Math.pow(Math.PI,8)+H.Bo["Math"]["random"]()*1e4);
```
π⁸ ≈ 9,488 ms, plus 0–10,000 ms at random, so it runs **9.5 to 19.5 seconds after start**. The variable delay makes the cause hard to connect to the effect.

### 5b. The payload: `kU.prototype.a2f`

| File | Lines |
|------|-------|
| current | 17795–17796 |
| original | 18525–18526 |

```js
kU.prototype.a2f=function(h){if(this.k2())return 1;
if(H.azp()==0){this.o={};this.C8=function(R,a){return Math.sqrt(R*a)}}return-1};
```
When unlicensed, it:
- replaces the app's settings object with an empty one (`this.o={}`)
- replaces the layout function `C8` with a dummy that returns `Math.sqrt`

The interface then breaks.

**How it works, step by step:**

1. At load, `aDF` schedules `a2f` with `setTimeout`, delayed by π⁸ ms (≈ 9,488) plus a random 0–10,000 ms. `setTimeout`, `Math` and `random` are all looked up through scrambled names, so none of those words appear near the call.
2. When it fires, `a2f` returns immediately on `:8887` (`if(this.k2())return 1`).
3. Otherwise it re-runs the licence check. If there's no licence (`H.azp()==0`):
   - **`this.o={}`**: `this.o` is the application's state object. It holds preferences, tool settings and panel state. The tools and dialogs receive it and read from it in over a hundred places. Replacing it with an empty object makes any later code that reads a setting throw an error. For example, current line 8212 (the Move tool) does
     ```js
     a=j.w(this.o.xd);
     ```
     and `this.o.xd` (the preferences) no longer exists.
   - **`this.C8=function(R,a){return Math.sqrt(R*a)}`**: `C8(width,height)` is the method that lays the interface out for a window size. It's called on every window resize: the listener at current line 17419, `window.addEventListener("resize",this.wc.bind(this),!1);`, calls `wc` at lines 17423–17424, `bL.prototype.wc=function(h){var R=window.innerWidth,a=window.innerHeight;` / `this.C8(R,a)};`. The replacement does no layout at all; it just returns a number.
4. **Why it's built this way:** someone who bypassed the input lockout (4) would still have the app fall apart 10–20 seconds later. The random delay keeps the failure from looking tied to anything specific.

**Verified:** in the same no-`:8887` test, after waiting past the window and resizing the browser from 1400 to 1000 pixels wide, the layout didn't re-fit. The History, Layers and other right-hand panels stayed at their old positions, off the edge of the window.

**Status:** removed (round 7, ops `sabotage.timer`, `sabotage.fn`). Neither the π⁸ timer nor the `a2f` payload exists any more.

---

## 6. Random "coin flips" (removed)

The function (identical in both copies):
```js
H.Ob=function(h){var R=0;for(var l=0;l<h;l++)R+=Math.random();return R/h<Math.PI/4};
iK.Ob=function(h){var R=0;for(var l=0;l<h;l++)R+=Math.random();return R/h<Math.PI/4};
```
It averages `h` random numbers and compares the result with π/4 (0.785). With `h` between 24 and 31, the average is almost never that high, so it returns true all but roughly 1 in 10⁷–10⁸ times. The flips look like meaningful branches to anyone reading the code, but the rare branch is always sabotage.

**How it works:**

1. The average of `h` random numbers between 0 and 1 clusters around 0.5, and more tightly the larger `h` is (standard deviation 1/√(12h), about 0.05–0.06 here).
2. Reaching 0.785 is more than five standard deviations above the average. By the normal approximation, the chance per call is about 1 in 1.5 million for `h=24`, 1 in 4 million for 26, 1 in 7 million for 27, 1 in 12 million for 28, and 1 in 55 million for 31. The real chances are smaller still.
3. So `H.Ob(n)` is "true, almost always", and each call site is written so the true branch is the normal behaviour.
4. **What the rare branch does in the decoders:** `if(!H.Ob(28))h+="-3"` adds two characters to the scrambled input, so the decoded name gets junk on the end (`"Date"` becomes something like `"DateXY"`). The property lookup then returns `undefined`, and the next call on it throws. These decoders are used for every scrambled name, so one bad flip can break whatever is being decoded at that moment.
5. **Why:** to make the code harder to analyse and to make deliberate failures look like random, unreproducible bugs.

Every site, all in the **original** file:

| Original line | Code | Rare outcome | patch-bundle.js op |
|---|---|---|---|
| 11 | `var R=H.Ob(24)?" ":"",a=H.Bo[H.wT("Uu,"+R)];` (in `dw.he`) | wrong property name in the premium check | removed with `dw.he` (`premium.dw.he`) |
| 50 | `iK.Pm=…H.kF("3M"+(H.Ob(26)?"]K":"fa"))…H.kF("]["+(!H.Ob(28)?",":"`"))…` | the clock reads a garbage property, so `Date.now` breaks | `ob.pm` |
| 51 | `iK.Ob=function(h){…}` | (definition) | `ob.ik` |
| 52 | `iK.kF=function(h){var R="";`<br>`if(!iK.Ob(28))h+="-3";…` | the decoder appends junk, so every decoded name is wrong | `ob.ik.kF` |
| 53 | `iK.wT=function(h){var R="";if(!iK.Ob(28))h+="-1";…` | same | `ob.ik.wT` |
| 12853 | `H.Bo=document;H.Ob=function(h){…}` | (definition) | `ob.h` |
| 12854 | `H.kF=function(h){var R="";if(!H.Ob(28))h+="-3";…` | the decoder appends junk | `ob.h.kF` |
| 12855 | `H.wT=function(h){var R="";if(!H.Ob(28))h+="-1";…` | same | `ob.h.wT` |
| 12879 | `if(H.Ob(27))H.Bo=window;else H.Bo=H.arK;` | `H.arK` is never defined, so the app crashes | `ob.window` (now `H.Bo=window;`, current line 12705) |
| 18427 | `f=Math.PI*(8e3<<14)+(H.Ob(31)?20:10)*7e7` | expiry becomes 2005, so the document opens blank at once | `ob.expiry` |

**Status:** all removed. No `Ob(` calls remain in the current file.

---

## 7. Ad-blocker / "modified code" detector (removed)

### 7a. The probe: `dW.On`, original lines 210–212

Decoded:
```js
dW.On=function(){var h=H.Bo["document"],R=h["body"],a=0,e="setAttribute",U="adsense",n="jsLoaded3178",f=h["createElement"]("div");
f[e]("id",U);f[e]("style","width:500px; height:500px;");R["appendChild"](f);f=h["getElementById"](U);if(f==null){a=2}else{if(!R["contains"](f)){a=3}else if(Math.round(f["getBoundingClientRect"]()["width"]-500)>5){a=4}R["removeChild"](f)}if(dW.Fg)a=5;
if(dW.afU)a=6;return a};
```
It inserts a 500×500 `<div id="adsense">` and checks whether something hid, removed or resized it, which is what ad blockers do. It returns non-zero if so.

### 7b. The check when a document opens, original lines 18426–18427 (just before the expiry check)

Raw:
```js
if(dW.On()!=0&&!this.aFr&&H.Bo[H.wT("t$&!'31")].toString().indexOf(H.wT(" u,$4&"))==-1){alert(H.wT("j$--=#66A@5EU`KQaFVjSih\\hpirt2,x&5&+#++A4<8SFOO_FUMQ|"));
return}
```
Decoded:
```js
if(dW.On()!=0&&!this.aFr&&H.Bo["confirm"].toString().indexOf("native")==-1){alert("Your browser (or an extension) is modifying our code.");
return}
```
If an ad blocker was detected **and** `window.confirm` had been replaced (its source no longer says "native code"), the document was **not opened**.

This is the only `.toString().indexOf(…)` tamper check in the bundle.

Every call to `dW.On()`, all in the original file:

| Original line | Where | What it did | Removed by |
|---|---|---|---|
| 219 | `dW.ahm` (line 218) | repeating "We use advertisement… Your device seems to be blocking ads" popup, every π×10⁴ ms (~31 s) | `ads.dW.ahm` |
| 221 | `dW.av9` (line 220) | load Google ads (`securepubads.g.doubleclick.net`) unless blocked | `ads.dW.av9` (earlier round) |
| 15833 | export dialog | `var G=dW.On()!=0;` then a "$1 a week to remove ads" nag | `acct.nag.export` |
| 18194 | ad sidebar layout | `if(Math.random()<.002){if(gE==0&&dW.On())dw.event("gota_0")}`: reports ad blocking | `ads.layout` |
| 18426 | document open (7b) | refuses to open the document | `ads.open.check` |

**Status:** removed in round 5 (ops `ads.open.check`, `ads.dW.On`). Its relatives went in the same round:
- the repeating ad-blocker popup `dW.ahm`
- the `window.___osw` width probe ("Something is changing our source code (Brave Shields, uBlock Origin …)")
- the `__csrf` "let us show you ads" block

---

## 8. Anti-mirror page (removed)

**Where:** the `kU` constructor (the main app object), original lines 18103–18106.

Decoded:
```js
var a="rwx~#.-(;.]6EF rx'|#1,6><@87f?NO …".split(" "),e=H.Bo["location"]["ancestorOrigins"];
if(e==null)e=[H.Bo["document"]["referrer"]];for(var U=0;U<e.length;U++){var n=e[U];for(var l=0;l<a.length;l++){var f=H.wT(a[l]);
if(n.indexOf(f)!=-1){var y=H.Bo["document"]["body"],X="<h1 style=\"color:white;background:black;font-size:2.4em\">Try (&#x041f&#x043e&#x043f&#x0440&#x043e&#x0431&#x0443&#x0439&#x0442&#x0435)<br/><a href=\"//www.photopea.com\" target=\"_blank\">www.Photopea.com</a></h1>The owner of this site should get a <a href=\"//www.photopea.com/api/accounts#distributors\">Distributor account</a>.";
H.x$(y);y["innerHTML"]=X;return}}}this.X=kU.a1s();
```
If Photopea was embedded in, or opened from, one of the listed sites, it wiped the page and showed a "Try www.Photopea.com … The owner of this site should get a Distributor account" notice instead of the editor.

The scrambled list decodes to: academiaqa.com, adobephotopea.com, photoeditoronline.net, fotoshoponline.ru, photoshoponline.ru, photopea.ru, online-photoshop.org, spiralbetty.ru, editor.0lik.ru, photo-edit.online, xn--80aaaghku3bgfcnmjf4c.xn--p1ai, xn--80asehdbaaaj4aw9c.xn--p1ai, goonxp.ru, goonxp.online, fangguo.com, photopea.fr.

**Status:** removed (op `net.antimirror.page`). It could never trigger for a local copy anyway.

Every use of `ancestorOrigins` (the list of sites a page is embedded in; `iK.aHz` once decoded), all in the original file:

| Original line | Where | Status |
|---|---|---|
| 68 | `iK.aHz=iK.kF(…)`: the decoded name | constant, still present (unused) |
| 208 | `dW.bP`: is Photopea embedded in Figma or Framer (item 10) | removed (`embed.dW.bP`) |
| 18103 | anti-mirror check (this item) | removed |
| 18192 | ad sidebar: `window.location.ancestorOrigins` when embedded | removed (`ads.layout`) |

---

## 9. Promo list expiry (removed, harmless)

**Where:** original line 14107, right after the promo list `fg.Pd` (original lines 14106ff.).
```js
for(var l=0;l<fg.Pd.length;l++)if(fg.Pd[l][5]*1e3<Date.now()){fg.Pd.splice(l,1);l--}
```
It dropped sidebar promotions (PeaGames, "AI Tools" …) whose end date had passed. The whole promo list was emptied in an earlier round (op `side.pd`), and this loop was removed with op `side.pd.expiry`.

---

## 10. Time-of-day restrictions for free accounts (removed)

These are premium restrictions rather than kill switches, but like the expiry they read the clock.

### 10a. Sketch/XD → PSD limit: `kU.prototype.aMh`

- **Definition:** original line 18359.
- **Called at:** original lines 18265, 18272 and 18275, before saving: `if(!this.aMh(y,h.data.a7N))return;`, `if(!this.aMh(y,y.$Z.jd.formats))return;`, `if(!this.aMh(y))return;`.

Decoded:
```js
kU.prototype.aMh=function(h,R){var a=!0,f=15;if(R){a=!1;for(var l=0;l<R.length;l++)if(R[l].startsWith("psd"))a=!0}var e=h.Nb()||h.R3&&h.R3.uv.Nb();if(this.k2())e=!1;var U=new H.Bo["Date"],n=(U["getHours"]()&1)*60+U["getMinutes"]();
if(e&&!dw.he()&&n>f&&a){var y=H.Bo["location"]["href"],X="To get money for our development, Sketch/XD to PSD conversion can be used only during the first 15 minutes every two hours.";X+="\n\n"+"Get a Premium account (for the unlimited access), or wait "+(-n+120)+" minutes.";if(y.indexOf("photopea.com")==-1)X=a8.get([0,11+6,3+2]);H.Bo._cwY(X);return!1}return!0};
```
For free accounts, saving a Sketch/XD file as PSD was blocked except during the first 15 minutes of every even hour. It was already off on `:8887` (`if(this.k2())e=!1`).

**Status:** removed (ops `premium.aMh`, `premium.aMh.call1`–`3`).

### 10b. Save throttle inside Figma/Framer

- **The check:** original line 18267, in the Save handler.
- **The embed test `dW.bP`:** original line 208.

```js
if(dW.bP()){var eY=new Date;
if(!dw.he()&&(eY.getMinutes()&1)==1){alert("Wait "+(60-eY.getSeconds())+" seconds or press \"Account\" (at the top) to go Premium.",5e3);
return}alert("File Saved.",3e3);if(!dw.he()){/* premium nag */}}
```
```js
dW.bP=function(){var h=H.Bo["location"]["ancestorOrigins"];if(h==null)h=[H.Bo["document"]["referrer"]];for(var l=0;l<h.length;l++){if(h[l].indexOf("figma.com")!=-1)return!0;if(h[l].indexOf("framercdn.com")!=-1)return!0;if(h[l].indexOf("framer.com")!=-1)return!0}return!1};
```
When Photopea ran inside Figma or Framer, free accounts couldn't save during odd minutes.

**Status:** removed (ops `acct.nag.saved`, `embed.dW.bP`, `embed.k2`).

---

## Completeness check

Every token behind these mechanisms was counted in both files, and each occurrence above is accounted for. Counts as of 2026-10-22, after round 7:

| Token | Original | Current | Notes |
|---|---|---|---|
| `iK.Pm()` (clock calls) | 2 | 0 | expiry + serial key both removed |
| `k2()` | 7 | 2 | 1 call (the Jampea SoundFont path) + the unrelated `ak2` |
| `H.azp()` | 2 | 0 | lockout + sabotage call sites |
| `H.zR()` | 5 | 0 | |
| `iK.aKz(` | 1 | 0 | |
| `aHv` | 3 | 1 | the inert default `aHv:!1` |
| `a2f` | 3 | 0 | the third original hit was inside an image-upload key string (`"59e11a2fac9b1ed"`) |
| `Ob(` / `Ob=function` | 9 / 2 | 0 / 0 | |
| `dW.On()` / `dW.On=` | 5 / 1 | 0 / 0 | |
| `.toString().indexOf(` | 1 | 0 | |
| `this.o={}` | 1 | 0 | sabotage payload, removed with `a2f` |
| `Math.pow(Math.PI,8)` | 1 | 0 | sabotage timer, removed with `a2f` |
| `fg.Pd[l][5]` | 1 | 0 | |
| `ancestorOrigins` / `iK.aHz` | 1 / 3 | 0 / 1 | |

The broader sweep (every clock read, every `.Z=[]`, every comparison with a large or π-based number, every computed timer delay, every `eval`/reload/redirect) found nothing beyond what is listed here and the three harmless date checks named at the top.

## Finding these again in a newer Photopea build

Minified names (`kU`, `H.azp`, `iK.Pm` …) change between builds, so search for shapes rather than names:

| To find | Search for |
|---|---|
| the clock | a function returning `Math.floor(h()/1e3)` where `h` is a decoded `Date`/`now` |
| the expiry | `Math.PI*(` next to `7e7` or `<<14`, and `.Z=[]` |
| the port check | `iK.kF(")$!}y")`, which decodes to `:8887` |
| the licence check | `String.fromCharCode(35)` and the decoded `serial_key` |
| the input lockout | `if(this.aHv)` at the top of the main action handler, and where `aHv` is set |
| the sabotage timer | `Math.pow(Math.PI,8)` |
| coin flips | `<Math.PI/4` (the `Ob` function) and every call to it |
| anti-tamper | `.toString().indexOf(` near a decoded `"native"` |

Decode the strings with the two formulas under [How the code hides things](#how-the-code-hides-things).
