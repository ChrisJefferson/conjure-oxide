# Experimental Emscripten builds

The minimal supported experiment targets `wasm32-unknown-emscripten`, with the
normal Essence parser, released `minion-sys` 0.1.1 and BatSat. This is a library/build
experiment, not a browser interface. Other Wasm targets are not covered.

Minion remains available unconditionally. SAT is also always available in a
successful library build: select exactly one backend at compile time, `sat-cadical` or `sat-batsat`.
Native defaults select CaDiCaL and enable Z3. To select BatSat, disable defaults
and enable `sat-batsat` on `conjure-cp` / `conjure-cp-rules`. Re-enable `z3`
explicitly if needed on native builds. Selecting neither or both SAT features is
a compile error; consequently `--all-features` is not a supported configuration.

Wasm requires `sat-batsat`. CaDiCaL is a native-only dependency. Cargo features
are additive, so dependencies must agree on the target backend. Parser features
forward the same choice. Procedural macros run on the host and use the parser's
native default backend independently of the Wasm target's selection.

The common `Sat` adaptor API is unchanged. BatSat supports timeout termination
through its callback API; nonzero solver seeds return an explicit unsupported
error rather than being ignored. Default seed zero uses BatSat's defaults.
These features select the library backend. The native CLI/LSP currently retain
their default dependencies. Build the selected library/example, not the entire
native CLI/LSP workspace.

## Toolchain and dependency fix

Install Emscripten (with `emcc` on PATH), Node.js for the smoke check, and the Rust
Wasm target:

```sh
rustup target add wasm32-unknown-emscripten
```

Tree-sitter 0.27.0 incorrectly enables its bare-Wasm libc replacements for
Emscripten. The resulting `fprintf`/`fwrite` definitions conflict with Emscripten
libc. Until an upstream release fixes this, use a separate patched copy of that
crate. From the repository root, for example:

```sh
mkdir -p /tmp/conjure-tree-sitter
curl -fsSL https://static.crates.io/crates/tree-sitter/tree-sitter-0.27.0.crate \
  -o /tmp/conjure-tree-sitter/tree-sitter.crate
tar -xf /tmp/conjure-tree-sitter/tree-sitter.crate -C /tmp/conjure-tree-sitter
patch -d /tmp/conjure-tree-sitter/tree-sitter-0.27.0 -p1 \
  < tools/patches/tree-sitter-emscripten.patch
```

This keeps vendored dependencies and an absolute local path out of the workspace
manifest. The temporary Cargo override below changes Tree-sitter's lockfile entry;
that local-path lockfile change should not be committed. A published consuming
application must supply the patch itself until the upstream fix is released.

## Compile and run a smoke check

```sh
cargo rustc -p conjure-cp-rules --example wasm_minion \
  --no-default-features --features sat-batsat --target wasm32-unknown-emscripten \
  --config 'patch.crates-io.tree-sitter.path="/tmp/conjure-tree-sitter/tree-sitter-0.27.0"' \
  -- -C linker=tools/emscripten-linker.sh \
  -C link-arg=-sDEFAULT_TO_CXX=1 \
  -C link-arg=-sSTACK_SIZE=8388608 \
  -C link-arg=-sALLOW_MEMORY_GROWTH=1
node target/wasm32-unknown-emscripten/debug/examples/wasm_minion.js
```

If `CARGO_TARGET_DIR` is overridden, use that output directory instead of `target`.
Use the Python configured by the Emscripten SDK; set `EMSDK_PYTHON` explicitly if
its launcher picks an incompatible Python installation.

The linker wrapper uses `--whole-archive` for the built-in rules archive, retaining
objects referenced only through inventory constructors. Additional downstream
rule crates would need equivalent retention. Emscripten runs the constructors at
startup; no explicit registry or initialisation call is needed in Rust.

The C++ link setting supplies Minion's runtime. The larger stack supports parser,
rewriter and solver recursion. Memory growth allows Minion to allocate its search
storage rather than abort at the default fixed heap limit. `std::time::Instant`
works through Emscripten, so this path does not need a Wasm timing replacement.

The smoke executable checks registration, invalid Essence diagnostics and all six
solutions of three pairwise-distinct integer variables through both Minion and
SAT, reconstructing the original values from SAT representations. It also runs
natively, before using the temporary Cargo override (or after restoring the registry lock entry):

```sh
cargo run -p conjure-cp-rules --example wasm_minion --no-default-features --features sat-batsat --locked
```

No wasm-bindgen adapter, JavaScript worker, website or deployment
configuration is required for this experiment.
