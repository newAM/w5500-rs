# Contributing

## README Generation
The README file is generated with [cargo-readme], run this if you change the docstring in `src/lib.rs`.

```bash
cargo install cargo-readme
cargo readme > README.md
```

[cargo-readme]: https://github.com/livioribeiro/cargo-readme

## Documentation
The documentation uses an unstable rustdoc feature, this is normally enabled only when the documentation is being built by [docs.rs](https://docs.rs).

To build the documentation locally use this command:

```bash
RUSTDOCFLAGS='--cfg docsrs' cargo +nightly rustdoc --all-features
```

## Unit tests
Unit tests are required for all new features.

To test this crate run with the `--all-features` flag:

```bash
cargo test --all-features
```
