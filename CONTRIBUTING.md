# Contributing

## README Generation

The README file for each crate is generated with [cargo-readme].

```bash
cargo install cargo-readme
for crate in dhcp dns hl ll regsim; do cargo readme -t ../README.tpl -r "$crate" > "$crate"/README.md; done
```

[cargo-readme]: https://github.com/livioribeiro/cargo-readme
