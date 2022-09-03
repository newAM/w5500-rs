# w5500-sntp

SNTP client for the [Wiznet W5500] SPI internet offload chip.

## Limitations

* No support for message digests

## Feature Flags

All features are disabled by default.

* `chrono`: Enable conversion to `chrono::naive::NaiveDateTime`.
* `defmt`: Enable logging with `defmt`.
* `eh0`: Passthrough to [`w5500-hl`].
* `eh1`: Passthrough to [`w5500-hl`].
* `log`: Enable logging with `log`.
* `num-rational`: Enable conversion to `num_rational::Ratio`.
* `std`: Passthrough to [`w5500-hl`].
* `time`: Enable conversion to `time::PrimitiveDateTime`.
* `ufmt`: Enable formatting types with `ufmt`.

## Reference Documentation

* [RFC 4330](https://www.rfc-editor.org/rfc/rfc4330.html)

[`w5500-hl`]: https://crates.io/crates/w5500-hl
[Wiznet W5500]: https://www.wiznet.io/product-item/w5500/
