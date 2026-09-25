# Shell configuration boundary

The canonical configuration is
`~/.config/knave/config.toml`, owned and written through Knave settings.

Shell code consumes typed configuration projections. It must not create a
competing shell configuration file or persist runtime state as user settings.
A legacy reader is permitted only as a named migration adapter with explicit
precedence and removal criteria.

Adding a shell setting requires a schema entry, validation/default behavior,
serialization and migration review in Knave, and tests for unknown and legacy
data preservation. The shell must behave safely when an optional setting is
absent or comes from an older schema.
