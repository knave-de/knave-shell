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

## Current shell bounds

The current Rust/wgpu shell keeps the interactive overview bounded:

- snapshot refreshes use one worker, one-entry command and update channels, a
  500ms successful interval, and reconnect backoff capped at five seconds;
- overview rendering considers at most 32 windows from the active workspace;
- the overview uses a fixed four-column card layout and caps each window label at
  48 Unicode scalar values; and
- pointer and keyboard actions share one worker with a one-entry queue, so input
  bursts are dropped with a diagnostic instead of creating parallel work.

Window image previews and search are not fetched or rendered by this slice. Their
future implementation must define explicit decode, cache, refresh, and memory
limits before being enabled.
