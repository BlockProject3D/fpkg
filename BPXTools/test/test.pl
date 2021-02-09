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

sub hackFixedStatusCodeSystem {
    my $cmdline = $_[0];
    my $incorrect_res = system($cmdline);
    if ($incorrect_res == -1) {
        return -1;
    } elsif ($incorrect_res & 127) {
        return $incorrect_res & 127;
    } else {
        return $incorrect_res >> 8;
    }
}

foreach $a (@files) {
    if (!($a =~ /\./)) {
        require "$cwd/test/available/$a.pl";
        my $name = $Test->{Name};
        my $desc = $Test->{Description};
        my $cmd = $Test->{Command};
        my $status = $Test->{Status};
        print "Running test: $name - $desc\n";
        TestBegin();
        my $res = hackFixedStatusCodeSystem("./target/debug/bpxdbg $cmd 1>mybin.stdout 2>mybin.stderr");
        if ($res != $status) {
            print "Bad exit status: expected $status, got $res\n";
            print "\e[1;31m\t> Test Failure\e[0m\n";
            next;
        }
        my $stdout = "$cwd/test/available/$a.stdout";
        my $stderr = "$cwd/test/available/$a.stderr";
        if (-e $stdout) {
            if (!EnsureEqual($stdout, "mybin.stdout")) {
                print "\e[1;31m\t> Test Failure\e[0m\n";
                next;
            }
        }
        if (-e $stderr) {
            if (!EnsureEqual($stderr, "mybin.stderr")) {
                print "\e[1;31m\t> Test Failure\e[0m\n";
                next;
            }
        }
        if (!TestEnd()) {
            print "\e[1;31m\t> Test Failure\e[0m\n";
            next;
        }
        print "\e[1;32m\t> Test Passed\e[0m\n";
    }
}
