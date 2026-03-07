# docs preview wasm

Build the docs preview binary and copy it into the static docs assets:

```bash
cargo build --manifest-path docs/preview-wasm/Cargo.toml --target wasm32-wasip1 --release
cp docs/preview-wasm/target/wasm32-wasip1/release/artbox-docs-preview.wasm docs/public/wasm/artbox-preview.wasm
```
