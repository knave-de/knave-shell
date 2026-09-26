# Shell versioning and compatibility

The shell participates in three independently tracked domains:

- the Knave public desktop API;
- the private shell/UI protocol; and
- the shell's binary and library versions.

Protocol messages need a compatibility range and an additive-change rule.
Breaking changes require coordinated sender and receiver updates, an adapter or
migration where practical, and an explicit rollback path. Do not infer
compatibility from a successful shell-only build.

The supported shell/Villain/Knave combination belongs in the cross-repository
release notes or compatibility matrix. The removed Qt/C ABI implementation is
not a supported compatibility target; runtime compatibility is defined by the
Rust shell binary, the public desktop API, and the layer-shell contract.
