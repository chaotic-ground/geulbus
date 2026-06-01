# 05 — Wayland/XKB and the keyval an IBus engine receives (Dvorak case)

Scope: GNOME 49 / mutter 49.5 / gnome-shell 49.6 / IBus 1.5.33 / Fedora 43.
Pure-zbus engine implementing `org.freedesktop.IBus.Engine`,
`ProcessKeyEvent(keyval, keycode, state)`.

Confidence legend: **[FACT]** = documented or directly observed on this box;
**[STRONG]** = source + multiple corroborating reports; **[INFER]** = reasoned
deduction. Anything that should be empirically confirmed before it drives an
irreversible design choice is flagged in §"Verify on the box".

---

## TL;DR (the crisp answers)

1. **Under a Dvorak XKB layout on GNOME Wayland, `ProcessKeyEvent` receives
   the DVORAK keysym, not the QWERTY-position keysym.** Press physical
   QWERTY-R (= Dvorak-P) and the engine gets `keyval = 0x70 ('p')`, plus the
   physical (evdev) `keycode` and the modifier `state`. mutter resolves
   keycode→keysym through the *current* xkb_state (active layout/group) before
   the event reaches IBus. **[FACT — gnome-shell `js/misc/inputMethod.js`
   passes `event.get_key_symbol()` as the IBus `keyval`; the clutter symbol is
   libxkbcommon's layout-dependent keysym.]**

2. **X11 and Wayland agree on what the engine receives** (the
   current-layout keysym). They differ only in *who* translates (X server vs
   mutter/libxkbcommon) and in whether the engine's `<layout>` is honored
   (X11 yes, Wayland no). **[STRONG]**

3. **The component `<layout>` element is effectively IGNORED on GNOME
   Wayland.** mutter does not switch the system XKB layout to an engine's
   declared `<layout>`. So `<layout>us</layout>` does **not** force `us`; a
   Dvorak user keeps Dvorak. This is the well-known "GNOME/Wayland ignores
   per-engine IBus layout" behavior. **[STRONG]**

4. **"Borrow the user's XKB layout" WORKS for shortcuts** (because the keyval
   is already in the user's layout and pass-through re-derives correctly),
   but has a Hangul-position caveat (§A). **`ForwardKeyEvent`-based keyval
   remapping does NOT work on Wayland** and cannot be made to: the client
   re-derives the character from the *keycode* via its own XKB state; your
   swapped keyval character is ignored. This is exactly your observed bug.
   **[STRONG]**

5. **Use `<layout>default</layout>`** (or omit `<layout>`) so the engine does
   not force a layout and inherits the user's current XKB layout. `default`
   is the documented "don't change / inherit current" sentinel. **[STRONG]**

---

## How the key actually travels (GNOME Wayland)

```
physical key
  -> kernel evdev keycode
  -> mutter (libinput + clutter): builds a ClutterKeyEvent.
     key symbol computed via libxkbcommon xkb_state_key_get_one_sym()
     against the CURRENT keymap + group (the user's active XKB layout,
     e.g. dvorak). => event.get_key_symbol() is the Dvorak keysym.
  -> gnome-shell IBus integration calls IBus
     process_key_event_async(keyval, keycode, state):
        keyval  = event.get_key_symbol()       (layout-DEPENDENT, already Dvorak)
        keycode = event.get_key_code() - 8     (evdev; layout-INDEPENDENT)
        state   = modifier mask
  -> ibus-daemon (run with --panel disable; gnome-shell owns the panel)
     routes to the active engine over D-Bus
  -> YOUR ENGINE: ProcessKeyEvent(keyval, keycode, state)
```

Ground-truth pieces:

- **keyval = the active-layout keysym. Now FACT, source-confirmed.**
  gnome-shell `js/misc/inputMethod.js` `vfunc_filter_key_event(event)` calls:

  ```js
  this._context.process_key_event_async(
      event.get_key_symbol(),        // keyval  -> the CLUTTER key symbol
      event.get_key_code() - 8,      // keycode -> evdev (XKB keycode minus 8)
      state, -1, this._cancellable, ...)
  ```

  `event.get_key_symbol()` is the ClutterKeyEvent symbol, which mutter computes
  via libxkbcommon `xkb_state_key_get_one_sym(state, keycode)` against the
  current keymap+group. By definition that returns the keysym *for the active
  layout and level*. So with Dvorak active, physical QWERTY-R → `XK_p` (0x70),
  and that is exactly what is handed to IBus as `keyval`. **[FACT — source
  line in gnome-shell inputMethod.js + libxkbcommon semantics]**

- **keycode = layout-independent physical key.** GDK/GTK docs: "keycode is
  only determined by the location of the key and is irrelevant of keyboard
  layout. Input methods that expect a certain keyboard layout should use
  keycode; otherwise keyval is sufficient." This is the only reliable handle
  if you want layout-independent jamo. **[FACT]** (GDK Keyboard Handling /
  keys.md.)
  The keycode you receive in ProcessKeyEvent is the **evdev** code: gnome-shell
  passes `event.get_key_code() - 8` ("Convert XKB keycodes to evcodes", per the
  inline comment). Classic X11 uses evdev+8. So a keycode→jamo table must use
  evdev numbering, not X11 keycodes. **[FACT — gnome-shell inputMethod.js]**

- **Wayland sends keycodes, not keysyms, to clients.** The compositor hands
  clients a keymap + keycodes; each client runs its *own* xkb_state to derive
  symbols/characters. (Wayland Book "XKB, briefly"; Gräßlin "How input works
  – Keyboard input".) This is the crux of why ForwardKeyEvent remap fails
  (§C). **[FACT]**

---

## §A. Does "borrow the user's XKB layout" work? (your option A/B)

**Yes for shortcuts; the only real decision is about Hangul jamo binding.**

If geulbus declares no layout override (`<layout>default</layout>` / omit)
and the user sets GNOME keyboard layout to Dvorak:

- **Shortcut / non-Hangul keys** (Ctrl/Alt/Super+key, and any key you don't
  consume): naturally Dvorak. The keyval you receive is already Dvorak, and
  when you return `false` from ProcessKeyEvent (or forward the *original*
  event), the app re-derives the character from the same keycode + Dvorak
  xkb_state and gets the Dvorak result. **No remapping code needed.**
  **[STRONG]** This is precisely why a pure `('xkb','us+dvorak')` GNOME source
  "just works" for you: no engine sits in the path mangling keyvals; the app
  sees Dvorak end-to-end.

- **THE CATCH — Hangul jamo positions follow Dvorak too, if you key jamo off
  keyval.** Your jamo map is keysym→jamo. With Dvorak active, physical
  QWERTY-R delivers keysym `p`, so a keyval-keyed table treats that physical
  key as the jamo you assigned to `p`, not to `r`. The jamo layout therefore
  rotates to Dvorak physical positions. A user who *chose* Dvorak may want
  exactly that; a typical 두벌식/세벌식 user expecting ㅂㅈㄷㄱ on the physical
  top row will not. **[STRONG]**

| Approach | Shortcuts | Hangul jamo positions |
|---|---|---|
| Borrow user XKB (no override), jamo keyed off **keyval** | Follow user layout. No code. | Rotate with user's Latin layout (Dvorak → jamo move). |
| Borrow user XKB, jamo keyed off **keycode** (fixed table) | Follow user layout. No code. | Fixed to physical keys regardless of Latin layout. |
| Force `<layout>us</layout>` + remap shortcuts via ForwardKeyEvent | **Broken on Wayland** (§C). | Fixed (only because you forced us — which also doesn't take effect on Wayland). |

Recommendation lands in **row 2** for a generic IME (§Recommendation).

---

## §B. Meaning of `<layout>` values on GNOME Wayland

Installed evidence captured directly on this box:

- `geulbus.xml`: `<layout>us</layout>` (current)
- `hangul.xml` (ibus-hangul 1.5.5): `<layout>kr</layout>` +
  `<layout_variant>kr104</layout_variant>`
- `chewing.xml`: `<layout>us</layout>`
- `simple.xml` xkb-engines: concrete layouts, e.g.
  `xkb:us:dvorak:eng` → `<layout>us</layout><layout_variant>dvorak</layout_variant>`

**Documented semantics (IBusEngineDesc / component XML):**

- A concrete value (`us`, `kr`, `us(dvorak)` / layout+variant) = the XKB
  layout the engine *requests* while active. **[FACT — field purpose.]**
- Special value **`default`** = "do not change the keyboard layout; use the
  current/system layout." **[STRONG — `default` is the IBus EngineDesc
  sentinel for inherit-current; corroborated by ibus#1614 "'Default' XKB
  layout in ibus" and mozc#1142, where `layout=default` inherits and can drop
  `layout_variant`.]**
- Omitting `<layout>` → EngineDesc layout defaults to `default`. **[INFER,
  strong]**

**Per-backend behavior:**

- **X11 (ibus-daemon owns XKB):** switching to an engine applies its
  `<layout>` via the XKB/`setxkbmap` path. `us` forces us; `default` keeps the
  user's current layout. **[STRONG]**
- **GNOME Wayland (mutter owns XKB; gnome-shell drives IBus over D-Bus):**
  mutter does **not** honor the engine's `<layout>` to switch the system XKB
  layout. The mechanism IBus would use (`setxkbmap` / ibus-ui-gtk3's internal
  layout switch) is a documented no-op on Wayland. So `<layout>us</layout>`
  does **not** force us, and a Dvorak user keeps Dvorak. **[STRONG]**
  (ibus#2408, ibus#2644, ibus#2684, RH BZ 2076596.)

Therefore on GNOME Wayland **`default` vs `us` vs omitted are behaviorally
identical** (all ignored for layout switching). They differ only on X11.
Net for your bug: `<layout>us</layout>` was never actually forcing us on your
Wayland session — the Dvorak user stayed in Dvorak — which is half of why the
ForwardKeyEvent remap both seemed necessary and still failed. **[STRONG]**

---

## §C. Why `ForwardKeyEvent` remapping fails on Wayland (and can't be fixed)

`ForwardKeyEvent(keyval, keycode, state)` re-emits a key event "as if not
consumed." On GNOME Wayland gnome-shell turns that into a real key event and
delivers it to the focused client. Because **Wayland clients derive the
character from the keycode via their own XKB state** (not from any keysym you
attach), the path is:

```
you ForwardKeyEvent(keyval='p'=0x70, keycode=<QWERTY-R>, state)
  -> gnome-shell re-injects a key event carrying that keycode
  -> focused client applies ITS xkb_state (Dvorak) to the keycode
  -> client produces the Dvorak result again; your 'p' is discarded
```

That matches your symptom exactly ("I ForwardKeyEvent `p` but the app still
receives `r`" — the app re-derives from keycode, ignoring the forwarded
keyval's character). **[STRONG]**

**Source-grounded mechanism** (gnome-shell `js/misc/inputMethod.js`): the
engine's `forward-key-event` D-Bus signal is handled by
`_onForwardKeyEvent(_context, keyval, keycode, state)` which re-emits it
through Clutter's `InputMethod.forward_key()` using the *keycode*:

```js
// js/misc/inputMethod.js (gnome-shell)
_onForwardKeyEvent(_context, keyval, keycode, state) {
    const press = (state & IBus.ModifierType.RELEASE_MASK) === 0;
    state &= ~IBus.ModifierType.RELEASE_MASK;
    ...
    this.forward_key(keyval, keycode + 8, state & Clutter.ModifierType.MODIFIER_MASK, time, press);
}
```

The `keycode + 8` converts the IBus/evdev keycode back to a clutter/X11
keycode; the event is then re-injected into the clutter input pipeline as a
real key event. The focused client re-derives the symbol/character from that
keycode via its own XKB state, so your forwarded `keyval` character does not
determine the result. The `state & Clutter.ModifierType.MODIFIER_MASK` also
reveals a real bug (gnome-shell#5782): `MODIFIER_MASK` does **not** include
Control, so forwarded Ctrl+key combos have Ctrl stripped on Wayland. Both
facts confirm gnome-shell reconstructs a real key event from the keycode
rather than honoring your keyval character. The same re-injection path is what
broke ibus-bamboo's ForwardKeyEvent flow (Arch thread #283571). **[FACT —
gnome-shell `js/misc/inputMethod.js` `_onForwardKeyEvent`]**

Consequences:
- **Pass-through (forward the *original* event, or return `false`) works** —
  that is ForwardKeyEvent's legitimate use. **[STRONG]**
- **Substituting a different character via a swapped keyval does NOT work**
  and cannot for shortcut matching, because the keycode is the source of truth
  on the client. Forging a *different keycode* doesn't help either: the
  client's XKB layout would still re-map that keycode. **No.** **[STRONG]**
- **To inject text you control, use `CommitText`** (a UTF-8 string the client
  takes verbatim). Faking characters via synthetic key events is inherently
  fragile on Wayland. **[STRONG]**

---

## §2 recap. X11 vs Wayland

- **Engine receives the same thing both ways:** the current-layout keysym.
  **[STRONG]**
- **`<layout>` honoring differs:** X11 applies it; Wayland ignores it.
  **[STRONG]**
- **keycode offset differs:** the IBus/Wayland side is the evdev keycode;
  clutter/X11 is evdev+8. Confirmed by gnome-shell `inputMethod.js` doing
  `keycode + 8` when re-injecting a forwarded event (so what IBus handed it
  was evdev). Don't hardcode a +8 if you key a jamo table off `keycode` from
  ProcessKeyEvent — that value is evdev. **[FACT for the +8 add-back direction
  via gnome-shell#5782; verify the absolute value you receive in
  ProcessKeyEvent empirically]**

---

## Recommendation for geulbus (generic, layout-agnostic IME)

1. **Change `<layout>us</layout>` → `<layout>default</layout>`** (or remove
   the element). Explicitly inherit the user's XKB layout. No-op on Wayland
   today, but correct on X11 and future-proof, and stops forcing `us` onto
   Dvorak/Colemak users. **[STRONG]**

2. **Delete the Dvorak-remap + ForwardKeyEvent shortcut code.** It cannot work
   on Wayland and is unnecessary: with no forced layout, shortcuts already
   arrive in the user's layout; pass them through (return `false`, or forward
   the *original* event) and the app re-derives them correctly. **[STRONG]**

3. **Decide Hangul jamo binding deliberately (the real choice):**
   - Want jamo fixed to **physical positions regardless of Latin layout**
     (most robust, most IME-like): build the jamo table keyed on **keycode**,
     not keyval. A Dvorak user still gets ㅂ on the physical top-left, etc.
     Recommended for a generic IME. **[INFER, recommended]**
   - Want jamo to **follow the user's Latin layout**: key the jamo table on
     **keyval** and do nothing else. Simpler; jamo positions move with the
     layout. Document the choice either way.

4. **Use `CommitText` for Latin output you control**, not forged keysyms.
   **[STRONG]**

5. **Never try to switch/force XKB layout from the engine on Wayland** — mutter
   silently no-ops it. Users pick their Latin layout as a GNOME input source /
   XKB option themselves. **[STRONG]**

Net: geulbus becomes layout-agnostic. Shortcuts and Latin behavior follow the
user's configured XKB layout (Dvorak/Colemak/us — all free); the only thing
geulbus pins is Hangul composition (ideally keyed to keycodes for stable jamo
positions).

---

## Verify on the box (upgrades [STRONG] → [FACT])

Quick local checks to confirm before committing the design:

1. **Real keyval under Dvorak.** Set GNOME layout to English (Dvorak), switch
   to geulbus, log `ProcessKeyEvent` args, press physical QWERTY-R. Expect
   `keyval == 0x70 ('p')`; note the `keycode` value to settle the evdev vs +8
   offset. Settles §1 and the offset note.
2. **`<layout>` ignored on Wayland.** With geulbus active and
   `<layout>us</layout>`, a Dvorak user typing latin should still get Dvorak
   → confirms `<layout>` is ignored.
3. **ForwardKeyEvent re-derivation.** Forward keyval=`p` with the QWERTY-R
   keycode into a text field; confirm the field shows the *layout* result, not
   `p`.

(Local component-XML facts in §B captured directly and firm: geulbus=`us`,
hangul=`kr`/`kr104`, chewing=`us`. Versions: IBus 1.5.33, mutter 49.5,
gnome-shell 49.6.)

---

## Sources

- libxkbcommon — Keyboard State / `xkb_state_key_get_one_sym` (keysym per
  active layout+level): https://xkbcommon.org/doc/current/group__state.html
- The Wayland Protocol — "XKB, briefly" (compositor sends keymap+keycodes;
  client derives symbols via its own xkb_state):
  https://wayland-book.com/seat/xkb.html
- Martin Gräßlin — "How input works – Keyboard input" (Wayland sends keycode;
  symbol translation happens client/compositor side):
  https://blog.martin-graesslin.com/blog/2016/12/how-input-works-keyboard-input/
- GDK Keyboard Handling / keys.md (keyval = layout-translated keysym;
  keycode = layout-independent physical key; "IMs that expect a layout should
  use keycode"):
  https://developer-old.gnome.org/gdk4/stable/gdk4-Keyboard-Handling.html ,
  https://chromium.googlesource.com/external/github.com/GNOME/gtk/+/HEAD/docs/reference/gdk/keys.md
- gnome-shell `js/misc/inputMethod.js` (THE key path; verified this session):
  `vfunc_filter_key_event` → `process_key_event_async(event.get_key_symbol(),
  event.get_key_code() - 8, state, ...)` and `_onForwardKeyEvent` →
  `forward_key(keyval, keycode + 8, ...)`:
  https://github.com/GNOME/gnome-shell/blob/main/js/misc/inputMethod.js
- gnome-shell IBus glue: js/misc/ibusManager.js, js/ui/status/keyboard.js:
  https://github.com/GNOME/gnome-shell/blob/main/js/misc/ibusManager.js
- IBus IBusEngine / process_key_event docs (keyval/keycode/state semantics;
  forward_key_event): https://valadoc.org/ibus-1.0/IBus.Engine.process_key_event.html ,
  https://ibus.github.io/docs/ibus-1.5/IBusInputContext.html
- IBus Keyboard Layouts overview (ibus_keymap_lookup_keysym; layout handling):
  https://deepwiki.com/ibus/ibus/7.2-keyboard-layouts
- ibus#1614 "'Default' XKB layout in ibus" (semantics of layout=default):
  https://github.com/ibus/ibus/issues/1614
- mozc#1142 (layout=default inherits current; can drop layout_variant):
  https://github.com/google/mozc/discussions/1142
- ForwardKeyEvent on Wayland broke ibus-bamboo (gnome-shell re-injects a real
  key event): https://bbs.archlinux.org/viewtopic.php?id=283571
- gnome-shell#5782 (quotes the actual `js/misc/inputMethod.js` line:
  `this.forward_key(keyval, keycode + 8, state & Clutter.ModifierType.MODIFIER_MASK, ...)`
  — proves forwarded IM events are re-injected via keycode through a virtual
  device, and that Control is stripped by MODIFIER_MASK on Wayland):
  https://gitlab.gnome.org/GNOME/gnome-shell/-/issues/5782
- Per-engine XKB layout switching is a no-op on Wayland (setxkbmap doesn't
  work): ibus#2408 https://github.com/ibus/ibus/issues/2408 ,
  ibus#2644 https://github.com/ibus/ibus/issues/2644 ,
  ibus#2684 https://github.com/ibus/ibus/issues/2684 ,
  RH BZ 2076596 https://bugzilla.redhat.com/show_bug.cgi?id=2076596

Both load-bearing gnome-shell call sites are now confirmed directly from
source (`js/misc/inputMethod.js`, GNOME main): the inbound path
`vfunc_filter_key_event` passes `event.get_key_symbol()` as `keyval` and
`event.get_key_code() - 8` as `keycode`; the outbound/forward path
`_onForwardKeyEvent` re-injects via `forward_key(keyval, keycode + 8, ...)`.
Only-still-[STRONG] items (not FACT): mutter's internal use of
`xkb_state_key_get_one_sym` for the clutter symbol (well-established but not
re-grepped here) and IBus `src/ibusenginedesc.c` `default`/layout handling on
each backend.
