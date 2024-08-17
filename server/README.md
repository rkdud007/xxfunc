# xxfunc server

This is public endpoint of xxfunc service.

## Run

```sh
RUST_LOG=info cargo run -p server
```

## Endpoints

- deploy

```sh
curl --location '127.0.0.1:3000/deploy' \
--form 'module=@"/Users/piapark/Documents/GitHub/reth-exex-examples-my/wasm_exex/target/wasm32-unknown-unknown/debug/wasm-exex.wasm"'
```

- start

```sh
curl --location '127.0.0.1:3000/start' \
--header 'Content-Type: application/json' \
--data '{
"module": "wasm-exex"
}'
```
