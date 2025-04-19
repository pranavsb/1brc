use std::env;

static DEBUG: bool = true;

macro_rules! debug_println {
    ($($arg:tt)*) => {
        if DEBUG {
            println!($($arg)*);
        }
    };
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let filename = if args.len() > 1 { &args[1] } else { "measurements.txt" };
    debug_println!("Filename: {}", filename);
}
