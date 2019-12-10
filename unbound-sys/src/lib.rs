
/* generated by generate_sys.pl */

#![allow(non_camel_case_types)]
#![doc="
The following functions are behind features with the same name:

* ub_ctx_set_stub
* ub_ctx_add_ta_autr
* ub_ctx_set_tls
"]

extern crate libc;
extern crate openssl;

pub fn init() {
    openssl::init();
}



/* automatically generated by rust-bindgen */

pub enum ub_ctx{}
#[repr(C)]
pub struct ub_result {
    pub qname: *mut ::libc::c_char,
    pub qtype: ::libc::c_int,
    pub qclass: ::libc::c_int,
    pub data: *mut *mut ::libc::c_char,
    pub len: *mut ::libc::c_int,
    pub canonname: *mut ::libc::c_char,
    pub rcode: ::libc::c_int,
    pub answer_packet: *mut ::libc::c_void,
    pub answer_len: ::libc::c_int,
    pub havedata: ::libc::c_int,
    pub nxdomain: ::libc::c_int,
    pub secure: ::libc::c_int,
    pub bogus: ::libc::c_int,
    pub why_bogus: *mut ::libc::c_char,
    #[cfg(ub_ctx_has_was_ratelimited)] pub was_ratelimited: ::libc::c_int,
    pub ttl: ::libc::c_int,
}
pub type ub_callback_type = unsafe extern "C" fn(arg1: *mut ::libc::c_void, arg2: ::libc::c_int, arg3: *mut ub_result);
extern "C" {
    pub fn ub_ctx_create() -> *mut ub_ctx;
}
extern "C" {
    pub fn ub_ctx_delete(ctx: *mut ub_ctx);
}
extern "C" {
    pub fn ub_ctx_set_option(
        ctx: *mut ub_ctx,
        opt: *const ::libc::c_char,
        val: *const ::libc::c_char,
    ) -> ::libc::c_int;
}
extern "C" {
    pub fn ub_ctx_get_option(
        ctx: *mut ub_ctx,
        opt: *const ::libc::c_char,
        str: *mut *mut ::libc::c_char,
    ) -> ::libc::c_int;
}
extern "C" {
    pub fn ub_ctx_config(ctx: *mut ub_ctx, fname: *const ::libc::c_char) -> ::libc::c_int;
}
extern "C" {
    pub fn ub_ctx_set_fwd(ctx: *mut ub_ctx, addr: *const ::libc::c_char) -> ::libc::c_int;
}
extern "C" {
    #[cfg(feature = "ub_ctx_set_tls")] pub fn ub_ctx_set_tls(ctx: *mut ub_ctx, tls: ::libc::c_int) -> ::libc::c_int;
}
extern "C" {
    #[cfg(feature = "ub_ctx_set_stub")] pub fn ub_ctx_set_stub(
        ctx: *mut ub_ctx,
        zone: *const ::libc::c_char,
        addr: *const ::libc::c_char,
        isprime: ::libc::c_int,
    ) -> ::libc::c_int;
}
extern "C" {
    pub fn ub_ctx_resolvconf(ctx: *mut ub_ctx, fname: *const ::libc::c_char) -> ::libc::c_int;
}
extern "C" {
    pub fn ub_ctx_hosts(ctx: *mut ub_ctx, fname: *const ::libc::c_char) -> ::libc::c_int;
}
extern "C" {
    pub fn ub_ctx_add_ta(ctx: *mut ub_ctx, ta: *const ::libc::c_char) -> ::libc::c_int;
}
extern "C" {
    pub fn ub_ctx_add_ta_file(ctx: *mut ub_ctx, fname: *const ::libc::c_char) -> ::libc::c_int;
}
extern "C" {
    #[cfg(feature = "ub_ctx_add_ta_autr")] pub fn ub_ctx_add_ta_autr(ctx: *mut ub_ctx, fname: *const ::libc::c_char) -> ::libc::c_int;
}
extern "C" {
    pub fn ub_ctx_trustedkeys(ctx: *mut ub_ctx, fname: *const ::libc::c_char) -> ::libc::c_int;
}
extern "C" {
    pub fn ub_ctx_async(ctx: *mut ub_ctx, dothread: ::libc::c_int) -> ::libc::c_int;
}
extern "C" {
    pub fn ub_poll(ctx: *mut ub_ctx) -> ::libc::c_int;
}
extern "C" {
    pub fn ub_wait(ctx: *mut ub_ctx) -> ::libc::c_int;
}
extern "C" {
    pub fn ub_fd(ctx: *mut ub_ctx) -> ::libc::c_int;
}
extern "C" {
    pub fn ub_process(ctx: *mut ub_ctx) -> ::libc::c_int;
}
extern "C" {
    pub fn ub_resolve(
        ctx: *mut ub_ctx,
        name: *const ::libc::c_char,
        rrtype: ::libc::c_int,
        rrclass: ::libc::c_int,
        result: *mut *mut ub_result,
    ) -> ::libc::c_int;
}
extern "C" {
    pub fn ub_resolve_async(
        ctx: *mut ub_ctx,
        name: *const ::libc::c_char,
        rrtype: ::libc::c_int,
        rrclass: ::libc::c_int,
        mydata: *mut ::libc::c_void,
        callback: ub_callback_type,
        async_id: *mut ::libc::c_int,
    ) -> ::libc::c_int;
}
extern "C" {
    pub fn ub_cancel(ctx: *mut ub_ctx, async_id: ::libc::c_int) -> ::libc::c_int;
}
extern "C" {
    pub fn ub_resolve_free(result: *mut ub_result);
}
extern "C" {
    pub fn ub_strerror(err: ::libc::c_int) -> *const ::libc::c_char;
}
extern "C" {
    pub fn ub_ctx_zone_add(
        ctx: *mut ub_ctx,
        zone_name: *const ::libc::c_char,
        zone_type: *const ::libc::c_char,
    ) -> ::libc::c_int;
}
extern "C" {
    pub fn ub_ctx_zone_remove(ctx: *mut ub_ctx, zone_name: *const ::libc::c_char) -> ::libc::c_int;
}
extern "C" {
    pub fn ub_ctx_data_add(ctx: *mut ub_ctx, data: *const ::libc::c_char) -> ::libc::c_int;
}
extern "C" {
    pub fn ub_ctx_data_remove(ctx: *mut ub_ctx, data: *const ::libc::c_char) -> ::libc::c_int;
}
extern "C" {
    pub fn ub_version() -> *const ::libc::c_char;
}
