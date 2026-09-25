# Shell component boundaries

This repository owns the desktop-facing shell implementation and its UI
composition. It may contain transitional Qt/QML/C++ code and the Rust shell,
renderer, UI, and Wayland work needed to replace it.

The shell:

- consumes typed settings projections and versioned desktop contracts from
  Knave;
- uses explicit adapters for Wayland and compositor integration;
- owns presentation and interaction state; and
- reports lifecycle failures to the session owner.

It does not own compositor layout, focus, window-management, or input policy.
It does not write the canonical user configuration or reach through a public
contract into Villain internals. A shell change that alters a protocol or
lifecycle assumption must update both sides and document compatibility.
