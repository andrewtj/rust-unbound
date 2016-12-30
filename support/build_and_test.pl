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

foreach my $command (@commands) {
    foreach my $manifest (@manifests) {
        my $exec = "cargo $command --verbose --manifest-path $manifest";
        die "$command $manifest failed" unless system($exec) eq 0;
    }
}
