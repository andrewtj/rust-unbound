
pub use sys::*;

extern crate libc;
extern crate openssl;

#[allow(non_camel_case_types)]
mod sys;

pub fn init() {
    openssl::ssl::init();
}
