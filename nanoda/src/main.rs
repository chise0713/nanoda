use std::{env, process::ExitCode};

use chase_seq::ChaseSeqBuilder;

const EXIT_ARG: u8 = 2;

const UNKNOWN: &str = "unknown";
const BUILD_TIMESTAMP: &str = env!("VERGEN_BUILD_TIMESTAMP");
const CARGO_PKG_VERSION: &str = env!("CARGO_PKG_VERSION");
const TARGET_TRIPLE: &str = env!("VERGEN_CARGO_TARGET_TRIPLE");
const GIT_DESCRIBE: &str = env!("VERGEN_GIT_DESCRIBE");
const GIT_SHA: &str = env!("VERGEN_GIT_SHA");
const GITHUB_RUN_URL: Option<&str> = option_env!("GITHUB_RUN_URL");

#[cfg(target_pointer_width = "64")]
const MAX_REGION_SIZE: usize = 4096 * chase_seq::MB;
#[cfg(target_pointer_width = "32")]
const MAX_REGION_SIZE: usize = 1024 * chase_seq::MB;

#[derive(supershorty::Args, Debug)]
#[args(name = "nanoda", allow_no_args = true)]
struct Args {
    #[arg(flag = 'n', help = "used memory in `MiB`, minimum is `0.001` MiB")]
    memory_size: Option<f64>,
    #[arg(flag = 'i', help = "test iterations, maximum is `65535`")]
    test_iterations: Option<u64>,
    #[arg(flag = 'b', help = "show build message")]
    build_info: bool,
    #[arg(flag = 'd', help = "don't show any output except results")]
    direct: bool,
}

fn build_info() {
    version_info();
    println!(
        "{:<20} {}",
        "commit sha2 hash:",
        if GIT_SHA == "VERGEN_IDEMPOTENT_OUTPUT" {
            UNKNOWN
        } else {
            GIT_SHA
        }
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
        if GIT_DESCRIBE == "VERGEN_IDEMPOTENT_OUTPUT" {
            UNKNOWN
        } else {
            GIT_DESCRIBE
        }
    );
    println!("{:<20} {}", "build target:", TARGET_TRIPLE);
}

fn main() -> ExitCode {
    let args = match Args::parse() {
        Ok(v) => v,
        Err(e) => {
            return e;
        }
    };

    let memory_size = if let Some(n) = args.memory_size {
        let n = (n * chase_seq::KB as f64).round() as usize;
        if n == 0 {
            eprintln!("`n` is too small");
            return ExitCode::from(EXIT_ARG);
        }
        n
    } else {
        0
    };
    if memory_size > MAX_REGION_SIZE / chase_seq::KB {
        eprintln!("`n` is too large");
        return ExitCode::from(EXIT_ARG);
    }
    let test_iterations = if let Some(n) = args.test_iterations {
        if n > u16::MAX as u64 {
            eprintln!("`i` is too large");
            return ExitCode::from(EXIT_ARG);
        }
        n as usize
    } else {
        0usize
    };

    let no_work = memory_size == 0 || test_iterations == 0;
    if args.build_info {
        build_info();
        if no_work {
            return ExitCode::SUCCESS;
        }
    } else {
        if no_work {
            Args::usage();
            return ExitCode::FAILURE;
        }
        if !args.direct {
            version_info();
        }
    }

    if !args.direct {
        println!(
            "{:<20} {} KiB, test iterations {}",
            "memory size:", memory_size, test_iterations
        );
    }

    let results = ChaseSeqBuilder::default()
        .size(memory_size)
        .unwrap()
        .fence(false)
        .seed("zundamon nanoda!!")
        .build()
        .chase(test_iterations)
        .unwrap();

    let min = results.iter().copied().reduce(f64::min).unwrap();
    let max = results.iter().copied().reduce(f64::max).unwrap();
    if !args.direct {
        print!("\nresults:\n")
    }
    print!("min = {min:.3} ns, max = {max:.3} ns");
    if results.len() > 2 {
        let avg = results.iter().sum::<f64>() / results.len() as f64;
        print!(", avg = {avg:.3} ns");
    }
    println!();

    ExitCode::SUCCESS
}
