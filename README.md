# Knave Shell

Knave Shell is the desktop-facing shell for the Knave Desktop Environment. The
current implementation is a Qt Quick application with C++ adapters, a Rust
core, and a private Qt Wayland layer-shell plugin. The Rust/wgpu shell is a
future replacement, not the current build.

## Responsibilities

Knave Shell owns presentation and interaction:

- the top bar;
- the workspace and window overview;
- shell-side search and focus actions;
- the Rust IPC client and decoded state model; and
- layer-shell integration for shell surfaces.

Villain remains the source of truth for windows, workspaces, focus, layout,
input, and composition. The shell does not own compositor policy or a
user-facing configuration file.

    Qt Quick / QML
          |
          v
    C++ ShellService
          | typed C ABI
          v
    knave-shell-core (Rust)
          | Villain IPC
          v
    Villain compositor

The current shell uses Villain IPC protocol v2. The Rust core rejects an older
protocol during connection setup.

## Repository layout

| Path | Purpose |
| --- | --- |
| app/ | Application entry point and QML service adapter |
| core/ | Rust state, IPC client, previews, and stable C header |
| layer-shell/ | Qt Wayland shell-integration plugin |
| protocols/ | Wayland XML used for generated client code |
| qml/ | Bar and overview views |

## Requirements

- Rust and Cargo;
- CMake 3.24 or newer, Ninja, and a C++20 compiler;
- Qt 6.6 or newer with Core, Gui, QML, Quick, Quick Controls 2,
  Wayland Client, and Qt Wayland private development headers;
- Wayland client development files, wayland-protocols, wayland-scanner, and
  pkg-config; and
- a running Villain compositor exposing IPC protocol v2.

The layer-shell plugin uses Qt Wayland private API. Rebuild it after a Qt
Wayland update; it is tied to the Qt build it was compiled against.

## Build

Use an isolated Ninja build directory and an explicit staging prefix:

    cmake -S . -B build -G Ninja -DCMAKE_BUILD_TYPE=Debug
    cmake --build build
    cmake --install build --prefix "$PWD/stage"

CMake invokes Cargo with the locked workspace dependency set and links the
debug Rust static library into the shell executable. The current CMake
integration is transitional; it is not a Cargo-only shell build.

Run the Rust checks separately:

    cargo fmt --all -- --check
    cargo clippy --workspace --all-targets -- -D warnings
    cargo test --workspace

## Run

Start Villain first, then run the shell inside the same Wayland session:

    build/bin/knave-shell bar
    build/bin/knave-shell overview

The role defaults to bar. The overview uses a per-display local control socket
so a second overview invocation toggles the existing instance. Set
KNAVE_SHELL_DISABLE_SINGLE_INSTANCE=1 to disable that behavior during testing.

The Rust core discovers Villain at:

    $XDG_RUNTIME_DIR/villain-$WAYLAND_DISPLAY.sock

Set VILLAIN_SOCKET to override the socket path. The shell configures its
Wayland platform and layer-shell plugin path from the executable location;
QT_PLUGIN_PATH may be supplied for an additional development path.

## Interaction

The bar's menu action starts the overview. Villain can also bind a standalone
modifier release to:

    [[bind]]
    keys = "MOD"
    dispatch = "exec"
    args = ["knave-shell", "overview"]

The overview can focus a workspace or window through Villain. Escape closes
the overview without changing focus. Behavior that depends on focus, input,
layer-shell, GPU presentation, or an installed prefix requires a live test;
unit tests and compilation are not sufficient.

## Configuration and migration

Knave owns the target configuration at ~/.config/knave/config.toml. This
repository must not introduce a competing shell configuration file. Current
shell settings are supplied by runtime contracts and environment variables
while the Knave settings API is being implemented.

## Documentation

- [Architecture index](docs/architecture/README.md)
- [Component boundaries](docs/architecture/component-boundaries.md)
- [Performance policy](docs/architecture/performance.md)
- [Change-impact checklist](docs/architecture/change-impact.md)
- [Agent instructions](AGENTS.md)

Use Conventional Commits and describe cross-repository contract, compatibility,
verification, rollout, and rollback impact in pull requests.
