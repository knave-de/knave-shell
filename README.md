# Knave Shell

Knave Shell is the Rust/wgpu user-facing shell for the independent Knave
Desktop Environment. It owns presentation and interaction surfaces while
Villain owns compositor policy, window state, focus, and input dispatch.
Winit is not a host-desktop integration: it is only a nested development
backend. The direct Wayland layer-shell path is the runtime target.

## Runtime

    knave-shell bar
    knave-shell overview

The bar is a top layer with a 36-pixel exclusive zone. The overview is an
on-demand exclusive overlay; Escape closes it and number keys 1-9/0 focus the
corresponding workspace before closing it. Typing opens a bounded search over
windows, workspaces, and close; Up/Down changes selection and Enter activates it.
The bar exposes workspace hit
targets for pointer activation; overview cards focus normal windows or restore
minimized ones, and clicking its background closes it. Both consume Knave's
versioned
desktop snapshot contract and keep IPC off the Wayland frame thread.

## Build

    cargo fmt --all -- --check
    cargo test --workspace
    cargo clippy --workspace --all-targets -- -D warnings
    cargo build --workspace --release --locked

## Desktop contract

The shell reads desktop state from Knave's JSON-lines API through
knave-desktop-api:

    $XDG_RUNTIME_DIR/knave/desktop-$WAYLAND_DISPLAY.sock

KNAVE_SOCKET overrides the derived path for isolated tests. The client uses one
bounded worker, a one-entry snapshot channel, a 500ms successful refresh
interval, and exponential reconnect backoff capped at five seconds. It never
blocks the Wayland frame callback on desktop IPC.

Input actions use a separate one-entry bounded queue and one worker. A full
queue drops an action with an explicit diagnostic instead of creating threads.

The shell does not own persistent settings. It receives the compositor's
workspace/window state from Villain through Knave's public contract and sends
user actions back through that same contract.

## Install

    scripts/install.sh --user
    scripts/install.sh --system
    scripts/install.sh --prefix "$PWD/stage"

--user installs to ~/.local/bin and --system installs to /usr/local/bin,
requesting sudo only for the target prefix. --prefix is explicit and useful for
packaging. The installer also writes the README below the selected prefix.

## Migration status

The Rust/wgpu workspace is the sole shell implementation. It draws bounded
rectangle and bitmap-text commands through the direct Wayland layer-shell
runtime. Pointer activation is limited to workspace targets and overview
dismissal. Remaining product work is workspace image previews, richer text and image
primitives, packaging integration, and live direct-TTY/GPU coverage; none of
these depend on restoring the removed Qt/CMake path.

## Workspace

- knave-ui: renderer-independent scene and interaction primitives;
- knave-renderer: render-list, bounded bitmap-text painter, and wgpu boundary;
  and
- knave-wayland: direct layer-shell client and bounded desktop-state bridge.
