# IBus Korean Engine in Pure Rust over D-Bus (zbus)

Goal: implement an IBus input-method **engine** entirely in Rust, talking D-Bus
directly with the [`zbus`](https://docs.rs/zbus) crate. **No `libibus`, no GObject,
no C glue.** This is proven viable by **librush** (`fm-elpac/librush`,
a.k.a. `艾刷` = lib + IBus + Rust), a working pure-Rust IBus module that this guide
is based on, cross-checked against the IBus C source.

> Accuracy note on "kime": **kime does NOT have an IBus engine.** Its Rust
> frontends are only `xim` (uses the `xim` crate over `x11rb`) and `wayland`.
> IBus support is the still-open issue [Riey/kime#422 "ibus 서버 인터페이스 구현"].
> Its GTK/Qt frontends are C/C++ immodules, not engines. So the real reference for
> a pure-Rust zbus IBus engine is **librush**, not kime. Do not copy a kime "ibus
> frontend" — it does not exist.

All numeric constants below were read live from `python3 -c "import gi; ... IBus"`
on this machine (ibus 1.5.33, Fedora 43), so they are ground-truth, not guessed.

---

## 1. ARCHITECTURE: out-of-process IBus engine

```
  app (GTK/Qt) ──im-module──> ibus-daemon ──D-Bus──> YOUR engine process
                               (owns its own bus)      (this Rust program)
```

`ibus-daemon` runs **its own private D-Bus bus** (NOT the session bus). It reads
component `.xml` files at startup, and when the user selects your engine, it
`fork+exec`s your `<exec>` command. Your process then connects back to the daemon's
bus, registers a **Factory**, and the daemon calls `CreateEngine` on it.

### Discovery + activation flow

1. **Install a component `.xml`** under `/usr/share/ibus/component/` (system) or
   `~/.config/ibus/component/` (user; create dir if missing). The daemon scans
   these on (re)start. Trigger a rescan with `ibus restart` or
   `ibus write-cache --system`.
2. The `.xml` `<exec>` is launched by the daemon. Pass a flag (e.g. `--ibus`) so
   your binary knows it was started by ibus.
3. Your process **finds the IBus bus address** (section 5), connects, and obtains
   a unique name (the daemon's `Hello` handshake; zbus does this automatically on
   `Builder::address(...).build()`).
4. Your process **requests the well-known bus name** = the component `<name>`
   string (e.g. `org.freedesktop.IBus.Presguel`). In librush this is literally
   the same string used as the component `<name>` and the `request_name` argument.
5. Your process **registers a Factory** object at path
   `/org/freedesktop/IBus/Factory` implementing `org.freedesktop.IBus.Factory`.
6. Daemon calls `org.freedesktop.IBus.Factory.CreateEngine(name:s) -> o`. You
   create an Engine object, serve it at e.g.
   `/org/freedesktop/IBus/Engine/1`, and return that object path.
7. Daemon talks to that object via `org.freedesktop.IBus.Engine`. Key events flow
   in as method calls; you push text back out as **signals** on the same object.

### Exact names / paths (constants)

| Thing | Value |
|---|---|
| Engine bus address | from `IBUS_ADDRESS` env, else address file (section 5) — **not** session bus |
| Well-known name to request | = component `<name>`, e.g. `org.freedesktop.IBus.Presguel` |
| Factory object path | `/org/freedesktop/IBus/Factory` |
| Factory interface | `org.freedesktop.IBus.Factory` |
| Factory method | `CreateEngine(name:s) -> (path:o)` |
| Engine object path | you choose, e.g. `/org/freedesktop/IBus/Engine/{n}` |
| Engine interface | `org.freedesktop.IBus.Engine` |
| Daemon's own service | `org.freedesktop.IBus`, path `/org/freedesktop/IBus`, iface `org.freedesktop.IBus` (has `GetAddress`, `CreateInputContext`, etc.) |

### Component `.xml` template (CRITICAL — copy this)

Install to `/usr/share/ibus/component/presguel.xml`. Modeled on the live
`hangul.xml` and on librush's `pmim_ibrus.xml`. The component `<name>` MUST equal
the D-Bus well-known name your process requests.

```xml
<?xml version="1.0" encoding="utf-8"?>
<!-- /usr/share/ibus/component/presguel.xml -->
<component>
  <name>org.freedesktop.IBus.Presguel</name>
  <description>Presguel Korean Input Method</description>
  <exec>/usr/lib/presguel/presguel-ibus --ibus</exec>
  <version>0.1.0</version>
  <author>You &lt;you@example.com&gt;</author>
  <license>GPL</license>
  <homepage>https://example.com/presguel</homepage>
  <textdomain>presguel</textdomain>

  <engines>
    <engine>
      <name>presguel</name>            <!-- the name passed to CreateEngine -->
      <language>ko</language>
      <license>GPL</license>
      <author>You &lt;you@example.com&gt;</author>
      <icon>presguel</icon>
      <layout>kr</layout>
      <layout_variant>kr104</layout_variant>
      <longname>Presguel (한글)</longname>
      <description>Korean Input Method (날개셋-style)</description>
      <rank>99</rank>
      <symbol>한</symbol>             <!-- shown in the panel; &#xD55C; == 한 -->
      <!-- <setup>/usr/lib/presguel/presguel-setup</setup>  optional -->
    </engine>
  </engines>
</component>
```

Notes:
- `<name>` at top = well-known bus name your process requests.
- `<engine><name>` = the string the daemon passes to `CreateEngine` (here
  `"presguel"`); reject any other name in `create_engine`.
- `<layout>kr</layout>` puts the keyboard in the Korean physical layout; you still
  receive raw keysyms in `ProcessKeyEvent`.
- After installing, `ibus restart` (or log out/in), then pick "Presguel" in the
  IBus settings / GNOME input-source list.

### Alternative: dynamic registration (no file)

Instead of (or in addition to) the `.xml`, you can call
`org.freedesktop.IBus.RegisterComponent(component:v)` on `org.freedesktop.IBus`
at `/org/freedesktop/IBus`, passing a serialized `IBusComponent`. This is more
work (you must serialize the whole component tree) and is typically used for
engines launched independently (e.g. for testing without installing). **For a
shippable engine, the `.xml` file is the standard, simplest path.** Recommend
starting with the file.

---

## 2. `org.freedesktop.IBus.Engine` interface

Below is the full interface as introspected in IBus 1.5 (from librush's verbatim
copy of `src/ibusengine.c`'s introspection XML). **E** = essential for a Hangul
engine, **U** = UI/candidate only (stub it `Ok(())`), **—** = ignore/stub.

### Methods you implement (daemon -> engine)

| Method | Signature | Need |
|---|---|---|
| `ProcessKeyEvent` | `(u keyval, u keycode, u state) -> b` | **E** (the core) |
| `FocusIn` | `()` | **E** (commit/clear on focus change) |
| `FocusInId` | `(s object_path, s client)` | — (newer variant; stub) |
| `FocusOut` | `()` | **E** |
| `FocusOutId` | `(s object_path)` | — (stub) |
| `Reset` | `()` | **E** (drop current composition) |
| `Enable` | `()` | E-ish (engine turned on) |
| `Disable` | `()` | E-ish (engine turned off) |
| `SetCapabilities` | `(u caps)` | recommended (see caps bits below) |
| `SetCursorLocation` | `(i x, i y, i w, i h)` | U (popup placement) |
| `PropertyActivate` | `(s name, u state)` | — (property menu) |
| `PropertyShow` | `(s name)` | — |
| `PropertyHide` | `(s name)` | — |
| `CandidateClicked` | `(u index, u button, u state)` | U |
| `PageUp` / `PageDown` | `()` | U |
| `CursorUp` / `CursorDown` | `()` | U |
| `SetSurroundingText` | `(v text, u cursor_pos, u anchor_pos)` | — (advanced) |
| `ProcessHandWritingEvent` | `(ad coordinates)` | — |
| `CancelHandWriting` | `(u n_strokes)` | — |
| `PanelExtensionReceived` | `(v event)` | — |
| `PanelExtensionRegisterKeys` | `(v data)` | — |

Properties (zbus `#[zbus(property)]`): `ContentType` `(uu)` write,
`FocusId` `(b)` read, `ActiveSurroundingText` `(b)` read. Return defaults.

### Signals you emit (engine -> daemon)

| Signal | Signature | Need |
|---|---|---|
| `CommitText` | `(v text)` | **E** — commits finished text to the app |
| `UpdatePreeditText` | `(v text, u cursor_pos, b visible, u mode)` | **E** — shows composing syllable. (older 3-arg `UpdatePreeditText(v,u,b)` and `UpdatePreeditTextWithMode(v,u,b,u)` exist; current daemon expects the 4-arg form with `mode`.) |
| `ShowPreeditText` | `()` | E (or just toggle `visible` in UpdatePreeditText) |
| `HidePreeditText` | `()` | E (or `visible=false` in UpdatePreeditText) |
| `UpdateAuxiliaryText` | `(v text, b visible)` | U |
| `UpdateLookupTable` | `(v table, b visible)` | U (hanja candidates) |
| `RegisterProperties` | `(v props)` | — |
| `UpdateProperty` | `(v prop)` | — |
| `ForwardKeyEvent` | `(u keyval, u keycode, u state)` | useful — re-inject a key you chose not to handle |

> Practical Hangul minimum: `ProcessKeyEvent`, `FocusIn`, `FocusOut`, `Reset`
> (+ `Enable`/`Disable`) as methods; `CommitText`, `UpdatePreeditText`
> (4-arg) as signals. You can drive show/hide purely through the `visible`
> argument of `UpdatePreeditText` and never emit `Show/HidePreeditText`.
> `ForwardKeyEvent` is handy for keys you decline (e.g. Enter while no
> composition) but usually just returning `false` from `ProcessKeyEvent` is
> enough (the daemon then forwards the key to the app).

`mode` in `UpdatePreeditText` = `IBusPreeditFocusMode`: **CLEAR=0, COMMIT=1**
(what to do with preedit when focus moves: discard vs auto-commit).

---

## 3. IBusText / IBusAttrList SERIALIZATION  (the #1 thing to get right)

IBus types are GObjects that serialize to GVariant via the `IBusSerializable`
contract. **Every serializable starts with a header `(s a{sv} ...)`** where the
first `s` is the GType name string and `a{sv}` is the "attachments" dict (almost
always empty `{}`). Subclass fields follow.

Verified from IBus C source:
- `ibus_serializable_serialize_object()` adds `g_type_name(...)` first (the `s`).
- `ibus_serializable_real_serialize()` adds the attachments `a{sv}`.
- `ibus_text_serialize()` then adds: text `s`, then a **variant** wrapping the
  serialized `IBusAttrList`.
- `ibus_attr_list_serialize()` adds an **`av`** (array of *variants*), each
  element a serialized `IBusAttribute`.
- `ibus_attribute_serialize()` adds four `u`: `type`, `value`,
  `start_index`, `end_index`.

So the exact GVariant signatures are:

```
IBusText      : (sa{sv}sv)            name="IBusText",      attach={}, text, <attrlist>
IBusAttrList  : (sa{sv}av)            name="IBusAttrList",  attach={}, [ <attr>, ... ]
IBusAttribute : (sa{sv}uuuu)          name="IBusAttribute", attach={}, type, value, start, end
```

> **GOTCHA #1:** the attribute list is **`av` (array of variant)**, NOT `a(uuuu)`.
> Each attribute is its own full serializable wrapped in a variant. Sending
> `a(uuuu)` directly will mis-deserialize and (historically) could even **crash
> ibus-daemon** — see [ibus/ibus#2611] ("ibus-daemon gvariant deserialization
> crash"), filed by someone writing exactly this kind of pure-Rust zbus engine.
> Get the structure right or the daemon dies.
>
> **GOTCHA #2:** the whole `IBusText` is sent as the signal arg of type `v`
> (variant). So `CommitText`'s wire arg is a variant *containing* the
> `(sa{sv}sv)` struct. In zbus, declaring the signal arg as `Value<'_>` and
> passing `Value::new(structure)` does the variant-wrapping for you.

### zvariant construction (verified working, from librush `ibus_serde.rs`)

```rust
use std::collections::HashMap;
use zbus::zvariant::{Structure, Value};

/// Build an IBusText with NO attributes (plain text). Signature: (sa{sv}sv)
pub fn make_ibus_text(text: String) -> Value<'static> {
    // inner IBusAttrList: (sa{sv}av) with empty attribute array
    let attr_list = Structure::from((
        "IBusAttrList",
        HashMap::<String, Value<'static>>::new(), // a{sv} attachments = {}
        Vec::<Value<'static>>::new(),             // av = []  (array of variant)
    ));

    // outer IBusText: (sa{sv}sv)
    let ibus_text = Structure::from((
        "IBusText",
        HashMap::<String, Value<'static>>::new(), // a{sv} = {}
        text,                                     // s
        Value::new(attr_list),                    // v wrapping the attrlist struct
    ));

    Value::new(ibus_text)
}
```

librush asserts exactly this in a unit test:
`make_ibus_text("test").value_signature() == "(sa{sv}sv)"`.

### IBusText WITH an underline attribute (for preedit) — extend the above

Hangul preedit is normally drawn underlined. Add one `IBusAttribute` of type
UNDERLINE/SINGLE spanning the whole preedit string. **Each attribute element of
the `av` array must itself be a `Value::new(Structure(...))` of `(sa{sv}uuuu)`.**

Numeric constants (read live from this machine's IBus):

```
IBUS_ATTR_TYPE_UNDERLINE   = 1     IBUS_ATTR_UNDERLINE_NONE   = 0
IBUS_ATTR_TYPE_FOREGROUND  = 2     IBUS_ATTR_UNDERLINE_SINGLE = 1
IBUS_ATTR_TYPE_BACKGROUND  = 3     IBUS_ATTR_UNDERLINE_DOUBLE = 2
                                   IBUS_ATTR_UNDERLINE_LOW    = 3
                                   IBUS_ATTR_UNDERLINE_ERROR  = 4
```

`start_index` / `end_index` are in **characters** (Unicode code points / glyphs as
IBus counts them via `g_utf8` offsets), not bytes; `end` is exclusive. For a full
underline of a 2-char preedit: `start=0, end=2`.

```rust
use std::collections::HashMap;
use zbus::zvariant::{Structure, Value};

const IBUS_ATTR_TYPE_UNDERLINE: u32 = 1;
const IBUS_ATTR_UNDERLINE_SINGLE: u32 = 1;

/// One serialized IBusAttribute: (sa{sv}uuuu)
fn ibus_attribute(attr_type: u32, value: u32, start: u32, end: u32) -> Value<'static> {
    Value::new(Structure::from((
        "IBusAttribute",
        HashMap::<String, Value<'static>>::new(), // a{sv}
        attr_type, // u  (UNDERLINE=1, FOREGROUND=2, BACKGROUND=3)
        value,     // u  (UNDERLINE_SINGLE=1, or 0xRRGGBB for fg/bg)
        start,     // u  start char index (inclusive)
        end,       // u  end char index (exclusive)
    )))
}

/// IBusText whose entire span is single-underlined (typical Hangul preedit).
pub fn make_preedit_text(text: String) -> Value<'static> {
    let char_len = text.chars().count() as u32;

    // av: array of variant, each a serialized IBusAttribute
    let attrs: Vec<Value<'static>> = vec![ibus_attribute(
        IBUS_ATTR_TYPE_UNDERLINE,
        IBUS_ATTR_UNDERLINE_SINGLE,
        0,
        char_len,
    )];

    let attr_list = Structure::from((
        "IBusAttrList",
        HashMap::<String, Value<'static>>::new(),
        attrs, // av  (NOT a(uuuu)!)
    ));

    Value::new(Structure::from((
        "IBusText",
        HashMap::<String, Value<'static>>::new(),
        text,
        Value::new(attr_list),
    )))
}
```

For foreground color use `attr_type=2, value=0xRRGGBB`; background `attr_type=3`.
Multiple attributes = push more `Value::new(Structure(...))` into the `av` Vec.

`IBusLookupTable` (for hanja candidates) serializes as
`(sa{sv}uubbi av av)` = name, attach, page_size:u, cursor_pos:u,
cursor_visible:b, round:b, orientation:i, candidates:av(of IBusText),
labels:av(of IBusText). See librush `lookup_table.rs` for the exact builder.

---

## 4. KEY EVENT decoding

`ProcessKeyEvent(keyval:u, keycode:u, state:u) -> b`.

- **`keyval`** = X11/GDK **keysym**. IBus's `IBUS_KEY_*` constants are identical to
  the X11 `XK_*` constants, so the [`xkeysym`](https://docs.rs/xkeysym) crate's
  `Keysym` maps 1:1 (librush feeds `keyval.into()` straight into `xkeysym::Keysym`).
- **`keycode`** = hardware scancode (layout-independent). Usually ignore for a
  layout-driven Hangul engine; use `keyval`.
- **`state`** = modifier bitmask. **Bits (verified live):**

```
IBUS_SHIFT_MASK    = 1<<0  = 0x00000001
IBUS_LOCK_MASK     = 1<<1  = 0x00000002   (Caps Lock)
IBUS_CONTROL_MASK  = 1<<2  = 0x00000004
IBUS_MOD1_MASK     = 1<<3  = 0x00000008   (Alt)
IBUS_MOD2_MASK     = 1<<4               (Num Lock)
IBUS_MOD4_MASK     = 1<<6               (Super/Win)
IBUS_HANDLED_MASK  = 1<<24
IBUS_FORWARD_MASK  = 1<<25
IBUS_SUPER_MASK    = 1<<26
IBUS_META_MASK     = 1<<28
IBUS_RELEASE_MASK  = 1<<30 = 0x40000000   (key RELEASE, not press)
```

- **Ignore release events**: `if state & IBUS_RELEASE_MASK != 0 { return Ok(false); }`.
  Hangul composition acts on key *press* only.
- **Ignore keybinding modifiers**: if Control/Alt/Super/Meta are held, it's a
  shortcut, not text — return `false` so the app handles it. librush's helper:
  `has_special_modifiers = control|mod1|mod4|super|meta|hyper`.
- **Printable ASCII**: for printable keys, `keyval` **equals the ASCII code of the
  base character**. `XK_a`=0x61 (97), `XK_A`=0x41 (65), `XK_1`=0x31, `XK_space`=0x20.
  Shift produces the shifted keysym directly (e.g. Shift+`a` arrives as
  `keyval == 0x41 == 'A'`; Shift+`2` as `0x40 == '@'`). So:
  ```rust
  // ASCII printable range 33..=126 maps directly to keyval.
  if (0x20..=0x7e).contains(&keyval) {
      let ch = char::from_u32(keyval).unwrap(); // already shift-resolved
      // index your KeyTable by `ch as u8` (33..=126), or 0x20 for space
  }
  ```
  Your "날개셋-style" KeyTable indexed by ASCII 33..126 indexes directly on
  `keyval` for those values. Space (0x20) is special-cased (commit / pass-through).
- **Special Hangul keys (verified live):**

```
IBUS_Hangul        = 0xff31    (한/영 toggle key; the dedicated Hangul key)
IBUS_Hangul_Hanja  = 0xff34    (한자 conversion key)
IBUS_BackSpace     = 0xff08    (edit current syllable: pop last jamo)
IBUS_Return        = 0xff0d
IBUS_Escape        = 0xff1b
IBUS_space         = 0x0020
```
  Other useful `0xff..` keysyms: `Tab=0xff09`, `Delete=0xffff`,
  `Left=0xff51 .. Down=0xff54`, `Home=0xff50`, `End=0xff57`.
  (`xkeysym` exposes these as `Keysym::Hangul`, `Keysym::BackSpace`, etc.)

- **Return value**: `true` = "I consumed this key" (engine handled it, app sees
  nothing). `false` = "not mine" (daemon forwards the raw key to the app). When
  composition is empty and a non-jamo key arrives, return `false`. When BackSpace
  should delete a committed char (no preedit), return `false` so the app deletes.

---

## 5. zbus SPECIFICS

### Finding the IBus bus address (NOT the session bus)

IBus runs a private bus. Resolution order (librush `addr.rs`, mirroring
`ibus_get_address` in `ibusshare.c`):

1. `$IBUS_ADDRESS` env var → use directly.
2. else `$IBUS_ADDRESS_FILE` → read that file.
3. else compute the address file path:
   `$XDG_CONFIG_HOME/ibus/bus/<machine-id>-<host>-<display>`
   (fallback `$HOME/.config/ibus/bus/...`), where:
   - `<machine-id>` = trimmed contents of `/var/lib/dbus/machine-id` or
     `/etc/machine-id`.
   - On **Wayland**: `host="unix"`, `display=$WAYLAND_DISPLAY` (e.g. `wayland-0`),
     filename ends `...-unix-wayland-0`.
   - On **X11**: parse `$DISPLAY` (`host:display.screen`); empty host → `"unix"`,
     filename ends `...-unix-0`.
4. Read the file, skip `#` comment lines, take the `IBUS_ADDRESS=` line's value.
   That value is a normal D-Bus address string
   (`unix:path=/run/user/1000/ibus/dbus-XXXX,guid=...`).

You can also ask the daemon: call `GetAddress()` on `org.freedesktop.IBus`
`/org/freedesktop/IBus` — but you'd need an address to do that, so the file/env
method is the bootstrap.

### Connecting + serving with zbus

zbus connects to a **custom address** (not `Connection::session()`):
`Builder::address(addr)?.build().await?`. The `Hello` handshake / unique name is
automatic. Then `request_name(well_known)` and serve objects via `object_server().at(path, obj)`.

```rust
use zbus::connection::Builder;
let conn = Builder::address(addr.as_str())?.build().await?;
let _unique = conn.unique_name();          // proves Hello succeeded
conn.request_name("org.freedesktop.IBus.Presguel").await?;
conn.object_server().at("/org/freedesktop/IBus/Factory", factory).await?;
```

Interfaces use `#[zbus::interface(name = "...")]`; signals are `#[zbus(signal)]`
async fns that take `&SignalEmitter`. Inside a method you get the emitter via
`#[zbus(signal_emitter)] se: SignalEmitter<'_>` and the object server via
`#[zbus(object_server)] server: &ObjectServer`. Emitting a signal = calling the
signal fn with that emitter; zbus routes it to the right object path.

### Cargo deps (from librush, zbus 5.x)

```toml
[dependencies]
zbus     = { version = "5", default-features = false, features = ["tokio"] }
zvariant = "5"            # re-exported as zbus::zvariant; pin to match zbus
tokio    = { version = "1", features = ["full"] }
xkeysym  = "0.2"          # Keysym mapping for keyval
# (librush also uses bitbybit/arbitrary-int for the modifier bitfield — optional)
```

> `zbus = { default-features = false, features = ["tokio"] }` — pick `tokio` OR
> `async-io`, not the default. Use `zbus::zvariant` re-export so versions match.

### Minimal compilable sketch

```rust
use std::collections::HashMap;
use zbus::{
    connection::Builder, fdo, interface, object_server::SignalEmitter,
    Connection, ObjectServer,
    zvariant::{ObjectPath, Structure, Value},
};

const IBUS_RELEASE_MASK: u32 = 1 << 30;

// ---- IBusText builder (section 3) ----
fn make_ibus_text(text: String) -> Value<'static> {
    let attr_list = Structure::from((
        "IBusAttrList",
        HashMap::<String, Value<'static>>::new(),
        Vec::<Value<'static>>::new(),
    ));
    Value::new(Structure::from((
        "IBusText",
        HashMap::<String, Value<'static>>::new(),
        text,
        Value::new(attr_list),
    )))
}

// ---- Engine object: org.freedesktop.IBus.Engine ----
struct Engine {
    preedit: String, // your hangul automaton state would live here
}

#[interface(name = "org.freedesktop.IBus.Engine")]
impl Engine {
    async fn process_key_event(
        &mut self,
        #[zbus(signal_emitter)] se: SignalEmitter<'_>,
        keyval: u32,
        _keycode: u32,
        state: u32,
    ) -> fdo::Result<bool> {
        if state & IBUS_RELEASE_MASK != 0 {
            return Ok(false); // ignore key releases
        }
        // toy logic: printable ASCII -> commit it; demonstrates both signals
        if (0x21..=0x7e).contains(&keyval) {
            let ch = char::from_u32(keyval).unwrap();
            // show as preedit (underlined in a real impl), then commit on next key:
            self.preedit.push(ch);
            Engine::update_preedit_text(
                &se,
                make_ibus_text(self.preedit.clone()),
                self.preedit.chars().count() as u32, // cursor
                true,                                  // visible
                0,                                     // mode = CLEAR
            ).await?;
            return Ok(true);
        }
        if keyval == 0x20 && !self.preedit.is_empty() {
            // space commits the buffer
            Engine::commit_text(&se, make_ibus_text(std::mem::take(&mut self.preedit))).await?;
            Engine::update_preedit_text(&se, make_ibus_text(String::new()), 0, false, 0).await?;
            return Ok(true);
        }
        Ok(false)
    }

    async fn focus_in(&mut self) -> fdo::Result<()> { Ok(()) }
    async fn focus_out(&mut self) -> fdo::Result<()> { Ok(()) }
    async fn reset(&mut self) -> fdo::Result<()> { self.preedit.clear(); Ok(()) }
    async fn enable(&mut self) -> fdo::Result<()> { Ok(()) }
    async fn disable(&mut self) -> fdo::Result<()> { Ok(()) }
    fn set_capabilities(&mut self, _caps: u32) -> fdo::Result<()> { Ok(()) }

    // ---- signals (engine -> daemon). Note `text: Value` => wire type `v`. ----
    #[zbus(signal)]
    async fn commit_text(se: &SignalEmitter<'_>, text: Value<'_>) -> zbus::Result<()>;

    #[zbus(signal)]
    async fn update_preedit_text(
        se: &SignalEmitter<'_>,
        text: Value<'_>,
        cursor_pos: u32,
        visible: bool,
        mode: u32,
    ) -> zbus::Result<()>;

    #[zbus(signal)]
    async fn forward_key_event(
        se: &SignalEmitter<'_>, keyval: u32, keycode: u32, state: u32,
    ) -> zbus::Result<()>;
}

// ---- Factory object: org.freedesktop.IBus.Factory ----
struct Factory { conn: Connection, next: u32 }

#[interface(name = "org.freedesktop.IBus.Factory")]
impl Factory {
    async fn create_engine(&mut self, name: String) -> fdo::Result<ObjectPath<'_>> {
        if name != "presguel" {
            return Err(fdo::Error::Failed(format!("unknown engine: {name}")));
        }
        self.next += 1;
        let path = format!("/org/freedesktop/IBus/Engine/{}", self.next);
        self.conn
            .object_server()
            .at(path.clone(), Engine { preedit: String::new() })
            .await
            .map_err(|e| fdo::Error::Failed(e.to_string()))?;
        ObjectPath::try_from(path).map_err(|e| fdo::Error::Failed(e.to_string()))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = find_ibus_address()?;              // section 5 resolution
    let conn = Builder::address(addr.as_str())?.build().await?;
    let _ = conn.unique_name();                   // Hello handshake done by zbus
    conn.object_server()
        .at("/org/freedesktop/IBus/Factory",
            Factory { conn: conn.clone(), next: 0 })
        .await?;
    conn.request_name("org.freedesktop.IBus.Presguel").await?;
    // keep the process alive forever
    std::future::pending::<()>().await;
    Ok(())
}

fn find_ibus_address() -> Result<String, Box<dyn std::error::Error>> {
    // see librush addr.rs: IBUS_ADDRESS env, else parse the ibus/bus/<...> file
    Ok(std::env::var("IBUS_ADDRESS")?)
}
```

This compiles against zbus 5 with the `tokio` feature. For a real engine, hold the
Hangul automaton in `Engine`, and replace `make_ibus_text` with `make_preedit_text`
(underline) for `update_preedit_text`.

### Capabilities bits (from `SetCapabilities`, verified live)

```
PREEDIT_TEXT=1  AUXILIARY_TEXT=2  LOOKUP_TABLE=4  FOCUS=8  SURROUNDING_TEXT=32
```
Check `caps & PREEDIT_TEXT` before relying on inline preedit; some clients lack it
and you must commit directly instead.

---

## 6. Reference implementations (be accurate)

### librush — the real pure-Rust zbus engine (USE THIS)

- Repo: `https://codeberg.org/fm-elpac/librush` (mirror `github.com/fm-elpac/librush`).
- **Pure Rust, no GObject/libibus.** Talks D-Bus directly with `zbus` 5.x.
- License LGPL-2.1+/GPL-3.0+. Binary `ibrus`. Used by `pmim` (Chinese pinyin), but
  the `src/ibus/` module is a generic, reusable IBus-engine framework.
- Files to copy patterns from:
  - `src/ibus/addr.rs` — IBus address discovery (env + file).
  - `src/ibus/init.rs` — connect (`Builder::address`), `request_name`.
  - `src/ibus/factory.rs` — `org.freedesktop.IBus.Factory` + `CreateEngine`.
  - `src/ibus/engine.rs` — full `org.freedesktop.IBus.Engine` impl, all methods
    + signals, with `#[zbus(signal_emitter)]` / `#[zbus(object_server)]` plumbing
    and the trait-based `IBusEngine` abstraction.
  - `src/ibus/ibus_serde.rs` — `make_ibus_text` (the `(sa{sv}sv)` builder) and the
    `IBusModifierState` bitfield (all the mask bits).
  - `src/ibus/lookup_table.rs` — `IBusLookupTable` serialization for candidates.
  - `aur/pmim_ibrus.xml`, `rpm/ibrus.spec` — component `.xml` + install paths
    (`/usr/share/ibus/component/`, binary in `/usr/lib/<name>/`).
- Gotchas it already solved: the `av`-not-`a(uuuu)` attribute layout; the
  `v`-wrapped IBusText signal arg; the address-file path computation for
  Wayland vs X11; only one engine instance (`/org/freedesktop/IBus/Engine/1`).

### kime — does NOT have an IBus engine

- Repo: `github.com/Riey/kime`. Korean IME, mostly Rust core
  (`src/engine/backends/hangul/` — the actual jamo automaton, worth reading for
  Hangul logic).
- Frontends: `gtk3`/`gtk4` (C immodules), `qt5`/`qt6` (C++), `xim` (Rust, uses the
  `xim` crate over `x11rb`), `wayland` (Rust, `input-method-unstable-v2`).
- **There is NO `ibus` frontend.** IBus support is open issue #422 and unimplemented.
  Its non-IBus integrations are *im-module/protocol* approaches, deliberately
  avoiding ibus-daemon. So: read kime for the **Hangul automaton** (`state.rs`,
  `characters.rs`, `layout.rs`), but **not** for IBus wiring — use librush for that.

### Other Rust IBus projects (context)

- `mominul/ibus` — a Rust reimplementation of libibus (more ambitious, GObject-ish);
  not needed if you go pure-zbus.
- `ibus/ibus#2611` — the bug report from a pure-Rust zbus engine author whose
  malformed IBusText crashed ibus-daemon: concrete proof that the serialization
  layout in section 3 must be exact.

---

## Quick implementation checklist for Presguel

1. `presguel-ibus` binary: resolve IBus address (librush `addr.rs` logic),
   `Builder::address().build()`.
2. Serve `Factory` at `/org/freedesktop/IBus/Factory`; `request_name(
   "org.freedesktop.IBus.Presguel")`.
3. `CreateEngine("presguel")` → serve `Engine` at `/org/freedesktop/IBus/Engine/1`,
   return that path.
4. In `ProcessKeyEvent`: drop releases (`& 1<<30`), pass through special-modifier
   chords (return `false`), map printable `keyval` (0x20–0x7e) to your ASCII
   KeyTable, handle `Hangul=0xff31`/`Hanja=0xff34`/`BackSpace=0xff08`/`space=0x20`.
5. Push composition via `UpdatePreeditText(make_preedit_text(s), cursor, true, 0)`;
   finalize via `CommitText(make_ibus_text(s))` + hide preedit.
6. On `FocusOut`/`Reset`: commit-or-clear the in-progress syllable.
7. Install `/usr/share/ibus/component/presguel.xml`; `ibus restart`; select engine.
8. **Validate the IBusText signature is exactly `(sa{sv}sv)`** with a unit test
   (copy librush's) before sending anything live, or you risk crashing the daemon.

[Riey/kime#422 "ibus 서버 인터페이스 구현"]: https://github.com/Riey/kime/issues/422
[ibus/ibus#2611]: https://github.com/ibus/ibus/issues/2611
