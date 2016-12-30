#!/usr/bin/env perl
use strict;
use FindBin;
use Cwd 'realpath';
use File::Temp 'tempdir';
use File::Spec::Functions;

my $temp_dir = tempdir(CLEANUP => 1);
my $unbound_dir = realpath(catfile($FindBin::Bin, "..", "unbound"));
my $unbound_manifest = catfile($unbound_dir, "Cargo.toml");

# tutorials may exit non-zero so only consider the exit code of builds
my $result = 0;
for my $i (1..6) {
    $result |= system(
    "cargo build --manifest-path $unbound_manifest --example tutorial$i");
}

my $dnskey = qx/dig -q nlnetlabs.nl -t DNSKEY/;
$_ = $dnskey;
if ( ! m/status: NOERROR/ || ! m/ANSWER: [1-9]/ ) {
    print "Failed to obtain DNSKEY for nlnetlabs.nl:\n$dnskey\n";
} else {
    my $dnskey_file = catfile($temp_dir, "keys");
    open (my $dnskey_fh, '>>', $dnskey_file) or die "Failed to write dnskey";
    print $dnskey_fh $dnskey;
    close($dnskey_fh) or die "Failed to write dnskey";
}

my @examples = (
    "tutorial1",
    "tutorial2",
    "tutorial3 reckoner.com.au thesizzle.com.au",
    "tutorial4",
    "tutorial5",
    "tutorial6",
);

foreach my $example (@examples) {
    my $cmd = qq/
        cd $temp_dir && \
        cargo run --manifest-path $unbound_manifest --example $example
    /;
    system($cmd);
}

exit $result;