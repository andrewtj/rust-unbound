/* generated by generate_sys.pl */
#![allow(non_camel_case_types, non_snake_case)]
pub fn init() {
    openssl::init();
}
/* automatically generated by rust-bindgen */

pub enum ub_ctx {}
#[repr(C)]
#[derive(Debug)]
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
    pub ttl: ::libc::c_int,
}
pub type ub_callback_type =
    unsafe extern "C" fn(arg1: *mut ::libc::c_void, arg2: ::libc::c_int, arg3: *mut ub_result);
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
    #[cfg(ub_ctx_set_stub)]
    pub fn ub_ctx_set_stub(
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
    #[cfg(ub_ctx_add_ta_autr)]
    pub fn ub_ctx_add_ta_autr(ctx: *mut ub_ctx, fname: *const ::libc::c_char) -> ::libc::c_int;
}
extern "C" {
    pub fn ub_ctx_trustedkeys(ctx: *mut ub_ctx, fname: *const ::libc::c_char) -> ::libc::c_int;
}
extern "C" {
    pub fn ub_ctx_debugout(ctx: *mut ub_ctx, out: *mut ::libc::c_void) -> ::libc::c_int;
}
extern "C" {
    pub fn ub_ctx_debuglevel(ctx: *mut ub_ctx, d: ::libc::c_int) -> ::libc::c_int;
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
    pub fn ub_ctx_print_local_zones(ctx: *mut ub_ctx) -> ::libc::c_int;
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
#[repr(C)]
#[derive(Debug)]
pub struct ub_shm_stat_info {
    pub num_threads: ::libc::c_int,
    pub time: ub_shm_stat_info__bindgen_ty_1,
    pub mem: ub_shm_stat_info__bindgen_ty_2,
}
#[repr(C)]
#[derive(Debug)]
pub struct ub_shm_stat_info__bindgen_ty_1 {
    pub now_sec: ::libc::c_longlong,
    pub now_usec: ::libc::c_longlong,
    pub up_sec: ::libc::c_longlong,
    pub up_usec: ::libc::c_longlong,
    pub elapsed_sec: ::libc::c_longlong,
    pub elapsed_usec: ::libc::c_longlong,
}
#[repr(C)]
#[derive(Debug)]
pub struct ub_shm_stat_info__bindgen_ty_2 {
    pub msg: ::libc::c_longlong,
    pub rrset: ::libc::c_longlong,
    pub val: ::libc::c_longlong,
    pub iter: ::libc::c_longlong,
    pub subnet: ::libc::c_longlong,
    pub ipsecmod: ::libc::c_longlong,
    pub respip: ::libc::c_longlong,
    pub dnscrypt_shared_secret: ::libc::c_longlong,
    pub dnscrypt_nonce: ::libc::c_longlong,
}
#[repr(C)]
pub struct ub_server_stats {
    pub num_queries: ::libc::c_longlong,
    pub num_queries_ip_ratelimited: ::libc::c_longlong,
    pub num_queries_missed_cache: ::libc::c_longlong,
    pub num_queries_prefetch: ::libc::c_longlong,
    pub sum_query_list_size: ::libc::c_longlong,
    pub max_query_list_size: ::libc::c_longlong,
    pub extended: ::libc::c_int,
    pub qtype: [::libc::c_longlong; 256usize],
    pub qtype_big: ::libc::c_longlong,
    pub qclass: [::libc::c_longlong; 256usize],
    pub qclass_big: ::libc::c_longlong,
    pub qopcode: [::libc::c_longlong; 16usize],
    pub qtcp: ::libc::c_longlong,
    pub qtcp_outgoing: ::libc::c_longlong,
    pub qipv6: ::libc::c_longlong,
    pub qbit_QR: ::libc::c_longlong,
    pub qbit_AA: ::libc::c_longlong,
    pub qbit_TC: ::libc::c_longlong,
    pub qbit_RD: ::libc::c_longlong,
    pub qbit_RA: ::libc::c_longlong,
    pub qbit_Z: ::libc::c_longlong,
    pub qbit_AD: ::libc::c_longlong,
    pub qbit_CD: ::libc::c_longlong,
    pub qEDNS: ::libc::c_longlong,
    pub qEDNS_DO: ::libc::c_longlong,
    pub ans_rcode: [::libc::c_longlong; 16usize],
    pub ans_rcode_nodata: ::libc::c_longlong,
    pub ans_secure: ::libc::c_longlong,
    pub ans_bogus: ::libc::c_longlong,
    pub rrset_bogus: ::libc::c_longlong,
    pub queries_ratelimited: ::libc::c_longlong,
    pub unwanted_replies: ::libc::c_longlong,
    pub unwanted_queries: ::libc::c_longlong,
    pub tcp_accept_usage: ::libc::c_longlong,
    pub zero_ttl_responses: ::libc::c_longlong,
    pub hist: [::libc::c_longlong; 40usize],
    pub msg_cache_count: ::libc::c_longlong,
    pub rrset_cache_count: ::libc::c_longlong,
    pub infra_cache_count: ::libc::c_longlong,
    pub key_cache_count: ::libc::c_longlong,
    pub num_query_dnscrypt_crypted: ::libc::c_longlong,
    pub num_query_dnscrypt_cert: ::libc::c_longlong,
    pub num_query_dnscrypt_cleartext: ::libc::c_longlong,
    pub num_query_dnscrypt_crypted_malformed: ::libc::c_longlong,
    pub num_query_dnscrypt_secret_missed_cache: ::libc::c_longlong,
    pub shared_secret_cache_count: ::libc::c_longlong,
    pub num_query_dnscrypt_replay: ::libc::c_longlong,
    pub nonce_cache_count: ::libc::c_longlong,
}
#[repr(C)]
pub struct ub_stats_info {
    pub svr: ub_server_stats,
    pub mesh_num_states: ::libc::c_longlong,
    pub mesh_num_reply_states: ::libc::c_longlong,
    pub mesh_jostled: ::libc::c_longlong,
    pub mesh_dropped: ::libc::c_longlong,
    pub mesh_replies_sent: ::libc::c_longlong,
    pub mesh_replies_sum_wait_sec: ::libc::c_longlong,
    pub mesh_replies_sum_wait_usec: ::libc::c_longlong,
    pub mesh_time_median: f64,
}
