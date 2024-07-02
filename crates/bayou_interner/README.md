A string interner.

A string interner is a data structure commonly used in compilers and other contexts that need to
cheaply store and compare many often identical strings. "Interning" a string returns a pointer (or in
this implementation, an ID) that is cheap to copy and to perform string equality checks on. This is
achieved by deduplicating strings using an internal hash table.

This string interner also stores all strings in a single bump-allocated buffer, courtesy of
[bumpalo](https://crates.io/crates/bumpalo), avoiding excessive allocation.

I decided to represent interned strings with a 32-bit ID instead of a reference to avoid introducing lifetimes.
This does mean that accessing the underlying string requires calling a method on the interner, but this is a
single array lookup.

# Example
```rust
use bayou_interner::Interner;

let interner = Interner::new();

let hello = interner.intern("hello");
let hello2 = interner.intern("hello");
let world = interner.intern("world");

// Interned strings can be compared cheaply.
assert_ne!(hello, hello2);
assert_ne!(hello, world);

// Getting the associated string for an interned string.
assert_eq!(interner.get_str(hello), Some("hello"));
```