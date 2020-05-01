This project is abandoned. To revive it I'd need to find a nice way to handle
libunbound structures changing in backward incompatible ways and rethink how
libunbound's dependencies (OpenSSL and expat) are handled. Some combination of
bindgen at build time and the approach to dependencies the curl-sys crate uses
might be appropriate.

If you would like to take ownership of the unbound and unbound-sys crate names
you'll either need to be from the unbound project or show that you both have a
plan for the crates and will execute on that plan.

---

# rust-unbound

License: [BSD 3-clause](LICENSE)

[libunbound](https://unbound.nlnetlabs.nl) is an implementation of a DNS
resolver, including cache and DNSSEC validation. Contained here are two Rust
crates for working with libunbound:

* [unbound-sys](unbound-sys) provides unsafe wrappers to libunbound
* [unbound](unbound) implements a safe wrapper atop unbound-sys

Please see their respective READMEs for further information.
