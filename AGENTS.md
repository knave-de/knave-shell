# Knave Shell Agent Instructions

Knave Shell is the desktop-facing shell for the Knave Desktop Environment. It
currently contains Qt/QML UI, C++ adapters, a Rust core, and a Qt Wayland
layer-shell plugin. The long-term Rust/wgpu shell is a migration, not a reason
to silently rewrite this repository.

## Startup and investigation

Before editing:

1. Read this file and the umbrella `knave/AGENTS.md` when working on a
   cross-repository change.
2. Read relevant README, architecture, API, and build documentation when it
   exists.
3. Inspect the complete owning module and search all of its consumers.
4. Check `git status` and preserve unrelated changes.
5. Classify whether the change affects UI, C ABI, Rust core, Wayland protocol,
   compositor IPC, configuration, build, packaging, or runtime behavior.
6. Write a short impact summary before changing a shared contract.

Do not implement a local-looking change until its cross-component effects have
been checked.

## Ownership and contracts

Knave owns the user-facing configuration and settings model. This repository
consumes Knave-owned settings only through typed APIs and public desktop
contracts. It must not introduce another user configuration source.

Villain remains the source of truth for workspace, window, focus, and preview
state. The shell consumes typed desktop contracts and must not duplicate
compositor policy.

Treat these as contracts:

- the Rust core C ABI;
- the Villain IPC payloads;
- shell roles and layer-shell behavior;
- workspace/window model fields;
- plugin discovery and installation paths;
- configuration and runtime environment variables.

Changing any of them requires consumer search, version/migration analysis,
tests, and documentation updates where user behavior changes.

## Build policy

The Rust core is built with Cargo. The current UI and layer-shell plugin are
built with CMake and Qt. Do not claim that the shell is Cargo-only until the
Qt implementation has actually been replaced.

Use Ninja for transitional CMake builds. Keep debug and release artifacts
correctly separated, and verify that the Rust static-library path matches the
selected build profile. Do not install into `/usr/local` implicitly.

After build changes, update `README.md` with only the commands and behavior
that actually exist.

## Code quality

Keep Qt/QML UI, C++ adapters, Rust core, Wayland integration, and compositor
transport in explicit modules. Do not mix rendering, transport, configuration,
and UI policy in one class or function.

Use typed errors and honest unavailable/disconnected states. Do not fabricate
empty successful data when Villain is unavailable. Preserve reconnect behavior.

Review lifecycle and ownership carefully, especially for QML objects, FFI
handles, image providers, layer surfaces, overview single-instance sockets,
and spawned processes.

Comments should explain invariants or non-obvious reasons. Do not narrate
obvious C++, Rust, or QML code. Keep comments short.

## Performance and resource usage

Review resource behavior before adding QML timers, file or socket watchers,
IPC subscriptions, image providers, background tasks, threads, caches, or
parallel work.

- Give each activity an explicit owner, lifetime, cancellation path, and
  cleanup path for reload, disconnect, and shutdown.
- Bound subscriptions, model updates, preview/image caches, queues, retries,
  and concurrency. Do not create one worker or watcher per event without a
  measured bound.
- Prefer event-driven notifications, debouncing, batching, and backoff over
  polling or continuously waking event handlers.
- Measure CPU, resident memory, threads, file descriptors, and wakeups under
  idle, normal, and stress workloads when a change can affect them.

Compilation and functional tests do not establish acceptable shell performance.

## Documentation

Keep README instructions short and executable. Add architecture or migration
documentation only when a boundary or public contract changes. Do not write
long documentation for routine refactors or repeat source code in prose.

## Git and GitHub

- Preserve unrelated dirty changes.
- Create a new focused branch for every task.
- Use Conventional Commits.
- Commit every coherent implementation slice.
- Link dependent cross-repository pull requests.
- Describe affected contracts, consumers, compatibility, verification,
  rollout, and rollback in cross-repository pull requests.
- Always squash and merge.

## Verification

For Rust changes, run:

```console
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

For native/UI changes, configure and build with Ninja, then run the relevant
Wayland/QML smoke tests. A successful compile does not prove layer-shell,
plugin discovery, focus restoration, GPU behavior, or installed-binary startup.

Before completion, inspect the final diff, run `git diff --check`, check
generated files, and report anything not tested.
