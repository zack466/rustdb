# RustDB

A simple key-value store a la Redis, implemented in pure Rust.
Uses [linear hashing](https://en.wikipedia.org/wiki/Linear_hashing) for a dynamically-resizeable hash table.
Supports serialization to disk along with a websocket interface.

Note: this was written mainly for fun and learning, not for production use!