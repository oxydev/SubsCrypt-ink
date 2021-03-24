# SubsCrypt-ink!

[![Rust](https://github.com/oxydev/SubsCrypt-ink/actions/workflows/rust.yml/badge.svg)](https://github.com/oxydev/SubsCrypt-ink/actions/workflows/rust.yml)

ink! implementation of SubsCrypt. for more information please visit [online docs](https://oxydev.github.io/SubsCrypt-docs/#/)


## Testing:

First of all you need to clone the repository, run:

```bash
git clone https://github.com/oxydev/SubsCrypt-ink
cd SubsCrypt-ink
```

Then, you can run the tests with this line of code:

```bash
cargo +nightly test
```


## Building:

To build the wasm of your contract you can clone and change directory to the ink project of SubsCrypt and then you have to run this line:

```bash
cargo +nightly contract build
```

This command will take some minutes and the output will be sth like this:

```bash
Your contract is ready. You can find it here:
/root/test/SubsCrypt-ink/target/SubsCrypt.wasm
```


To build the metadata json of contract you can clone and change directory to the ink project of SubsCrypt and then you have to run this line:

```bash 
cargo +nightly contract generate-metadata
```

You can also use the pre-built version of our code and access to the wasm and metadata files, [here](https://github.com/oxydev/SubsCrypt-ink/blob/main/deploy/SubsCrypt.wasm) and [here](https://github.com/oxydev/SubsCrypt-ink/blob/main/deploy/metadata.json).
