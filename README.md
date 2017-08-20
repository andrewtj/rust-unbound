# rust-unbound

License: [BSD 3-clause](LICENSE)

[libunbound](https://unbound.nlnetlabs.nl) is an implementation of a DNS
resolver, including cache and DNSSEC validation. Contained here are two Rust
crates for working with libunbound:

* [unbound-sys](unbound-sys) provides unsafe wrappers to libunbound
* [unbound](unbound) implements a safe wrapper atop unbound-sys

Please see their respective READMEs for further information.
