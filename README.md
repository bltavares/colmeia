# colmeia

----
Hive (in portuguese). Attempt to make an interop layer to connect to [dat](https://github.com/datrs/) on [hyperswarm](https://github.com/hyperswarm) and legacy infra as well.

- [x] `colmeia-mdns` - compat with `npm i -g dat`
  - [x] `Locator`: stream to find dat members in the network
  - [x] `Announcer`: stream that announces a dat in the network
  - [x] `Mdns`: announces and find dat in the network
  - [ ] Add support to hyperswarm dns
- [ ] `colmeia-dht`: Interop with hypwerswarm dht infrastructure
- [ ] `colmeia-dns`: DNS locator to resolve a hostname into a dat hash
- [ ] `colmeia-resolve`: converts a `.well-known/dat`  or DNS TXT [into a hash](https://beakerbrowser.com/docs/guides/use-a-domain-name-with-dat) 
- [ ] `colmeia-network`: Network pool manager, based on mdns and dht members
- [ ] :warning: `colmeia-proto` **wip**: Parses DAT wire protocol

## Utils

### Find a local peer

```sh
RUST_LOG=debug cargo run --bin colmeia-mdns -- dat://460f04cf12c3b9833e5a0d3dd8eea05eab59dd8c1438a7454afe9630b9b4f8bd
```

### Connect and troubleshoot a peer

```sh
RUST_LOG=debug cargo run --bin colmeia-nc -- 127.0.0.1:3282
```
