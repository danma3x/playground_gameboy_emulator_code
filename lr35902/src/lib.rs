mod operations;
mod cpu;

pub use cpu::{LR35902, Flag};

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
