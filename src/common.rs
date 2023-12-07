#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

pub fn benchmark<F, T>(func: F) -> T
    where F: FnOnce() -> T
{
    let s = std::time::Instant::now();
    let r = func();
    let d = s.elapsed();
    println!("{}ms", d.as_millis());
    r
}
