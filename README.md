# Binary formats evaluation for sourmash signatures

[Main discussion](https://github.com/dib-lab/sourmash/issues/1262)

## Easy ones (serde support)

- [x] bincode
- [x] cbor
- [x] flexbuffers
- [x] msgpack
- [x] postcard

## To check

- [ ] avro (need to write schema)
- [ ] arrow?
- [ ] [Tree-buf](https://github.com/That3Percent/tree-buf)
- [ ] [cap'n proto](https://capnproto.org/)
- [ ] [rkyv](https://github.com/djkoloski/rkyv)
- [ ] [bitmagic](https://github.com/dib-lab/sourmash/pull/1221) (not really a format, but for the "hashes-as-compressed vector" idea.
