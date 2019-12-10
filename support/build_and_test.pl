#!/usr/bin/env perl
use strict;
use FindBin;
use Cwd 'realpath';
use File::Spec::Functions;

my $workspace_dir = realpath(catfile($FindBin::Bin, ".."));
my @commands = ("build", "test");
my @manifests = (
    catfile($workspace_dir, "unbound", "Cargo.toml"),
    catfile($workspace_dir, "unbound-sys", "Cargo.toml"),
);
my @features = (
    'ub_ctx_set_tls',
    'ub_ctx_set_stub',
    'ub_ctx_add_ta_autr',
);

foreach my $command (@commands) {
    foreach my $manifest (@manifests) {
        foreach my $feature (@features) {
            my $exec = "cargo $command --verbose --manifest-path $manifest --features $feature --lib";
            die "$command $manifest $feature failed" unless system($exec) eq 0;
        }
    }
}
