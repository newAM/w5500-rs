# w5500-tls American Fuzzy Lop

Source documentation: <https://rust-fuzz.github.io/book/afl.html>

## Quickstart

```console
$ cd "$REPOSITORY_ROOT"
$ tls/afl/run.sh
... let it run
$ cargo run -p w5500-tls-afl --bin replay -- out/default/crashes/
... enjoy the crashing
```
