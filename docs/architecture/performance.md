# Shell performance and resource usage

Shell performance includes UI responsiveness, event-loop wakeups, IPC traffic,
Wayland subscriptions, scene updates, rendering work, and background lifecycle
tasks.

Before adding a timer, watcher, subscription, cache, buffer, task, or thread:

- define its owner, lifetime, cancellation, and cleanup behavior;
- bound event frequency, queue size, cache growth, and concurrency;
- avoid polling, duplicate subscriptions, and unbounded model or preview
  updates; and
- test disconnect, reload, shutdown, and compositor-unavailable paths.

Measure CPU, resident memory, threads, file descriptors, wakeups, and relevant
latency under idle, normal, and stress workloads. A shell that renders correctly
or passes unit tests can still wake continuously or retain GPU/IPC resources.

Record a baseline and expected delta for performance-sensitive changes. If no
baseline exists, establish one before declaring the change complete. Report live
Wayland, GPU, and installed-session measurements separately from build and unit
test results.

## Scheduling

The runtime uses SCTK's calloop integration and an event-driven wake channel.
The Wayland loop blocks when there is no compositor event or worker update.
A snapshot worker polls the desktop contract at the existing 500ms interval but
signals the UI only when the snapshot generation changes. Preview completion and
input changes request one redraw; a frame callback is not rescheduled after an
unchanged frame. The wake channel has one slot, and the runtime tracks one
pending frame callback.

## Current shell bounds

The current Rust/wgpu shell keeps the interactive overview bounded:

- snapshot refreshes use one worker, one-entry command and update channels, a
  500ms successful interval, and reconnect backoff capped at five seconds;
- overview preview capture uses one worker only while the overview is running,
  one command slot, and one latest-value update slot;
- preview capture considers at most 10 workspaces per changed snapshot and asks
  Villain for exactly 320x180 images; it does not poll or decode per frame;
- overview rendering considers at most 32 windows from the active workspace;
- the overview uses a fixed four-column card layout and caps each window label at
  48 Unicode scalar values;
- local search caps input at 64 Unicode scalar values and results at 12 entries; and
- pointer and keyboard actions share one worker with a one-entry queue, so input
  bursts are dropped with a diagnostic instead of creating parallel work; and
- the renderer caches the scene and render list, rebuilding them only after a
  snapshot or surface-size change rather than on every frame callback; and
- the renderer keeps at most 16 decoded preview textures and reuses one image
  vertex buffer, uploading a texture only when its immutable image source changes.

Preview decoding requires the requested dimensions, rejects malformed PNG data,
rejects base64 payloads over 512 KiB, and caps decoded pixels at 320x180
(230,400 RGBA bytes per image). Failed or
unavailable captures leave the workspace card's fallback panel in place; they do
not block frame rendering. Ten current previews therefore have a bounded CPU
pixel payload of about 2.2 MiB, while the 16-entry GPU cache is bounded at about
3.7 MiB before driver overhead.

## Nested baseline

A release nested-session sample on 2026-09-26 used Winit, the bar role, no
overview service, and six one-second process samples after two seconds of
startup. The sampled process-lifetime CPU values settled from 2.4% to 0.8% for
Villain and from 8.0% to 2.4% for Knave Shell; Knave stayed at 0.0%. RSS stayed
around 121 MiB for Villain, 189 MiB for Knave Shell, and 3 MiB for Knave.
The process counts were 9, 38, and 1 thread respectively. The run shut down
without project child processes remaining. These are host-specific nested
baselines, not acceptance thresholds; direct TTY/DRM/GPU behavior and visual
latency remain unverified.
