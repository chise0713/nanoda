ChaseSeq: A pointer chasing benchmark library.

A Rust port of [MemoryLatencyTest](https://github.com/ChipsandCheese/MemoryLatencyTest)'s pointer chasing benchmark.[¹](#footnote-1)

# Quick Start

Add this to your `Cargo.toml`:

```toml
[dependencies]
chase_seq = "0.2"
```

Use it in your code:

```rust
use chase_seq::{ChaseSeqBuilder, KB};

// `size` is in KiB
let chase_seq = ChaseSeqBuilder::default().size(64 * KB)?.build();

let results = chase_seq.chase(10)?;

for (i, result) in results.iter().enumerate() {
   println!("Iteration {}: {:.3} ns", i, result);
}
```

## License

Licensed under either of

 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

#### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
<!-- copied from smol's [README.md](https://github.com/smol-rs/smol/tree/1532526ed932495c1b64623043104d567e9fb165?tab=readme-ov-file#license) -->

---
<a name="footnote-1"></a>
¹ the assembly parts are not ported.