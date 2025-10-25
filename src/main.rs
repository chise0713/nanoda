use std::process::ExitCode;

const ITER: usize = 10_000_000;

const EXIT_TOO_LARGE: u8 = 2;

#[derive(supershorty::Args, Debug)]
#[args(name = "nanoda", allow_no_args = true)]
struct Args {
    #[arg(flag = 'n', help = "used memory in `MiB`")]
    memory_size: Option<u64>,
    #[arg(flag = 'i', help = "iterations")]
    iterations: Option<u64>,
}

unsafe fn uninit_boxed_slice<T>(len: usize) -> Box<[T]> {
    use std::alloc::{Layout, alloc, handle_alloc_error};
    unsafe {
        let layout = Layout::array::<T>(len).unwrap();
        let ptr = alloc(layout) as *mut T;
        if ptr.is_null() {
            handle_alloc_error(layout);
        }
        Box::from_raw(std::slice::from_raw_parts_mut(ptr, len))
    }
}

#[cfg(target_pointer_width = "64")]
#[inline(always)]
fn permute(i: usize, n: usize) -> usize {
    (i.wrapping_mul(6364136223846793005).wrapping_add(1)) & (n - 1)
}

#[cfg(target_pointer_width = "32")]
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
    let memory_size = args.memory_size.unwrap();
    if memory_size > usize::MAX as u64 / 8 / 1024 / 1024 {
        eprintln!("`n` is too large");
        return ExitCode::from(EXIT_TOO_LARGE);
    }
    let memory_size = (memory_size as usize) * 1024 * 1024 / size_of::<usize>();
    let iterations = args.iterations.unwrap();
    if iterations > usize::MAX as u64 {
        eprintln!("`i` is too large");
        return ExitCode::from(EXIT_TOO_LARGE);
    }
    let iterations = iterations as usize;

    let mut data: Box<[usize]> = unsafe { uninit_boxed_slice(memory_size) };
    for i in 0..memory_size {
        data[i] = permute(i + 1, memory_size);
    }

    let mut results = Vec::with_capacity(iterations);
    let clock = quanta::Clock::new();
    for r in 0..iterations {
        let mut p = r;
        let t0 = clock.now();
        for _ in 0..ITER {
            p = data[p];
        }
        let dt = t0.elapsed().as_secs_f64() * 1e9 / ITER as f64;
        results.push(dt);
        std::hint::black_box(p);
    }

    let min = results.iter().copied().reduce(f64::min).unwrap();
    let max = results.iter().copied().reduce(f64::max).unwrap();
    let mut format = format!("min = {min:.3} ns, max = {max:.3} ns");
    if iterations > 2 {
        let avg = results.iter().sum::<f64>() / results.len() as f64;
        format.push_str(&format!(", avg = {avg:.3} ns"));
    }
    println!("{format}");

    return ExitCode::SUCCESS;
}
