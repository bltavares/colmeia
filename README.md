# colmeia

----
Hive (in portuguese). Attempt to make an interop layer to connect to [dat](https://github.com/datrs/) on [hyperswarm](https://github.com/hyperswarm) and legacy infra as well.

Studies:

- [x] How to query mdns dat repos? Any client already supporting?
  - Looks like we need to have it implemented manually...

----

libp2p test:

```sh
RUST_LOG=debug cargo run --bin libp2p&
RUST_LOG=debug cargo run --bin libp2p&
```

mdns libs bin:

```sh
RUST_LOG=debug cargo run --bin mdns -- 62e3097994f2077e3c3c567ded32ed448384407b5eb7d6fbb090c7c3f57b95eb
```