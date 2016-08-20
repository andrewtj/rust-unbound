//! Crate unbound wraps [libunbound](https://unbound.nlnetlabs.nl) from
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
use std::collections::HashMap;
use std::ffi::{CStr, CString, NulError};
use std::{fmt, mem, ptr};
use std::sync::Mutex;
use std::path::Path;

extern crate libc;
extern crate unbound_sys as sys;

/// Common Result type for operations.
pub type Result<T> = std::result::Result<T, Error>;

/// Common Error type for operations.
#[derive(Debug)]
pub enum Error {
    /// Argument contained a null byte
    NullByte,
    /// A libunbound error
    UB(libc::c_int),
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

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
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
    ($err:expr) => (into_result!($err, ()));
    ($err:expr, $ok:expr) => (match $err {
        0 => Ok($ok),
        err => Err(Error::UB(err)),
    })
}

/// Wraps `ub_result`. The result of DNS resolution and validation of a query.
pub struct Answer(*mut sys::Struct_ub_result);

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
    pub fn datas(&self) -> Datas {
        Datas {
            index: 0,
            answer: &self,
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
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_fmt(format_args!("Answer({:p}/{}/{}/{})",
                                 self,
                                 self.qname(),
                                 self.qtype(),
                                 self.qclass()))
    }
}

impl Drop for Answer {
    fn drop(&mut self) {
        unsafe { sys::ub_resolve_free(self.0) }
    }
}

/// An iterator over the datas of an [Answer](struct.Answer.html).
pub struct Datas<'a> {
    index: isize,
    answer: &'a Answer,
}

impl<'a> std::iter::Iterator for Datas<'a> {
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

/// Wraps `ub_ctx`.
pub struct Context {
    ub_ctx: *mut sys::Struct_ub_ctx,
    callbacks: Mutex<ContextHashMap>,
}

// TODO: move this somewhere more appropriate.
fn path_to_cstring(path: &Path) -> Result<CString> {
    Ok(try!(CString::new(try!(path.to_str().ok_or(Error::UTF8)))))
}

impl Context {
    /// Attempts to construct a new `Context`.
    pub fn new() -> std::result::Result<Context, ()> {
        sys::init();
        let ctx = unsafe { sys::ub_ctx_create() };
        if ctx.is_null() {
            Err(())
        } else {
            Ok(Context {
                ub_ctx: ctx,
                callbacks: Mutex::new(ContextHashMap::new()),
            })
        }
    }
    /// Wraps `ub_ctx_set_option`.
    pub fn set_option(&self, opt: &str, val: &str) -> Result<()> {
        let opt = try!(CString::new(opt));
        let val = try!(CString::new(val));
        unsafe { into_result!(sys::ub_ctx_set_option(self.ub_ctx, opt.as_ptr(), val.as_ptr())) }
    }
    /// Wraps `ub_ctx_get_option`.
    pub fn get_option(&self, opt: &str) -> Result<String> {
        let opt = try!(CString::new(opt));
        unsafe {
            let mut result: *mut libc::c_char = ptr::null_mut();
            try!(into_result!(sys::ub_ctx_get_option(self.ub_ctx, opt.as_ptr(), &mut result)));
            // Assume values are always ASCII
            let val = CStr::from_ptr(result).to_str().unwrap().to_owned();
            libc::free(result as *mut libc::c_void);
            Ok(val)
        }
    }
    /// Wraps `ub_ctx_config`.
    pub fn config<P: AsRef<Path>>(&self, fname: P) -> Result<()> {
        let fname = try!(path_to_cstring(fname.as_ref()));
        unsafe { into_result!(sys::ub_ctx_config(self.ub_ctx, fname.as_ptr())) }
    }
    /// Wraps `ub_ctx_set_fwd`.
    pub fn set_fwd(&self, target: &str) -> Result<()> {
        let target = try!(CString::new(target));
        unsafe { into_result!(sys::ub_ctx_set_fwd(self.ub_ctx, target.as_ptr())) }
    }
    /// Wraps `ub_ctx_resolvconf`.
    pub fn resolvconf(&self) -> Result<()> {
        unsafe { into_result!(sys::ub_ctx_resolvconf(self.ub_ctx, ptr::null())) }
    }
    /// Wraps `ub_ctx_resolvconf`.
    pub fn resolvconf_path<P: AsRef<Path>>(&self, fname: P) -> Result<()> {
        let fname = try!(path_to_cstring(fname.as_ref()));
        unsafe { into_result!(sys::ub_ctx_resolvconf(self.ub_ctx, fname.as_ptr())) }
    }
    /// Wraps `ub_ctx_hosts`.
    pub fn hosts(&self) -> Result<()> {
        unsafe { into_result!(sys::ub_ctx_hosts(self.ub_ctx, ptr::null())) }
    }
    /// Wraps `ub_ctx_hosts`.
    pub fn hosts_path<P: AsRef<Path>>(&self, fname: P) -> Result<()> {
        let fname = try!(path_to_cstring(fname.as_ref()));
        unsafe { into_result!(sys::ub_ctx_hosts(self.ub_ctx, fname.as_ptr())) }
    }
    /// Wraps `ub_ctx_add_ta`.
    pub fn add_ta(&self, ta: &str) -> Result<()> {
        let ta = try!(CString::new(ta));
        unsafe { into_result!(sys::ub_ctx_add_ta(self.ub_ctx, ta.as_ptr())) }
    }
    /// Wraps `ub_ctx_add_ta_autr`.
    pub fn add_ta_autr<P: AsRef<Path>>(&self, fname: P) -> Result<()> {
        let fname = try!(path_to_cstring(fname.as_ref()));
        unsafe { into_result!(sys::ub_ctx_add_ta_autr(self.ub_ctx, fname.as_ptr())) }
    }
    /// Wraps `ub_ctx_add_ta_file`.
    pub fn add_ta_file<P: AsRef<Path>>(&self, fname: P) -> Result<()> {
        let fname = try!(path_to_cstring(fname.as_ref()));
        unsafe { into_result!(sys::ub_ctx_add_ta_file(self.ub_ctx, fname.as_ptr())) }
    }
    /// Wraps `ub_ctx_trustedkeys`.
    pub fn trustedkeys<P: AsRef<Path>>(&self, fname: P) -> Result<()> {
        let fname = try!(path_to_cstring(fname.as_ref()));
        unsafe { into_result!(sys::ub_ctx_trustedkeys(self.ub_ctx, fname.as_ptr())) }
    }
    /// Wraps `ub_ctx_debugout`.
    pub fn debugout(&self, out: *mut libc::FILE) -> Result<()> {
        unsafe { into_result!(sys::ub_ctx_debugout(self.ub_ctx, out as *mut _)) }
    }
    /// Wraps `ub_ctx_debuglevel`.
    pub fn debuglevel(&self, d: libc::c_int) -> Result<()> {
        unsafe { into_result!(sys::ub_ctx_debuglevel(self.ub_ctx, d)) }
    }
    /// Wraps `ub_ctx_async`.
    pub fn async(&self, dothread: bool) -> Result<()> {
        unsafe { into_result!(sys::ub_ctx_async(self.ub_ctx, dothread as libc::c_int)) }
    }
    /// Wraps `ub_poll`.
    pub fn poll(&self) -> bool {
        unsafe { sys::ub_poll(self.ub_ctx) != 0 }
    }
    /// Indicates whether there are any unprocessed asynchronous queries remaining.
    pub fn have_waiting(&self) -> bool {
        !self.callbacks.lock().unwrap().is_empty()
    }
    /// Reimplements `ub_wait`.
    pub fn wait(&self) -> Result<()> {
        unsafe {
            let _ = sys::ub_wait; // Reimplemented to go through self.process (and the lock)
            let mut set: libc::fd_set = mem::uninitialized();
            let fd = self.fd();
            libc::FD_ZERO(&mut set);
            libc::FD_SET(fd, &mut set);
            let nfds = fd + 1;
            let nil_set = 0 as *mut _;
            let nil_tv = 0 as *mut _;
            while self.have_waiting() {
                libc::select(nfds, &mut set, nil_set, nil_set, nil_tv);
                // Why call process blindly?
                //  1 - Ready: work to be done.
                //  0 - Timeout: process() won't block.
                // -1 - Error: no errno to check, if it's non-temporary process() should return it.
                try!(self.process())
            }
            Ok(())
        }
    }
    /// Wraps `ub_fd`.
    pub fn fd(&self) -> libc::c_int {
        unsafe { sys::ub_fd(self.ub_ctx) }
    }
    /// Wraps `process`.
    pub fn process(&self) -> Result<()> {
        let _ = self.callbacks.lock().unwrap();
        unsafe { into_result!(sys::ub_process(self.ub_ctx)) }
    }
    /// Wraps `ub_resolve`.
    pub fn resolve(&self, name: &str, rrtype: u16, class: u16) -> Result<Answer> {
        let mut result: *mut sys::Struct_ub_result = ptr::null_mut();
        let name = try!(CString::new(name));
        unsafe {
            into_result!(sys::ub_resolve(self.ub_ctx,
                                         name.as_ptr(),
                                         rrtype as libc::c_int,
                                         class as libc::c_int,
                                         &mut result),
                         Answer(result))
        }
    }
    /// Wraps `ub_resolve_async`.
    pub fn resolve_async<C>(&self,
                            name: &str,
                            rrtype: u16,
                            class: u16,
                            callback: C)
                            -> Result<AsyncID>
        where C: Fn(Result<Answer>) + 'static
    {
        let name = try!(CString::new(name));
        let mut hm = self.callbacks.lock().unwrap();
        let mut ctx = CallbackContext::new(&mut hm, callback);
        let id_raw = ctx.id_raw();
        let ctx_raw = ctx.into_raw();
        unsafe {
            let result = into_result!(sys::ub_resolve_async(self.ub_ctx,
                                                            name.as_ptr(),
                                                            rrtype as libc::c_int,
                                                            class as libc::c_int,
                                                            ctx_raw,
                                                            Some(rust_unbound_callback),
                                                            id_raw),
                                      AsyncID(*id_raw));
            if result.is_ok() {
                hm.insert(*id_raw, ctx_raw);
            } else {
                drop(CallbackContext::from_raw(ctx_raw));
            }
            result
        }
    }
    /// Wraps `ub_cancel`.
    pub fn cancel(&self, id: AsyncID) {
        let mut hm = self.callbacks.lock().unwrap();
        if let Some(ctx_raw) = hm.remove(&id.0) {
            unsafe {
                assert_eq!(sys::ub_cancel(self.ub_ctx, id.0), 0);
                drop(CallbackContext::from_raw(ctx_raw));
            }
        }
    }
    /// Wraps `ub_ctx_print_local_zones`.
    pub fn print_local_zones(&self) -> Result<()> {
        unsafe { into_result!(sys::ub_ctx_print_local_zones(self.ub_ctx)) }
    }
    /// Wraps `ub_ctx_zone_add`.
    pub fn zone_add(&self, zone_name: &str, zone_type: &str) -> Result<()> {
        let n = try!(CString::new(zone_name));
        let t = try!(CString::new(zone_type));
        unsafe { into_result!(sys::ub_ctx_zone_add(self.ub_ctx, n.as_ptr(), t.as_ptr())) }
    }
    /// Wraps `ub_ctx_zone_remove`.
    pub fn zone_remove(&self, zone_name: &str) -> Result<()> {
        let n = try!(CString::new(zone_name));
        unsafe { into_result!(sys::ub_ctx_zone_remove(self.ub_ctx, n.as_ptr())) }
    }
    /// Wraps `ub_ctx_data_add`.
    pub fn data_add(&self, data: &str) -> Result<()> {
        let data = try!(CString::new(data));
        unsafe { into_result!(sys::ub_ctx_data_add(self.ub_ctx, data.as_ptr())) }
    }
    /// Wraps `ub_ctx_data_remove`.
    pub fn data_remove(&self, data: &str) -> Result<()> {
        let data = try!(CString::new(data));
        unsafe { into_result!(sys::ub_ctx_data_remove(self.ub_ctx, data.as_ptr())) }
    }
}

impl fmt::Debug for Context {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_fmt(format_args!("Context({:p})", self))
    }
}

unsafe extern "C" fn rust_unbound_callback(ctx_raw: *mut libc::c_void,
                                           error: libc::c_int,
                                           result: *mut sys::Struct_ub_result) {
    CallbackContext::from_raw(ctx_raw).call_and_remove(into_result!(error, Answer(result)));
}


/// Identifies an asynchronous query.
// TODO: Copy? .cancel() ?
#[derive(Clone, Debug)]
pub struct AsyncID(libc::c_int);

impl std::cmp::PartialEq for AsyncID {
    fn eq(&self, other: &AsyncID) -> bool {
        self.0 == other.0
    }
}

type ContextHashMap = HashMap<libc::c_int, *mut libc::c_void>;

struct CallbackContext {
    inner: Box<CallbackContextInner>,
}

struct CallbackContextInner(libc::c_int, *mut ContextHashMap, Box<Fn(Result<Answer>)>);

impl CallbackContext {
    fn new<C>(chm: &mut ContextHashMap, cb: C) -> Self
        where C: 'static + Fn(Result<Answer>)
    {
        let inner = CallbackContextInner(0, chm, Box::new(cb));
        CallbackContext { inner: Box::new(inner) }
    }
    unsafe fn from_raw(raw: *mut libc::c_void) -> Self {
        CallbackContext { inner: Box::from_raw(raw as *mut CallbackContextInner) }
    }
    fn into_raw(self) -> *mut libc::c_void {
        Box::into_raw(self.inner) as *mut libc::c_void
    }
    fn id_raw(&mut self) -> *mut libc::c_int {
        &mut self.inner.0 as *mut _
    }
    fn call_and_remove(&self, result: Result<Answer>) {
        unsafe {
            mem::transmute::<_, &mut ContextHashMap>(self.inner.1).remove(&self.inner.0);
            self.inner.2(result);
        }
    }
}

unsafe impl Sync for Context{}
unsafe impl Send for Context{}

impl Drop for Context {
    fn drop(&mut self) {
        unsafe {
            sys::ub_ctx_delete(self.ub_ctx);
            for &ctx_raw in self.callbacks.lock().unwrap().values() {
                drop(CallbackContext::from_raw(ctx_raw));
            }
        }
    }
}

/// Wraps `ub_version`.
pub fn version() -> &'static str {
    unsafe { CStr::from_ptr(sys::ub_version()).to_str().unwrap() }
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
    assert!(ctx.set_fwd("8.8.8.8").is_ok());
    assert!(ctx.set_fwd("!").is_err());
    assert!(ctx.hosts().is_ok());
    assert!(ctx.hosts_path("test/empty").is_ok());
    assert!(ctx.hosts_path("test/no-such-file").is_err());
}
