# w5500-tls

TLS v1.3 client for the [Wiznet W5500] SPI internet offload chip.

This requires roughly 19k of flash for a `thumbv7em-none-eabi` target
with `-O3`, debug assertions enabled, and the `p256-cm4` feature.
Enabling all logging requires an additional ~40k of flash.

## Warning

⚠️ This is in an early alpha state ⚠️

All the usual security disclaimers apply here, read the license, your hamster
may explode if you use this, don't use this code in production, etc.

Additionally this is not secure from side channel attacks.

* Encryption may occur in-place in the socket buffers, anything with access
  to the physical SPI bus or the SPI device registers can easily intercept
  data.
* To facilitate the ill-advised encryption in-place in the socket buffers
  there is a hacky AES implementation that has little thought put towards
  constant-time evaluation.

## Limitations

At the moment this only supports pre-shared keys.
This will not work for majority of web (HTTPS) applications.

* Requires a local buffer equal to the socket buffer size.
  * TLS record fragmentation makes implementing socket buffer streaming
    impractical.
* Limited cryptography support
  * Cipher: `TLS_AES_128_GCM_SHA256`
  * Key Exchange: `secp256r1`
* Does not support certificate validation
* Does not support client certificates (mutual TLS)
* Does not support early data
* Does not support serving TLS

## Feature Flags

All features are disabled by default.

* `embedded-hal`: Passthrough to [`w5500-hl`].
* `std`: Passthrough to [`w5500-hl`].
* `defmt`: Enable logging with `defmt`. Also a passthrough to [`w5500-hl`].
* `log`: Enable logging with `log`.
* `p256-cm4`: Use a p256 implementation optimized for the Cortex-M4 CPU.

[`w5500-hl`]: https://github.com/newAM/w5500-hl-rs
[Wiznet W5500]: https://www.wiznet.io/product-item/w5500/
