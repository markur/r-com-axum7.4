# Yew Frontend

## Prerequisites
- Rust toolchain
- Trunk (`cargo install trunk`)
- wasm32 target (`rustup target add wasm32-unknown-unknown`)

## Running Locally
```sh
# Install Trunk if you haven't
cargo install trunk

# Add the wasm32 target if you haven't
rustup target add wasm32-unknown-unknown

# Start the dev server
trunk serve --open
```

This will build the app and open it in your browser. 