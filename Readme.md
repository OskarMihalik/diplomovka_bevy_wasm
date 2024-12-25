# how to run
## db
`docker compose up`

## backend
`cargo run -p backend`

## frontend
native:
`cargo run -p bevy_diplomovka`

wasm:
first setup run-wasm by:
1. Create a .cargo/config file containing:
```
[alias]
run-wasm = "run --release --package run-wasm --"
```
2. then just run 
```
cargo run-wasm --package bevy_diplomovka

```
