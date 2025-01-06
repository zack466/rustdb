# RustDB

A simple key-value store a la Redis, implemented in pure Rust.
Uses [linear hashing](https://en.wikipedia.org/wiki/Linear_hashing) for a dynamically-resizeable hash table.
Supports serialization to disk along with a websocket interface.

Note: this was written mainly for fun and learning, not for production use!

## Usage

Start the server using.
```
cargo run --bin server
```

Then, start the client using.
```
cargo run --bin client
```

On the client side, you can use the following commands:

- `GET <key>`: get the value of a key
- `SET <key> <value>`: set the value of a key
- `INC <key>`: increment the value of a key
- `DEC <key>`: decrement the value of a key
- `SAVE`: save the database to disk
- `EXIT`: exit the client
- `HELP`: show this help message

Keys are allowed to be arbitrary strings, and values are allowed to be strings, integers, or arrays of values.

Example:
```
> hello
WORLD
> get foo
(nil)
> set bar 10
OK
> inc bar
OK
> get bar
(integer) 11
> save
OK
```