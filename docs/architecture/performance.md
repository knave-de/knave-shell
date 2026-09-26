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

The current Rust/wgpu shell keeps idle work, app discovery, and image memory
bounded:

- the snapshot worker uses one-entry command/update channels, a 500ms successful
  interval, and reconnect backoff capped at five seconds;
- preview capture uses one worker only while overview is running, requests at
  most 10 workspace captures per changed snapshot, and decodes exact 320x180
  images off the Wayland callback;
- the app catalog uses one overview-lifetime worker, scans at most 8,192
  directory entries, 2,048 desktop files, and 8 MiB of file data, and retains at
  most 512 launchable entries; each source file is limited to 64 KiB;
- icon requests are coalesced into one latest-value slot; at most 48 icons are
  decoded/cached as 48x48 RGBA images, from source files no larger than 1 MiB;
  PNG dimensions are capped at 512x512 and SVG output is rasterized directly to
  48x48;
- the overview shows at most 10 workspaces, 12 tiled windows, 24 minimized
  windows, and 12 app-search results; search text is capped at 64 characters;
- one event-loop timer refreshes the centered local date/time once per minute;
  it creates no thread and no periodic redraw when the displayed minute has not
  changed;
- pointer and keyboard actions share one worker with a one-entry queue; a full
  queue logs and drops the action rather than spawning parallel work;
- the scene and render list rebuild only after a snapshot, input, clock, or
  surface-size change; unchanged frames are not submitted continuously; and
- the renderer reuses one image vertex buffer and bounds cached textures to 64
  entries and 4 MiB of source pixels.

Preview decoding rejects malformed PNG data, base64 payloads over 512 KiB, and
decoded images larger than 320x180 (230,400 RGBA bytes per preview). Ten current
previews therefore use about 2.2 MiB of CPU pixel data. At the same time, the
48-icon CPU limit is about 0.42 MiB; the GPU cache can hold those icons plus the
workspace previews and logo within its 4 MiB source-pixel cap. These figures
exclude allocator and graphics-driver overhead.

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
