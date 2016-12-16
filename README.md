# rust-unbound

License: [BSD 3-clause](LICENSE)

## Overview

rust-unbound provides unsafe FFI bindings and a safe wrapper for
[libunbound](https://unbound.nlnetlabs.nl/).

* [Documentation](http://www.andrewtj.org/rust-unbound/unbound/)
* [Examples](unbound/examples)

## Building

rust-unbound wraps libunbound. libunbound depends on OpenSSL which this
crate relies on [rust-openssl](https://github.com/sfackler/rust-openssl)
to provide.

The following environment variables influence the build process:

* `UNBOUND_STATIC`- If specified libunbound will be linked statically.
* `UNBOUND_DIR` - Directory in which libunbound's `include` and `lib` folders may be found.
