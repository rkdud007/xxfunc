# cargo-xxfunc

`cargo-xxfunc` is a cargo subcommand that provides a set of functions to interact with the xxfunc service.

### build

Builds the function into WACI binary.

```console
cargo xxfunc build
```

```console
cargo xxfunc build --release
```

### deploy

Deploys the function to the xxfunc service.

```console
cargo xxfunc deploy ---url <server-url> --wasm-path <wasm-file-path>
```

It will return the function signature.

### start

Starts the function to be triggered by exex event.

```console
cargo xxfunc start ---url <server-url> --module-name <module-name>
```
