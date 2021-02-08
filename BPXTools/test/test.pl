#!/usr/bin/perl

use Digest::SHA qw(sha256);

opendir my $dir, "test/available" or die "Cannot open directory: $!";
my @files = sort { $a cmp $b } readdir $dir;
closedir $dir;

foreach $a (@files) {
    if (!($a =~ /\./)) {
        print "$a\n";
    }
}
