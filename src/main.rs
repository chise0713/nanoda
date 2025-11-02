use std::{
    env, hint,
    process::ExitCode,
    ptr,
    sync::atomic::{Ordering, compiler_fence},
};

const ITER: usize = 10_000_000;

const EXIT_TOO_LARGE: u8 = 2;

const CARGO_PKG_VERSION: &str = env!("CARGO_PKG_VERSION");
const TARGET_TRIPLE: &str = env!("VERGEN_CARGO_TARGET_TRIPLE");
const GIT_DESCRIBE: &str = env!("VERGEN_GIT_DESCRIBE");

#[cfg(target_pointer_width = "64")]
const PTR_SIZE: usize = 8;
#[cfg(target_pointer_width = "32")]
const PTR_SIZE: usize = 4;

const MB: usize = 1024 * 1024;

#[derive(supershorty::Args, Debug)]
#[args(name = "nanoda", allow_no_args = true)]
struct Args {
    #[arg(flag = 'n', help = "used memory in `MiB`")]
    memory_size: Option<u64>,
    #[arg(flag = 'i', help = "test iterations")]
    iterations: Option<u64>,
}

#[cfg(target_pointer_width = "64")]
#[inline(always)]
fn permute(i: usize, n: usize) -> usize {
    (i.wrapping_mul(6364136223846793005).wrapping_add(1)) & (n - 1)
}

#[cfg(target_pointer_width = "32")]
#[inline(always)]
fn permute(i: usize, n: usize) -> usize {
    (i.wrapping_mul(1664525).wrapping_add(1013904223)) & (n - 1)
}

fn main() -> ExitCode {
    let args = match Args::parse() {
        Ok(v) => v,
        Err(e) => {
            return e;
        }
    };

    if args.memory_size.is_none() || args.iterations.is_none() {
        Args::usage();
        return ExitCode::FAILURE;
    }
    let n_mb = args.memory_size.unwrap();
    if n_mb > (usize::MAX / (PTR_SIZE * MB)) as u64 {
        eprintln!("`n` is too large");
        return ExitCode::from(EXIT_TOO_LARGE);
    }
    let memory_size = n_mb as usize * MB / PTR_SIZE;
    let iterations = args.iterations.unwrap();
    if iterations > usize::MAX as u64 {
        eprintln!("`i` is too large");
        return ExitCode::from(EXIT_TOO_LARGE);
    }
    let iterations = iterations as usize;

    println!(
        "nanoda version:\tv{} (cargo), {} (git)\nbuild target:\t{}\nmemory size:\t{} MiB, test iterations {}",
        CARGO_PKG_VERSION,
        if GIT_DESCRIBE == "VERGEN_IDEMPOTENT_OUTPUT" {
            "unknown"
        } else {
            GIT_DESCRIBE
        },
        TARGET_TRIPLE,
        n_mb,
        iterations
    );

    let mut data = Box::new_uninit_slice(memory_size);
    for i in 0..memory_size {
        data[i].write(permute(i + 1, memory_size));
    }
    let data = unsafe { data.assume_init() };
    let base = data.as_ptr();
    let mut results = Box::new_uninit_slice(iterations);
    let clock = quanta::Clock::new();
    for r in 0..iterations {
        let mut p = r;
        let t0 = clock.now();
        for _ in 0..ITER {
            p = unsafe { ptr::read_volatile(base.add(p)) };
            compiler_fence(Ordering::SeqCst);
        }
        let dt = t0.elapsed().as_secs_f64() * 1e9 / ITER as f64;
        results[r].write(dt);
        hint::black_box(p);
    }

    let results = unsafe { results.assume_init() };
    let min = results.iter().copied().reduce(f64::min).unwrap();
    let max = results.iter().copied().reduce(f64::max).unwrap();
    print!("\nresults:\nmin = {min:.3} ns, max = {max:.3} ns");
    if iterations > 2 {
        let avg = results.iter().sum::<f64>() / results.len() as f64;
        print!(", avg = {avg:.3} ns");
    }
    println!();

    ExitCode::SUCCESS
}
