#!/usr/bin/env perl
use strict;
use File::Temp 'tempfile';

my $preamble = qq {
/* generated by generate_sys.pl */

#![allow(non_camel_case_types)]

extern crate libc;
extern crate openssl;

/**
 * Initialize library.
 */
pub fn init() {
    openssl::init();
}

};

my $CFLAGS = $ENV{CFLAGS} || "";

(my $wrapper_fh, my $wrapper_filename) = tempfile(UNLINK => 1, SUFFIX => ".h");
print $wrapper_fh "#include <unbound.h>\n";
close $wrapper_fh;

# Used servo's bindgen: https://github.com/servo/rust-bindgen
(my $bind_cmd = qq {
    bindgen \
    --ctypes-prefix ::libc \
    --generate functions,types \
    --no-doc-comments \
    $wrapper_filename \
    -- $CFLAGS
}) =~ s/[\n ]+/ /gm;

my $bindings = `$bind_cmd`;
die "bindgen failed - invoked as: $bind_cmd" unless $? eq 0;

my $ub_callback_expect = qq /
pub type ub_callback_t =
    ::std::option::Option<unsafe extern "C" fn(arg1: *mut ::libc::c_void,
                                               arg2: ::libc::c_int,
                                               arg3: *mut ub_result)>;
/;
my $ub_callback_replace = qq /
pub type ub_callback_t = unsafe extern "C" fn(arg1: *mut ::libc::c_void,
                                              arg2: ::libc::c_int,
                                              arg3: *mut ub_result);
/;

if (index($bindings, $ub_callback_expect) == -1) {
    die "ub_callback output has changed - bindings:\n$bindings";
}

$ub_callback_expect = quotemeta $ub_callback_expect;
$bindings =~ s/$ub_callback_expect/$ub_callback_replace/g;

my $ub_ctx_expect = qq {
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ub_ctx {
    _unused: [u8; 0],
}
};
my $ub_ctx_replace = "\npub enum ub_ctx{}\n";

if (index($bindings, $ub_ctx_expect) == -1) {
    die "ub_ctx output has changed - bindings:\n$bindings";
}

$ub_ctx_expect = quotemeta $ub_ctx_expect;
$bindings =~ s/$ub_ctx_expect/$ub_ctx_replace/g;

my $derive_expect = "\n#[derive(Debug, Copy)]\n";
my $derive_replace = "\n#[derive(Debug)]\n";

if (index($bindings, $derive_expect) == -1) {
    die "derive output has changed - bindings:\n$bindings";
}

$derive_expect = quotemeta $derive_expect;
$bindings =~ s/$derive_expect/$derive_replace/g;

$bindings =~ s/impl Clone for.*?}\n}\n//gs;
if (index($bindings, "Clone") != -1) {
    die "clone removal failed - bindings:\n$bindings";
}

foreach my $s ("ub_ctx_add_ta_autr", "ub_ctx_set_stub") {
    my $expect = "pub fn $s";
    my $replace = "#[cfg($s)] pub fn $s";
    if (index($bindings, $expect) == -1) {
        die "binding for $s not found - bindings:\n$bindings";
    }
    $bindings =~ s/$expect/$replace/;
}

print $preamble, $bindings;
