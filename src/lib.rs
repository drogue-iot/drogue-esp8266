
#![no_std]

pub mod adapter;
pub mod protocol;
pub mod ingress;
pub mod network;
mod buffer;
mod parser;
mod num;

pub use adapter::initialize;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
