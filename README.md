# datablockrs (WIP)
A basic implementation of a notion database using Rust, SQLite and WebAssembly Components.

## Usage

```
cargo build --release --target wasm32-wasip2
npx jco transpile target/wasm32-wasip2/release/datablockrs.wasm -o web/wasm-bindings

cd web/
npm run dev
```
