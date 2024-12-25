use std::process;

pub fn dd<T: std::fmt::Debug>(value: T) -> ! {
    dbg!(&value);
    process::exit(1);
}