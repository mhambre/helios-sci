# Helios - System Call Interface

`helios-sci` is the syscall interface layer for Helios. It provides a cleaner boundary between higher-level components and low-level kernel/runtime behavior.

For top-level build scripts, shared targets, and project-wide setup, use the main Helios README:
https://github.com/mhambre/helios

## Build / Run / Debug

From the Helios monorepo root:

```bash
just build sci release
just build sci debug
just gdb sci
```

Direct cargo build:

```bash
cargo +nightly build -p helios-sci --target x86_64-unknown-linux-gnu
```

Run directly after build:

```bash
./target/x86_64-unknown-linux-gnu/debug/helios-sci
```
