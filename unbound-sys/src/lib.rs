#![allow(non_camel_case_types)]

extern crate libc;
extern crate openssl;

pub fn init() {
    openssl::init();
}

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
