//! ChaseSeq: A pointer chasing benchmark library.
//!
//! A Rust port of [MemoryLatencyTest](https://github.com/ChipsandCheese/MemoryLatencyTest)'s pointer chasing benchmark.[¹](#footnote-1)
//!
//! # Quick Start
//!
//! Add this to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! chase_seq = "^0.2"
//! ```
//!
//! Use it in your code:
//!
//! ```no_run
//! use chase_seq::{ChaseSeqBuilder, KB};
//!
//! // `size` is in KiB
//! let chase_seq = ChaseSeqBuilder::default().size(64 * KB)?.build();
//!
//! let results = chase_seq.chase(10)?;
//!
//! for (i, result) in results.iter().enumerate() {
//!    println!("Iteration {}: {:.3} ns", i, result);
//! }
//! # Ok::<(), Box<dyn core::error::Error>>(())
//! ```
//!
//!
//! ---
//! <a name="footnote-1"></a>
//! ¹ the assembly parts are not ported.

mod builder;
mod errors;
#[cfg(all(miri, test))]
mod miri;
#[cfg(test)]
mod tests;

#[cfg(not(miri))]
use std::sync::OnceLock;
use std::{
    hint,
    sync::atomic::{Ordering, fence},
};

#[cfg(not(miri))]
use quanta::Clock;
use rand_core::Rng;
use rand_seeder::Seeder;
use rand_xoshiro::Xoshiro256PlusPlus;

pub use crate::{
    builder::ChaseSeqBuilder,
    errors::{ChaseSeqBuilderError, ChaseSeqError, CommonError},
};

/// Number of iterations for scaling.
#[cfg(all(target_pointer_width = "64", not(miri)))]
const ITER: usize = 100_000_000;
#[cfg(all(target_pointer_width = "32", not(miri)))]
const ITER: usize = 50_000_000;

/// From bytes, divide to get KiB.
/// From KiB, multiply to get bytes
pub const KB: usize = 1024;
/// From bytes, divide to get MiB.
/// From MiB, multiply to get bytes
pub const MB: usize = 1024 * KB;

/// Size of pointer in bytes
pub const PTR_SIZE: usize = size_of::<usize>();

#[cfg(not(miri))]
static CLOCK: OnceLock<Clock> = OnceLock::new();

/// `ChaseSeq` provides pointer chasing benchmark functionality.
#[derive(Clone, Copy)]
pub struct ChaseSeq {
    size: usize,
    num_iter: usize,
    seed: &'static str,
    fence: bool,
}

impl ChaseSeq {
    /// Set the size in KiB of memory region to chase.
    pub fn set_size(&mut self, size: usize) -> Result<(), ChaseSeqError> {
        if size == 0 {
            return Err(ChaseSeqError::CommonError(CommonError::SizeIsZero));
        };
        self.size = size
            .checked_mul(KB)
            .ok_or(ChaseSeqError::CommonError(CommonError::SizeTooLarge))?
            / PTR_SIZE;
        self.num_iter = scale_iterations(size);
        Ok(())
    }

    /// Get the size in KiB of memory region to chase.
    pub fn size(&self) -> usize {
        self.size / KB * PTR_SIZE
    }

    /// Set whether to use CPU fence on each pointer dereference.
    pub fn set_fence(&mut self, fence: bool) {
        self.fence = fence;
    }

    /// Get whether CPU fence is used on each pointer dereference.
    pub fn fence(&self) -> bool {
        self.fence
    }

    /// Set the seed string for random number generator.
    pub fn set_seed(&mut self, seed: &'static str) {
        // Zero-length seed is allowed.
        self.seed = seed;
    }

    /// Get the seed string for random number generator.
    pub fn seed(&self) -> &'static str {
        self.seed
    }

    /// Perform pointer chasing benchmark.
    /// `test_iterations` is the number of test iterations to perform.
    /// Returns a boxed slice of `f64` representing the time taken (in nanoseconds) per pointer chase for each iteration.
    pub fn chase(&self, test_iterations: usize) -> Result<Box<[f64]>, ChaseSeqError> {
        _ = test_iterations
            .checked_mul(size_of::<f64>())
            .ok_or(ChaseSeqError::TestIterationsTooLarge)?;
        let mut results = Box::new_uninit_slice(test_iterations);
        let mut rng: Xoshiro256PlusPlus = Seeder::from(self.seed).into_rng();
        for slot in results.iter_mut() {
            let mut data: Box<[usize]> = (0..self.size).collect();
            shuffle(&mut data, &mut rng);
            let base = data.as_ptr();
            // the code is duplicated to enable inlining
            let result = if self.fence {
                self._chase::<true>(base)
            } else {
                self._chase::<false>(base)
            };
            slot.write(result);
        }
        Ok(unsafe { results.assume_init() }) // safe: fully initialize
    }

    #[inline(always)]
    fn _chase<const FENCE: bool>(&self, base: *const usize) -> f64 {
        #[cfg(not(miri))]
        let t0 = CLOCK.get_or_init(Clock::new).now();
        #[cfg(miri)]
        let t0 = std::time::Instant::now();
        let mut p = 0;
        for _ in 0..self.num_iter {
            if FENCE {
                self._chase_fenced(base, &mut p);
            } else {
                self._chase_unfenced(base, &mut p);
            }
        }
        hint::black_box(p);
        t0.elapsed().as_secs_f64() * 1e9 / self.num_iter as f64
    }

    #[inline(always)]
    fn _chase_fenced(&self, base: *const usize, p: &mut usize) {
        *p = unsafe { base.add(*p).read_volatile() };
        fence(Ordering::SeqCst);
    }

    #[inline(always)]
    fn _chase_unfenced(&self, base: *const usize, p: &mut usize) {
        *p = unsafe { *base.add(*p) };
    }
}

impl Default for ChaseSeq {
    /// Create a default `ChaseSeq`.
    /// The default `size` is 2 KiB.
    /// The default `seed` is `"chase_seq_benchmark"`.
    /// The default `fence` is `true`.
    fn default() -> Self {
        let size = 2 * KB;
        Self {
            size: size / PTR_SIZE,
            num_iter: scale_iterations(size),
            seed: "chase_seq_benchmark",
            fence: true,
        }
    }
}

#[cfg(target_pointer_width = "64")]
fn rng_usize(rng: &mut impl Rng) -> usize {
    rng.next_u64() as usize
}

#[cfg(target_pointer_width = "32")]
fn rng_usize(rng: &mut impl Rng) -> usize {
    rng.next_u32() as usize
}

// https://github.com/ChipsandCheese/MemoryLatencyTest/blob/a93bc1ba76dfe8cee76707e99f20ee13b5b485aa/src/memorylatency.c#L139-L146
fn shuffle(data: &mut [usize], mut rng: &mut impl Rng) {
    let n = data.len();
    for iter in (1..n).rev() {
        let j = if iter == 1 {
            0
        } else {
            let max_acceptable = usize::MAX - usize::MAX % iter;
            let mut x = rng_usize(&mut rng);
            while x >= max_acceptable {
                x = rng_usize(&mut rng);
            }
            x % iter
        };
        data.swap(iter, j);
    }
}

// https://github.com/ChipsandCheese/MemoryLatencyTest/blob/a93bc1ba76dfe8cee76707e99f20ee13b5b485aa/src/memorylatency.c#L110-L112
#[cfg(not(miri))]
fn scale_iterations(size: usize) -> usize {
    let size_f = size as f64;
    let iterations_f = ITER as f64;
    let scaled = 10.0 * iterations_f / size_f.powf(0.25);
    scaled.round() as usize
}

#[cfg(miri)]
fn scale_iterations(_: usize) -> usize {
    5
}
