//! This crate wraps [libunbound](https://unbound.nlnetlabs.nl) from
//! [NLnet Labs](https://nlnetlabs.nl). libunbound is an implementation of a DNS resolver,
//! including cache and DNSSEC validation.
//!
//! The interface provided follows libunbound closely:
//!
//! * `ub_ctx` is wrapped by [Context](struct.Context.html). OpenSSL is initialised when a
//! [Context](struct.Context.html) is substantiated. Functions from libunbound that
//! operate on `ub_ctx` are accessed using methods on [Context](struct.Context.html).
//!
//! * `ub_result` is wrapped by [Answer](struct.Answer.html). Methods on
//! [Answer](struct.Answer.html) are used to safely access the fields of `ub_result`.
//!
//! *Note:* A panic during a callback will lead to an abort in Rust 1.24 and later.
//! In earlier releases Rust will try to unwind which will not go well.
//!
use unbound_sys as sys;

use std::borrow::Borrow;
use std::cell::UnsafeCell;
use std::ffi::{CStr, CString, NulError};
use std::io::{self, Write};
use std::path::Path;
use std::sync::Mutex;
use std::{fmt, net, ptr};

use libc::{c_char, c_int, c_void};

const IP_CSTR_MAX: usize = 40;

/// Common Result type for operations.
pub type Result<T> = std::result::Result<T, Error>;

/// Common Error type for operations.
pub enum Error {
    /// Argument contained a null byte
    NullByte,
    /// A libunbound error
    UB(c_int),
    /// Argument contained invalid UTF8
    UTF8,
}

impl Error {
    fn as_str(&self) -> &str {
        match *self {
            Error::NullByte => "argument contains null byte",
            Error::UB(n) => {
                unsafe {
                    // At time of writing ub_strerror always returns a string.
                    // Assume that won't change in the future.
                    CStr::from_ptr(sys::ub_strerror(n)).to_str().unwrap()
                }
            }
            Error::UTF8 => "argument is invalid UTF-8",
        }
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_str().fmt(f)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_str().fmt(f)
    }
}

impl std::error::Error for Error {
    fn description(&self) -> &str {
        self.as_str()
    }
}

impl std::convert::From<NulError> for Error {
    fn from(_err: NulError) -> Error {
        Error::NullByte
    }
}

macro_rules! into_result {
    ($err:expr) => {
        into_result!($err, ())
    };
    ($err:expr, $ok:expr) => {
        match $err {
            0 => Ok($ok),
            err => Err(Error::UB(err)),
        }
    };
}

/// Wraps `ub_result`. The result of DNS resolution and validation of a query.
pub struct Answer(*mut sys::ub_result);

unsafe impl Sync for Answer {}
unsafe impl Send for Answer {}

impl Drop for Answer {
    fn drop(&mut self) {
        unsafe { sys::ub_resolve_free(self.0) }
    }
}

impl Answer {
    /// Returns original question's name.
    pub fn qname(&self) -> &str {
        unsafe {
            // Assume qname is always present and is in RFC 1035 presentation form (ASCII).
            CStr::from_ptr((*self.0).qname).to_str().unwrap()
        }
    }
    /// Returns original question's qtype.
    pub fn qtype(&self) -> u16 {
        unsafe { (*self.0).qtype as u16 }
    }
    /// Returns original question's qclass.
    pub fn qclass(&self) -> u16 {
        unsafe { (*self.0).qclass as u16 }
    }
    /// Returns an iterator over answer record datas.
    pub fn data(&self) -> DataIter<'_> {
        DataIter {
            index: 0,
            answer: self,
        }
    }
    /// Returns canonical name of result, if any.
    pub fn canonname(&self) -> Option<&str> {
        unsafe {
            let ptr = (*self.0).canonname;
            if ptr.is_null() {
                None
            } else {
                // Assume canonname is in RFC 1035 presentation form (ASCII).
                Some(CStr::from_ptr(ptr).to_str().unwrap())
            }
        }
    }
    /// Returns additional error code in case of no data.
    pub fn rcode(&self) -> u16 {
        unsafe { (*self.0).rcode as u16 }
    }
    /// Returns answer packet, if any.
    pub fn answer(&self) -> Option<&[u8]> {
        unsafe {
            let offset = (*self.0).answer_packet;
            if offset.is_null() {
                None
            } else {
                let len = (*self.0).answer_len as usize;
                Some(std::slice::from_raw_parts(offset as *const u8, len))
            }
        }
    }
    /// Returns true if there is data.
    pub fn havedata(&self) -> bool {
        unsafe { (*self.0).havedata != 0 }
    }
    /// Returns true if there is no data because a name does not exist.
    pub fn nxdomain(&self) -> bool {
        unsafe { (*self.0).nxdomain != 0 }
    }
    /// True if result is secure.
    pub fn secure(&self) -> bool {
        unsafe { (*self.0).secure != 0 }
    }
    /// True if a security failure happened.
    pub fn bogus(&self) -> bool {
        unsafe { (*self.0).bogus != 0 }
    }
    /// String error if response is bogus.
    pub fn why_bogus(&self) -> Option<&str> {
        if self.bogus() {
            // If bogus there should always be a string
            Some(unsafe { CStr::from_ptr((*self.0).why_bogus).to_str().unwrap() })
        } else {
            None
        }
    }
    /// Number of seconds the result is valid.
    pub fn ttl(&self) -> u32 {
        unsafe { (*self.0).ttl as u32 }
    }
}

impl fmt::Debug for Answer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!(
            "Answer({:p}/{}/{}/{})",
            self,
            self.qname(),
            self.qtype(),
            self.qclass()
        ))
    }
}

/// An iterator over the datas of an [Answer](struct.Answer.html).
pub struct DataIter<'a> {
    index: isize,
    answer: &'a Answer,
}

impl<'a> std::iter::Iterator for DataIter<'a> {
    type Item = &'a [u8];
    fn next(&mut self) -> Option<&'a [u8]> {
        let item = unsafe {
            let offset = (*self.answer.0).data.offset(self.index);
            if offset.is_null() {
                None
            } else {
                let len = *(*self.answer.0).len.offset(self.index) as usize;
                Some(std::slice::from_raw_parts(*offset as *const u8, len))
            }
        };
        if item.is_some() {
            self.index += 1
        }
        item
    }
}

struct Callback {
    async_id: AsyncID,
    ub_id: c_int,
    f: Box<dyn Fn(AsyncID, Result<Answer>) + 'static>,
}

/// Wraps `ub_ctx`.
pub struct Context {
    ub_ctx: *mut sys::ub_ctx,
    protected: Mutex<ContextProtected>,
}

unsafe impl Sync for Context {}
unsafe impl Send for Context {}

impl Drop for Context {
    fn drop(&mut self) {
        unsafe {
            sys::ub_ctx_delete(self.ub_ctx);
        }
    }
}

impl Context {
    /// Create a new `Context`.
    pub fn new() -> std::result::Result<Context, ()> {
        sys::init();
        let ctx = unsafe { sys::ub_ctx_create() };
        if ctx.is_null() {
            Err(())
        } else {
            Ok(Context {
                ub_ctx: ctx,
                protected: Mutex::new(Default::default()),
            })
        }
    }
    /// Set option `opt` to value `val`.
    pub fn set_option(&self, opt: &str, val: &str) -> Result<()> {
        let opt = CString::new(opt)?;
        let val = CString::new(val)?;
        unsafe {
            let ub_err = sys::ub_ctx_set_option(self.ub_ctx, opt.as_ptr(), val.as_ptr());
            into_result!(ub_err)
        }
    }
    /// Get the value of an option.
    pub fn get_option(&self, opt: &str) -> Result<String> {
        let opt = CString::new(opt)?;
        unsafe {
            let mut result: *mut c_char = ptr::null_mut();
            let ub_err = sys::ub_ctx_get_option(self.ub_ctx, opt.as_ptr(), &mut result);
            into_result!(ub_err)?;
            // Assume values are always ASCII
            let val_r = CStr::from_ptr(result).to_str().map(String::from);
            libc::free(result as *mut c_void);
            Ok(val_r.unwrap())
        }
    }
    /// Set configuration from file.
    pub fn config<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let path = path_to_cstring(path.as_ref())?;
        unsafe { into_result!(sys::ub_ctx_config(self.ub_ctx, path.as_ptr())) }
    }
    // TODO: add test covering this, and for every other option
    /// Stub a zone to a host.
    #[cfg(ub_ctx_set_stub)]
    pub fn set_stub<T: Borrow<net::IpAddr>>(&self, zone: &str, ip: T, prime: bool) -> Result<()> {
        match ip.borrow() {
            &net::IpAddr::V4(ref ip) => self.set_stub4(zone, ip, prime),
            &net::IpAddr::V6(ref ip) => self.set_stub6(zone, ip, prime),
        }
    }
    /// Stub a zone to an IPv4 host.
    #[cfg(ub_ctx_set_stub)]
    pub fn set_stub4<T>(&self, zone: &str, ip: T, prime: bool) -> Result<()>
    where
        T: Borrow<net::Ipv4Addr>,
    {
        let mut buf = [0u8; IP_CSTR_MAX];
        let ip = ipv4_to_cstr(ip.borrow(), &mut buf);
        self.set_stub_imp(zone, ip, prime)
    }
    /// Stub a zone to an IPv6 host.
    #[cfg(ub_ctx_set_stub)]
    pub fn set_stub6<T>(&self, zone: &str, ip: T, prime: bool) -> Result<()>
    where
        T: Borrow<net::Ipv6Addr>,
    {
        let mut buf = [0u8; IP_CSTR_MAX];
        let ip = ipv6_to_cstr(ip.borrow(), &mut buf);
        self.set_stub_imp(zone, ip, prime)
    }
    #[cfg(ub_ctx_set_stub)]
    fn set_stub_imp(&self, zone: &str, ip: &CStr, prime: bool) -> Result<()> {
        let zone = CString::new(zone)?;
        unsafe {
            let ub_err = sys::ub_ctx_set_stub(self.ub_ctx, zone.as_ptr(), ip.as_ptr(), prime as _);
            into_result!(ub_err)
        }
    }
    /// Forward queries to host.
    pub fn set_fwd<T: Borrow<net::IpAddr>>(&self, ip: T) -> Result<()> {
        match ip.borrow() {
            &net::IpAddr::V4(ref ip) => self.set_fwd4(ip),
            &net::IpAddr::V6(ref ip) => self.set_fwd6(ip),
        }
    }
    /// Forward queries to an IPv4 host.
    pub fn set_fwd4<T: Borrow<net::Ipv4Addr>>(&self, ip: T) -> Result<()> {
        let mut buf = [0u8; IP_CSTR_MAX];
        let target = ipv4_to_cstr(ip.borrow(), &mut buf);
        unsafe { into_result!(sys::ub_ctx_set_fwd(self.ub_ctx, target.as_ptr())) }
    }
    /// Forward queries to an IPv6 host.
    pub fn set_fwd6<T: Borrow<net::Ipv6Addr>>(&self, ip: T) -> Result<()> {
        let mut buf = [0u8; IP_CSTR_MAX];
        let target = ipv6_to_cstr(ip.borrow(), &mut buf);
        unsafe { into_result!(sys::ub_ctx_set_fwd(self.ub_ctx, target.as_ptr())) }
    }
    /// Read nameservers from /etc/resolv.conf.
    pub fn resolvconf(&self) -> Result<()> {
        unsafe { into_result!(sys::ub_ctx_resolvconf(self.ub_ctx, ptr::null())) }
    }
    /// Read nameservers from a file.
    pub fn resolvconf_path<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let path = path_to_cstring(path.as_ref())?;
        unsafe { into_result!(sys::ub_ctx_resolvconf(self.ub_ctx, path.as_ptr())) }
    }
    /// Read hosts from /etc/hosts.
    pub fn hosts(&self) -> Result<()> {
        unsafe { into_result!(sys::ub_ctx_hosts(self.ub_ctx, ptr::null())) }
    }
    /// Read hosts from a file.
    pub fn hosts_path<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let path = path_to_cstring(path.as_ref())?;
        unsafe { into_result!(sys::ub_ctx_hosts(self.ub_ctx, path.as_ptr())) }
    }
    /// Add a single line string containing a valid DNSKEY or DS RR as a trust anchor.
    pub fn add_ta(&self, ta: &str) -> Result<()> {
        let ta = CString::new(ta)?;
        unsafe { into_result!(sys::ub_ctx_add_ta(self.ub_ctx, ta.as_ptr())) }
    }
    /// Add a trust anchor that is updated automatically in line with
    /// [RFC 5011](https://tools.ietf.org/html/rfc5011).
    #[cfg(ub_ctx_add_ta_autr)]
    pub fn add_ta_autr<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let path = path_to_cstring(path.as_ref())?;
        unsafe { into_result!(sys::ub_ctx_add_ta_autr(self.ub_ctx, path.as_ptr())) }
    }
    /// Add trust anchors from a file containing DS and DNSKEY records.
    pub fn add_ta_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let path = path_to_cstring(path.as_ref())?;
        unsafe { into_result!(sys::ub_ctx_add_ta_file(self.ub_ctx, path.as_ptr())) }
    }
    /// Add trust anchors from a BIND-style configuration file.
    pub fn trustedkeys<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let path = path_to_cstring(path.as_ref())?;
        unsafe { into_result!(sys::ub_ctx_trustedkeys(self.ub_ctx, path.as_ptr())) }
    }
    /// Set debug and error output to the specified stream.
    pub fn debugout(&self, out: *mut libc::FILE) -> Result<()> {
        unsafe { into_result!(sys::ub_ctx_debugout(self.ub_ctx, out as *mut _)) }
    }
    /// Set debug verbosity. 0 is off, 1 is very minimal, 2 is detailed, and 3 is lots.
    pub fn debuglevel(&self, d: c_int) -> Result<()> {
        unsafe { into_result!(sys::ub_ctx_debuglevel(self.ub_ctx, d)) }
    }
    /// Do asynchronous resolution in a fork.
    pub fn async_via_fork(&self) -> Result<()> {
        unsafe { into_result!(sys::ub_ctx_async(self.ub_ctx, false as _)) }
    }
    /// Do asynchronous resolution on a new thread.
    pub fn async_via_thread(&self) -> Result<()> {
        unsafe { into_result!(sys::ub_ctx_async(self.ub_ctx, true as _)) }
    }
    /// Indicates whether new results are pending.
    pub fn poll(&self) -> bool {
        unsafe { sys::ub_poll(self.ub_ctx) != 0 }
    }
    /// Indicates whether there are any unprocessed asynchronous queries remaining.
    pub fn have_waiting(&self) -> bool {
        !self
            .protected
            .lock()
            .expect("have_waiting acquire protected")
            .callbacks
            .is_empty()
    }
    /// Waits for outstanding queries to complete and calls `self.process()`.
    pub fn wait(&self) -> Result<()> {
        unsafe {
            CONTEXT_PTR.with(|cell| *cell.get() = &self.protected);
            let ub_err = sys::ub_wait(self.ub_ctx);
            CONTEXT_PTR.with(|cell| *cell.get() = std::ptr::null());
            self.protected
                .lock()
                .expect("wait acquire protected")
                .adjust_capacity();
            into_result!(ub_err)
        }
    }
    /// Returns a file descriptor that is readable when one or more answers are ready.
    /// Valid for the life of the `Context`.
    pub fn fd(&self) -> c_int {
        unsafe { sys::ub_fd(self.ub_ctx) }
    }
    /// Process results from the resolver (when `fd` is readable).
    pub fn process(&self) -> Result<()> {
        unsafe {
            CONTEXT_PTR.with(|cell| *cell.get() = &self.protected);
            let ub_err = sys::ub_process(self.ub_ctx);
            CONTEXT_PTR.with(|cell| *cell.get() = std::ptr::null());
            self.protected
                .lock()
                .expect("process acquire protected")
                .adjust_capacity();
            into_result!(ub_err)
        }
    }
    /// Resolve and validate a query.
    pub fn resolve(&self, name: &str, rrtype: u16, class: u16) -> Result<Answer> {
        let mut result: *mut sys::ub_result = ptr::null_mut();
        let name = CString::new(name)?;
        unsafe {
            let ub_err = sys::ub_resolve(
                self.ub_ctx,
                name.as_ptr(),
                rrtype as c_int,
                class as c_int,
                &mut result,
            );
            into_result!(ub_err, Answer(result))
        }
    }
    /// Resolve and validate a query asynchronously.
    /// Cancel the query by supplying the `AsyncID` to `cancel`.
    /// See also `fd`, `poll` and `process`.
    pub fn resolve_async<C>(
        &self,
        name: &str,
        rrtype: u16,
        class: u16,
        callback: C,
    ) -> Result<AsyncID>
    where
        C: Fn(AsyncID, Result<Answer>) + 'static,
    {
        let name = CString::new(name)?;
        let f = Box::new(callback);
        unsafe {
            let mut p = self
                .protected
                .lock()
                .expect("resolve_async acquire protected");
            let async_id = AsyncID(p.next_id());
            let mut ub_id: c_int = 0;
            let ub_err = sys::ub_resolve_async(
                self.ub_ctx,
                name.as_ptr(),
                rrtype as c_int,
                class as c_int,
                async_id.0 as *mut c_void,
                rust_unbound_callback,
                &mut ub_id,
            );
            let result = into_result!(ub_err, async_id);
            if result.is_ok() {
                p.callbacks.push(Callback { async_id, ub_id, f });
            }
            result
        }
    }
    /// Cancel an asynchronous query.
    pub fn cancel(&self, id: AsyncID) {
        let mut p = self.protected.lock().expect("cancel acquire protected");
        if let Some(i) = p.callbacks.iter().position(|c| c.async_id == id) {
            let ctx = p.callbacks.swap_remove(i);
            unsafe { sys::ub_cancel(self.ub_ctx, ctx.ub_id) };
            p.adjust_capacity();
        }
    }
    /// Print the local zone information to debug output.
    pub fn print_local_zones(&self) -> Result<()> {
        unsafe { into_result!(sys::ub_ctx_print_local_zones(self.ub_ctx)) }
    }
    /// Add or update the zone `zone_name` as type `zone_type`.
    pub fn zone_add(&self, zone_name: &str, zone_type: &str) -> Result<()> {
        let n = CString::new(zone_name)?;
        let t = CString::new(zone_type)?;
        unsafe { into_result!(sys::ub_ctx_zone_add(self.ub_ctx, n.as_ptr(), t.as_ptr())) }
    }
    /// Remove the zone `zone_name`.
    pub fn zone_remove(&self, zone_name: &str) -> Result<()> {
        let n = CString::new(zone_name)?;
        unsafe { into_result!(sys::ub_ctx_zone_remove(self.ub_ctx, n.as_ptr())) }
    }
    /// Add a DNS record.
    pub fn data_add(&self, data: &str) -> Result<()> {
        let data = CString::new(data)?;
        unsafe { into_result!(sys::ub_ctx_data_add(self.ub_ctx, data.as_ptr())) }
    }
    /// Delete data (inserted by `data_add`) from `name`.
    pub fn data_remove(&self, name: &str) -> Result<()> {
        let data = CString::new(name)?;
        unsafe { into_result!(sys::ub_ctx_data_remove(self.ub_ctx, data.as_ptr())) }
    }
}

impl fmt::Debug for Context {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("Context({:p})", self))
    }
}

#[cfg(feature = "mio")]
impl mio::Evented for Context {
    fn register(
        &self,
        poll: &mio::Poll,
        token: mio::Token,
        interest: mio::Ready,
        opts: mio::PollOpt,
    ) -> io::Result<()> {
        mio::unix::EventedFd(&self.fd()).register(poll, token, interest, opts)
    }
    fn reregister(
        &self,
        poll: &mio::Poll,
        token: mio::Token,
        interest: mio::Ready,
        opts: mio::PollOpt,
    ) -> io::Result<()> {
        mio::unix::EventedFd(&self.fd()).register(poll, token, interest, opts)
    }
    fn deregister(&self, poll: &mio::Poll) -> io::Result<()> {
        mio::unix::EventedFd(&self.fd()).deregister(poll)
    }
}

#[derive(Default)]
struct ContextProtected {
    id: usize,
    callbacks: Vec<Callback>,
}

impl ContextProtected {
    fn adjust_capacity(&mut self) {
        if self.callbacks.capacity() > self.callbacks.len() * 4 {
            self.callbacks.shrink_to_fit();
        }
    }
    fn next_id(&mut self) -> usize {
        let id = self.id;
        self.id = id.wrapping_add(1);
        id
    }
}

thread_local!(static CONTEXT_PTR: UnsafeCell<*const Mutex<ContextProtected>> = UnsafeCell::new(std::ptr::null()));

unsafe extern "C" fn rust_unbound_callback(
    ctx_raw: *mut c_void,
    ub_err: c_int,
    result: *mut sys::ub_result,
) {
    let id = AsyncID(ctx_raw as usize);
    let result = into_result!(ub_err, Answer(result));
    let mut p = CONTEXT_PTR.with(|cell| {
        (*cell.get())
            .as_ref()
            .expect("ContextProtectedMutex")
            .lock()
            .expect("lock callbacks")
    });
    if let Some(i) = p.callbacks.iter().position(|cb| cb.async_id == id) {
        let f = p.callbacks.swap_remove(i).f;
        f(id, result);
    };
}

/// Identifies an asynchronous query.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct AsyncID(usize);

/// Wraps `ub_version`.
pub fn version() -> &'static str {
    unsafe { CStr::from_ptr(sys::ub_version()).to_str().unwrap() }
}

fn path_to_cstring(path: &Path) -> Result<CString> {
    Ok(CString::new(path.to_str().ok_or(Error::UTF8)?)?)
}

fn ipv4_to_cstr<'a>(ip: &net::Ipv4Addr, buf: &'a mut [u8; IP_CSTR_MAX]) -> &'a CStr {
    let len = {
        let mut w = io::BufWriter::new(&mut buf[..]);
        w.write_fmt(format_args!("{}", &ip))
            .expect("write_fmt ipv4");
        IP_CSTR_MAX + 1 - w.into_inner().expect("into_inner ipv4").len()
    };
    CStr::from_bytes_with_nul(&buf[..len]).expect("valid ipv4 c str")
}

fn ipv6_to_cstr<'a>(ip: &net::Ipv6Addr, buf: &'a mut [u8; IP_CSTR_MAX]) -> &'a CStr {
    let len = {
        let mut w = io::BufWriter::new(&mut buf[..]);
        w.write_fmt(format_args!("{}", &ip))
            .expect("write_fmt ipv6");
        IP_CSTR_MAX + 1 - w.into_inner().expect("into_inner ipv6").len()
    };
    CStr::from_bytes_with_nul(&buf[..len]).expect("valid ipv6 c str")
}

#[test]
fn test_ctx_options() {
    let ctx = Context::new().unwrap();
    assert!(ctx.set_option("do-ip4:", "no").is_ok());
    assert_eq!(ctx.get_option("do-ip4").unwrap(), "no");
    assert!(ctx.set_option("foo", "bah").is_err());
    assert!(ctx.get_option("foo").is_err());
    assert!(ctx.config("test/empty").is_ok());
    assert!(ctx.config("test/no-such-file").is_err());
    assert!(ctx.resolvconf().is_ok());
    assert!(ctx.resolvconf_path("test/google-dns-resolv.conf").is_ok());
    assert!(ctx.resolvconf_path("test/no-such-file").is_err());
    assert!(ctx.set_fwd4(net::Ipv4Addr::new(8, 8, 8, 8)).is_ok());
    assert!(ctx.hosts().is_ok());
    assert!(ctx.hosts_path("test/empty").is_ok());
    assert!(ctx.hosts_path("test/no-such-file").is_err());
}

#[test]
#[cfg(ub_ctx_set_stub)]
fn test_stub_after_final() {
    let ctx = Context::new().unwrap();
    let addr = net::Ipv4Addr::from([0xDD; 4]);
    assert!(ctx.hosts().is_ok());
    assert!(ctx.set_stub4("example.com.", addr, false).is_ok());
    assert!(ctx.async_via_thread().is_ok());
    assert!(ctx.resolve_async("localhost", 1, 1, |_, _| {}).is_ok());
    assert!(ctx.set_stub4("example.net.", addr, false).is_err());
}

#[test]
fn test_move_context() {
    let mut a = Context::new().unwrap();
    let mut b = Context::new().unwrap();
    for c in &[&a, &b] {
        c.async_via_thread().unwrap();
    }
    b.resolve_async("localhost", 1, 1, |_, _| {}).is_ok();
    std::mem::swap(&mut a, &mut b);
    drop(b);
    a.wait().unwrap();
}
