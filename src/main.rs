use std::{
    env, hint,
    ops::{Deref, DerefMut},
    process::ExitCode,
    ptr,
    sync::atomic::{Ordering, compiler_fence},
};

const ITER: usize = 10_000_000;

const EXIT_TOO_LARGE: u8 = 2;

const UNKNOWN: &str = "unknown";
const BUILD_TIMESTAMP: &str = env!("VERGEN_BUILD_TIMESTAMP");
const CARGO_PKG_VERSION: &str = env!("CARGO_PKG_VERSION");
const TARGET_TRIPLE: &str = env!("VERGEN_CARGO_TARGET_TRIPLE");
const GIT_DESCRIBE: &str = env!("VERGEN_GIT_DESCRIBE");
const GIT_SHA: &str = env!("VERGEN_GIT_SHA");
const GITHUB_RUN_URL: Option<&str> = option_env!("GITHUB_RUN_URL");

#[cfg(target_pointer_width = "64")]
const PTR_SIZE: usize = 8;
#[cfg(target_pointer_width = "32")]
const PTR_SIZE: usize = 4;

const MB: usize = 1024 * 1024;
#[cfg(target_pointer_width = "64")]
const MAX_REGION_SIZE: u64 = 4096;
#[cfg(target_pointer_width = "32")]
const MAX_REGION_SIZE: u64 = 1024;

#[derive(supershorty::Args, Debug)]
#[args(name = "nanoda", allow_no_args = true)]
struct Args {
    #[arg(flag = 'n', help = "used memory in `MiB`")]
    memory_size: Option<u64>,
    #[arg(flag = 'i', help = "test iterations")]
    iterations: Option<u64>,
    #[arg(flag = 'b', help = "show build message")]
    build_info: bool,
}

struct ChaseSeq(Box<[usize]>);

impl ChaseSeq {
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

    fn new(size: usize) -> Self {
        let mut data = Box::new_uninit_slice(size);
        for (i, slot) in data.iter_mut().enumerate() {
            slot.write(Self::permute(i + 1, size));
        }
        let data = unsafe { data.assume_init() }; // safe because all elements are initialized
        ChaseSeq(data)
    }
}

impl Deref for ChaseSeq {
    type Target = [usize];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ChaseSeq {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

fn build_info() {
    version_info();
    println!(
        "{:<20} {}",
        "commit sha2 hash:",
        (GIT_SHA == "VERGEN_IDEMPOTENT_OUTPUT")
            .then(|| UNKNOWN)
            .unwrap_or(GIT_SHA)
    );
    println!("{:<20} {}", "build timestamp:", BUILD_TIMESTAMP);
    let url = GITHUB_RUN_URL.unwrap_or(UNKNOWN);
    println!("{:<20} {}", "builder logs url:", url);
    println!(
        "only same binary gives comparable benchmark, so please note the above info and checksum."
    );
}

fn version_info() {
    println!(
        "{:<20} v{} (cargo), {} (git)",
        "nanoda version:",
        CARGO_PKG_VERSION,
        (GIT_DESCRIBE == "VERGEN_IDEMPOTENT_OUTPUT")
            .then(|| UNKNOWN)
            .unwrap_or(GIT_DESCRIBE),
    );
    println!("{:<20} {}", "build target:", TARGET_TRIPLE,);
}

fn main() -> ExitCode {
    let args = match Args::parse() {
        Ok(v) => v,
        Err(e) => {
            return e;
        }
    };
    if args.build_info {
        build_info();
        return ExitCode::SUCCESS;
    }

    if args.memory_size.is_none() || args.iterations.is_none() {
        Args::usage();
        return ExitCode::FAILURE;
    }
    let n_mb = args.memory_size.unwrap();
    if n_mb > MAX_REGION_SIZE {
        eprintln!("`n` is too large");
        return ExitCode::from(EXIT_TOO_LARGE);
    }
    let memory_size = n_mb as usize * MB / PTR_SIZE;
    let iterations = args.iterations.unwrap();
    if iterations > u16::MAX as u64 {
        eprintln!("`i` is too large");
        return ExitCode::from(EXIT_TOO_LARGE);
    }
    let iterations = iterations as usize;

    version_info();
    println!(
        "{:<20} {} MiB, test iterations {}",
        "memory size:", n_mb, iterations
    );

    let data = ChaseSeq::new(memory_size);
    let base = data.as_ptr();
    // unintialize results array, you should not read before written
    let mut results = Box::new_uninit_slice(iterations);
    let clock = quanta::Clock::new();
    for (r, slot) in results.iter_mut().enumerate() {
        let mut p = r;
        let t0 = clock.now();
        for _ in 0..(ITER.min(memory_size)) {
            p = unsafe { ptr::read_volatile(base.add(p)) };
            compiler_fence(Ordering::SeqCst);
        }
        let dt = t0.elapsed().as_secs_f64() * 1e9 / ITER as f64;
        slot.write(dt);
        hint::black_box(p);
    }
    // when you making changes,
    // always check results are fully initialized before reading.
    // do not move the `results.iter_mut().enumerate()` loop
    // below the `results.assume_init()`, never ever do that.
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
