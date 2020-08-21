#![no_std]

pub mod adapter;
mod buffer;
pub mod ingress;
pub mod network;
mod num;
mod parser;
pub mod protocol;

pub use adapter::initialize;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
