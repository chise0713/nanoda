# nanoda

Hey there! `nanoda` is your go-to Rust tool for benchmarking memory latency.<br>
The name's inspired by the adorable catchphrase "nanoda(なのだ)" from **Zundamon (ずんだもん)** in **VoiceVox** – because who doesn't love a cute mascot? 🎉

## What's It Do?

- Zips through memory with unsafe boxed slices for speedy allocation.
- Uses clever permutations (tuned PRNG for your pointer size) to chase data.
- Times each run in nanoseconds with `quanta::Clock`.
- Tweak memory size (`-n` in MiB) and iterations (`-i`) to your heart's content.
- Peek at build deets with the `-b` flag for reproducible fun.

## Get It Running

Grab it from crates.io or build from source:

```bash
# Install the binary
cargo install nanoda

# Or clone and build
git clone https://github.com/chise0713/nanoda.git
cd nanoda
cargo build --release
```

The `chase_seq` sub-crate is also on [crates.io](https://crates.io/crates/chase_seq) for your coding adventures!

## How to Use

Fire it up like this:

```bash
nanoda -n 0.01 -i 3
```

Or run with the build info:

```bash
nanoda -bn 0.01 -i 3
```

Sample output:

```
nanoda version:      v0.2.0 (cargo), v0.2.0 (git)
build target:        x86_64-unknown-linux-gnu
commit sha2 hash:    abcdef1234567890abcdef1234567890abcdef12
build timestamp:     1970-01-01T00:00:00.000000000Z
builder logs url:    unknown
only same binary gives comparable benchmark, so please note the above info and checksum.
memory size:         1 KiB, test iterations 3

results:
min = 0.000 ns, max = 0.000 ns, avg = 0.000 ns
```

## License

The command-line-interface `nanoda` is licensed under **GPL-3.0-or-later** – share the love! ❤️

And the sub-crate `chase_seq` is licensed under **MIT OR Apache-2.0** – pick your favorite! 🎉

---

Have a blast benchmarking with `nanoda` – remember, it's **nanoda**! 🎵

> Crafted with help from GitHub Copilot and AI pals.