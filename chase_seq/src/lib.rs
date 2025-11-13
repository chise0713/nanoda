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
//! chase_seq = "0.1"
//! ```
//!
//! Use it in your code:
//!
//! ```rust
//! use chase_seq::ChaseSeqBuilder;
//!
//! // `size` is in KiB
//! let chase_seq = ChaseSeqBuilder::default().size(64 * 1024)?.fence(true).build()?;
//!
//! let results = chase_seq.chase(10);
//!
//! for (i, result) in results.iter().enumerate() {
//!    println!("Iteration {}: {:.3} ns", i, result);
//! }
//! ```
//!
//!
//! ---
//! <a name="footnote-1"></a>
//! ¹ the assembly parts are not ported.

mod builder;
mod errors;

use std::{
    hint,
    sync::atomic::{Ordering, fence},
};

use quanta::Clock;
use rand_core::RngCore;
use rand_seeder::Seeder;
use rand_xoshiro::Xoshiro256PlusPlus;

pub use crate::builder::ChaseSeqBuilder;
pub use crate::errors::{ChaseSeqBuilderError, ChaseSeqError};

#[cfg(target_pointer_width = "64")]
const ITER: usize = 100_000_000;
#[cfg(target_pointer_width = "32")]
const ITER: usize = 50_000_000;

pub const KB: usize = 1024;
pub const MB: usize = 1024 * 1024;

#[cfg(target_pointer_width = "64")]
pub const MAX_REGION_SIZE: usize = 4096 * MB;
#[cfg(target_pointer_width = "32")]
pub const MAX_REGION_SIZE: usize = 1024 * MB;

pub const PTR_SIZE: usize = std::mem::size_of::<usize>();

/// ChaseSeq provides pointer chasing benchmark functionality.
pub struct ChaseSeq {
    size: usize,
    num_iter: usize,
    rng: Xoshiro256PlusPlus,
    clock: Clock,
    fence: bool,
}

impl ChaseSeq {
    /// Set the size in KiB of memory region to chase.
    pub fn set_size(&mut self, size: usize) -> Result<(), ChaseSeqError> {
        if size == 0 {
            return Err(ChaseSeqError::SizeIsZero);
        }
        self.size = size * KB / PTR_SIZE;
        self.num_iter = scale_iterations(size);
        Ok(())
    }

    /// Get the size in KiB of memory region to chase.
    pub fn size(&self) -> usize {
        self.size
    }

    /// Set whether to use CPU fence on each pointer dereference.
    pub fn set_fence(&mut self, fence: bool) {
        self.fence = fence;
    }

    /// Get whether CPU fence is used on each pointer dereference.
    pub fn fence(&self) -> bool {
        self.fence
    }

    /// Perform pointer chasing benchmark.
    /// `test_iterations` is the number of test iterations to perform.
    /// Returns a boxed slice of `f64` representing the time taken (in nanoseconds) per pointer chase for each iteration.
    pub fn chase(&self, test_iterations: usize) -> Box<[f64]> {
        let mut results = Box::new_uninit_slice(test_iterations);
        let mut rng = self.rng.clone();
        for slot in results.iter_mut() {
            let mut data: Box<[usize]> = (0..self.size).collect();
            shuffle(&mut data, &mut rng);
            let base = data.as_ptr();
            // the code is duplicated to enable inlining
            let result = if self.fence {
                self.chase_once_fenced(base)
            } else {
                self.chase_once(base)
            };
            slot.write(result);
        }
        unsafe { results.assume_init() } // safe: fully initialize
    }

    #[inline(always)]
    fn chase_once_fenced(&self, base: *const usize) -> f64 {
        let t0 = self.clock.now();
        let mut p = 0;
        for _ in 0..self.num_iter {
            p = unsafe { base.add(p).read_volatile() };
            fence(Ordering::SeqCst);
        }
        hint::black_box(p);
        t0.elapsed().as_secs_f64() * 1e9 / self.num_iter as f64
    }

    #[inline(always)]
    fn chase_once(&self, base: *const usize) -> f64 {
        let t0 = self.clock.now();
        let mut p = 0;
        for _ in 0..self.num_iter {
            p = unsafe { *base.add(p) };
        }
        hint::black_box(p);
        t0.elapsed().as_secs_f64() * 1e9 / self.num_iter as f64
    }
}

impl Default for ChaseSeq {
    /// Create a default `ChaseSeq`.
    /// The default `size` is 2 KiB.
    /// The default `fence` is `true`.
    fn default() -> Self {
        let size = 2 * KB;
        Self {
            size: size / PTR_SIZE,
            num_iter: scale_iterations(size),
            rng: Seeder::from("chase_seq_benchmark").into_rng(),
            clock: Clock::new(),
            fence: true,
        }
    }
}

fn rng_usize(rng: &mut impl RngCore) -> usize {
    #[cfg(target_pointer_width = "64")]
    {
        rng.next_u64() as usize
    }
    #[cfg(target_pointer_width = "32")]
    {
        rng.next_u32() as usize
    }
}

// https://github.com/ChipsandCheese/MemoryLatencyTest/blob/a93bc1ba76dfe8cee76707e99f20ee13b5b485aa/src/memorylatency.c#L139-L146
fn shuffle(data: &mut [usize], mut rng: &mut impl RngCore) {
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
fn scale_iterations(size: usize) -> usize {
    let size_f = size as f64;
    let iterations_f = ITER as f64;
    let scaled = 10.0 * iterations_f / size_f.powf(0.25);
    scaled.round() as usize
}
