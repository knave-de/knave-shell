# Knave Shell

Knave Shell is the Qt Quick desktop shell for the Knave desktop environment.
It presents desktop UI while Villain remains responsible for composition,
workspaces, windows, input, and focus.

The first implemented slice is a persistent top bar and a workspace overview.
The bar's menu button opens the overview. Pressing and releasing Villain's
`MOD` key on its own opens the same view; a modifier combination such as
`MOD+1` cancels that standalone action.

## Architecture

```text
Qt Quick / QML UI
        │
        │ typed C ABI
        ▼
knave-shell-core (Rust)
        │
        │ Villain JSON IPC v2
        ▼
Villain compositor
```

The QML layer renders the bar, search, workspace carousel, and interactions.
The C++ adapter exposes shell state to QML. `knave-shell-core` owns the typed
IPC client, reconnect behavior, workspace/window models, and decoded preview
cache. Villain generates the real workspace snapshots and handles focus
actions.

The bar and overview are native `wlr-layer-shell` surfaces:

- `bar` is anchored to the top edge and reserves 36 logical pixels.
- `overview` covers the output on the overlay layer and requests exclusive
  keyboard focus while it is open.

The overview searches open windows and workspace actions. Choosing a workspace
or window asks Villain to focus it and then closes the overview. `Escape` closes
the overview without changing focus.

## Repository layout

| Path | Description |
| --- | --- |
| `app/` | C++ application entry point and QML service adapter |
| `core/` | Rust shell core and stable C header |
| `layer-shell/` | Qt Wayland shell-integration plugin |
| `protocols/` | Layer-shell protocol definition used for code generation |
| `qml/` | Bar and workspace overview views |

## Requirements

- Rust and Cargo
- CMake 3.24 or newer
- a C++20 compiler
- Qt 6.6 or newer with Core, Gui, QML, Quick, Quick Controls 2, Wayland Client,
  and the Qt Wayland private development headers
- Wayland client development files, `wayland-protocols`, `wayland-scanner`, and
  `pkg-config`
- Villain with IPC protocol v2 workspace-preview support

The layer-shell integration uses Qt Wayland private API because Qt does not
provide a public API for custom shell roles. Rebuild Knave Shell after a Qt
Wayland update; the plugin is tied to the exact Qt build it was compiled
against.

## Build

```console
cmake -S . -B build -DCMAKE_BUILD_TYPE=Debug
cmake --build build
```

CMake builds the Rust core through Cargo and links its static library into the
shell executable. To install under a chosen prefix:

```console
cmake --install build --prefix /usr/local
```

## Run

Start the persistent bar inside a Wayland session running Villain:

```console
build/bin/knave-shell bar
```

Open or toggle the overview directly:

```console
build/bin/knave-shell overview
```

The shell discovers Villain at
`$XDG_RUNTIME_DIR/villain-$WAYLAND_DISPLAY.sock`. Set `VILLAIN_SOCKET` to use
an explicit socket during development.

Villain's built-in bindings launch `knave-shell overview` when `MOD` is pressed
and released alone. If a Villain configuration contains any custom `[[bind]]`
entries, it replaces the complete default set, so include this binding:

```toml
[[bind]]
keys = "MOD"
dispatch = "exec"
args = ["knave-shell", "overview"]
```

## Verify

Run the Rust checks:

```console
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Then configure and compile the complete application:

```console
cmake -S . -B build -DCMAKE_BUILD_TYPE=Debug
cmake --build build
```

The final interaction check must run in a real Wayland session: open the bar,
open the overview from both the menu button and `MOD`, switch a workspace,
focus a searched window, and confirm `Escape` returns keyboard focus to the
desktop.

## Contributing

Create a branch for each task, use [Conventional Commits](https://www.conventionalcommits.org/),
and squash changes before merging. See [`AGENTS.md`](AGENTS.md) for the
repository workflow rules.

## License

Knave Shell is available under the [MIT License](LICENSE).
