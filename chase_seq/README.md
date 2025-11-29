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
use chase_seq::ChaseSeqBuilder;

// `size` is in KiB
let chase_seq = ChaseSeqBuilder::default().size(64 * 1024)?.build()?;

let results = chase_seq.chase(10);

for (i, result) in results.iter().enumerate() {
   println!("Iteration {}: {:.3} ns", i, result);
}
```

# License
This project is licensed under either of

- Apache License, Version 2.0, (LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license (LICENSE-MIT or http://opensource.org/licenses/MIT)

at your option.

---
<a name="footnote-1"></a>
¹ the assembly parts are not ported.