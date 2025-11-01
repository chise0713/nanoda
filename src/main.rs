use std::{
    env, hint,
    process::ExitCode,
    ptr,
    sync::atomic::{Ordering, compiler_fence},
};

const ITER: usize = 10_000_000;

const EXIT_TOO_LARGE: u8 = 2;

const CARGO_PKG_VERSION: &str = env!("CARGO_PKG_VERSION");
const GIT_DESCRIBE: &str = env!("VERGEN_GIT_DESCRIBE");
const TARGET_TRIPLE: &str = env!("VERGEN_CARGO_TARGET_TRIPLE");

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
    let memory_size_orig = args.memory_size.unwrap();
    if memory_size_orig > usize::MAX as u64 / 8 / 1024 / 1024 {
        eprintln!("`n` is too large");
        return ExitCode::from(EXIT_TOO_LARGE);
    }
    let memory_size = (memory_size_orig as usize) * 1024 * 1024 / size_of::<usize>();
    let iterations = args.iterations.unwrap();
    if iterations > usize::MAX as u64 {
        eprintln!("`i` is too large");
        return ExitCode::from(EXIT_TOO_LARGE);
    }
    let iterations = iterations as usize;

    {
        let version = if GIT_DESCRIBE == "VERGEN_IDEMPOTENT_OUTPUT" {
            format!("v{} (cargo)", CARGO_PKG_VERSION)
        } else {
            format!("{} (git)\tv{} (cargo)", GIT_DESCRIBE, CARGO_PKG_VERSION)
        };
        println!(
            "nanoda version:\t{}\nbuild target:\t{}\nmemory size:\t{} MiB, test iterations {}",
            version, TARGET_TRIPLE, memory_size_orig, iterations
        );
    }

    let mut data: Box<[usize]> = unsafe { Box::new_uninit_slice(memory_size).assume_init() };
    let base = data.as_ptr();
    for i in 0..memory_size {
        data[i] = permute(i + 1, memory_size);
    }
    let mut results = unsafe { Box::new_uninit_slice(iterations).assume_init() };
    let clock = quanta::Clock::new();
    for r in 0..iterations {
        let mut p = r;
        let t0 = clock.now();
        for _ in 0..ITER {
            p = unsafe { ptr::read_volatile(base.add(p)) };
            compiler_fence(Ordering::SeqCst);
        }
        let dt = t0.elapsed().as_secs_f64() * 1e9 / ITER as f64;
        results[r] = dt;
        hint::black_box(p);
    }

    let min = results.iter().copied().reduce(f64::min).unwrap();
    let max = results.iter().copied().reduce(f64::max).unwrap();
    let mut format = format!("min = {min:.3} ns, max = {max:.3} ns");
    if iterations > 2 {
        let avg = results.iter().sum::<f64>() / results.len() as f64;
        format.push_str(&format!(", avg = {avg:.3} ns"));
    }
    println!("\nresults:\n{format}");

    return ExitCode::SUCCESS;
}
