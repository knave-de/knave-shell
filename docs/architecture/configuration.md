# Shell configuration boundary

The canonical configuration is
`~/.config/knave/config.toml`, owned and written through Knave settings.

Shell code consumes typed configuration projections. It must not create a
competing shell configuration file, parse Villain's legacy file, or persist
runtime state as user settings. Knave owns migration and supplies the
compositor/session/shell projections through versioned APIs.

Adding a shell setting requires a schema entry, validation/default behavior,
serialization and migration review in Knave, and tests for unknown and legacy
data preservation. The shell must behave safely when the desktop API is
unavailable, incompatible, or missing an optional projection.
