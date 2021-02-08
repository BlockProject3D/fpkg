#!/usr/bin/perl

use File::Copy;

opendir my $dir, "test/available" or die "Cannot open directory: $!";
my @files = sort { $a cmp $b } readdir $dir;
closedir $dir;

my $cwd = substr `pwd`, 0, -1;

sub EnsureEqual {
    my $file1 = $_[0];
    my $file2 = $_[1];
    my $sha1 = `shasum $file1`;
    my $r1 = $?;
    my $sha2 = `shasum $file2`;
    my $r2 = $?;

    if ($r1 != 0 || $r2 != 0 || $sha1 != $sha2) {
        print "Files $file1 and $file2 differ\n";
        return 0;
    }
    return 1;
}

foreach $a (@files) {
    if (!($a =~ /\./)) {
        require "$cwd/test/available/$a.pl";
        my $name = $Test->{Name};
        my $desc = $Test->{Description};
        print "Running test: $name - $desc\n";
        TestBegin();
        
        if (!TestEnd()) {
            print "\e[1;31m\t> Test Failure\e[0m\n";
            next;
        }
        print "\e[1;32m\t> Test Passed\e[0m\n";
    }
}
