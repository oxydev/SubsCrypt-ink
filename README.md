# SubsCrypt-ink!

[![Rust](https://github.com/oxydev/SubsCrypt-ink/actions/workflows/rust.yml/badge.svg)](https://github.com/oxydev/SubsCrypt-ink/actions/workflows/rust.yml)

ink! implementation of SubsCrypt. for more information please visit [online docs](https://oxydev.github.io/SubsCrypt-docs/#/)

## Testing

First of all you need to clone the repository, run:

```bash
git clone https://github.com/oxydev/SubsCrypt-ink
cd SubsCrypt-ink
```

Then, you can run the tests with this line of code:

```bash
cargo +nightly test
```

## Building

To build the WASM of your contract and metadata, you can clone and change directory to the ink project of SubsCrypt and then you have to run this line:

```bash
cargo +nightly contract build
```

This command will take some minutes and the output will be something like this:

```bash
Original wasm size: 99.1K, Optimized: 68.0K

Your contract artifacts are ready. You can find them in:
/yourDirectory/target/ink

  - SubsCrypt.contract (code + metadata)
  - SubsCrypt.wasm (the contract's code)
  - metadata.json (the contract's metadata)
```

You can also use the pre-built version of our code and access to the WASM and metadata files, [here](https://github.com/oxydev/SubsCrypt-ink/blob/main/deploy/SubsCrypt.wasm) and [here](https://github.com/oxydev/SubsCrypt-ink/blob/main/deploy/metadata.json).
