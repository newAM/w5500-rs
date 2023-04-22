# w5500-sntp

SNTP client for the [Wiznet W5500] SPI internet offload chip.

## Limitations

* No support for message digests

## Feature Flags

All features are disabled by default.

* `eh0`: Passthrough to [`w5500-hl`].
* `eh1`: Passthrough to [`w5500-hl`].
* `ip_in_core`: Passthrough to [`w5500-hl`].
* `std`: Passthrough to [`w5500-hl`].
* `defmt`: Enable logging with `defmt`. Also a passthrough to [`w5500-hl`].
* `log`: Enable logging with `log`.
* `chrono`: Enable conversion to `chrono::naive::NaiveDateTime`.
* `time`: Enable conversion to `time::PrimitiveDateTime`.
* `num-rational`: Enable conversion to `num_rational::Ratio`.

## Reference Documentation

* [RFC 4330](https://www.rfc-editor.org/rfc/rfc4330.html)

[`w5500-hl`]: https://crates.io/crates/w5500-hl
[Wiznet W5500]: https://www.wiznet.io/product-item/w5500/
