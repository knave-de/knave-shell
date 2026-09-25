# Shell build and packaging

The target shell is Rust/Cargo-first: Cargo should build its libraries,
binaries, tests, and workspace targets. The current Qt shell is transitional
and may continue using CMake with Ninja in an isolated build directory.

Build changes must check debug and release artifact paths, generated files,
dependency discovery, staged installation, and runtime lookup paths. Do not
silently install into `/usr/local` or assume a workspace artifact is the
installed artifact.

A shell build passing compilation does not prove Wayland, layer-shell, GPU, or
installed-session startup. Those paths require explicit integration or live
smoke verification and must be reported separately from unit-test results.
