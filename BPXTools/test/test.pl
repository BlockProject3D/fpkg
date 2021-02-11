#!/usr/bin/perl

use Digest::SHA;
use Cwd;

opendir my $dir, "test/available" or die "Cannot open directory: $!";
my @files = sort { $a cmp $b } readdir $dir;
closedir $dir;

my $cwd = getcwd;
print "Current working directory: $cwd\n";

sub CRLFToLF {
    my $file1 = $_[0];
    my $file2 = $_[1];

    open my $in, '<:raw:crlf', $file1 or die $!;
    open my $out, '>:raw', $file2 or die $!;
    print {$out} $_ while <$in>;
    close($in);
    close($out);
}

sub EnsureEqual {
    my $file1 = $_[0];
    my $file2 = $_[1];
    my $sha1 = Digest::SHA->new;
    my $sha2 = Digest::SHA->new;

    if (!(-e $file1) || !(-e $file2)) {
        print "Files $file1 and $file2 differ\n";
        return 0;
    }
    $sha1->addfile($file1);
    $sha2->addfile($file2);
    if ($sha1->hexdigest != $sha2->hexdigest) {
        print "Files $file1 and $file2 differ\n";
        return 0;
    }
    return 1;
}

sub hackFixedStatusCodeSystem {
    my $cmdline = $_[0];

    #Again another reason why perl is bad: all other high level languages and even C++ uses a single comparision operator for eveything
    #Perl must make things more complicated than they are supposed to be! F*** you perl!
    if ($^O eq "MSWin32") {
        $cmdline =~ s/\//\\/g; #Windows CMD hack
    }

    my $incorrect_res = system($cmdline);

    if ($incorrect_res == -1) {
        return -1;
    } elsif ($incorrect_res & 127) {
        return $incorrect_res & 127;
    } else {
        return $incorrect_res >> 8;
    }
}

my $endres = 0;

foreach $b (@files) {
    if ($b =~ /\.pl/) {
        @arr = split(/\./, $b);
        my $a = $arr[0];
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
            $endres = 1;
            next;
        }
        my $stdout = "$cwd/test/available/$a.stdout";
        my $stderr = "$cwd/test/available/$a.stderr";
        if (-e $stdout) {
            if (!EnsureEqual($stdout, "mybin.stdout")) {
                print "\e[1;31m\t> Test Failure\e[0m\n";
                $endres = 1;
                next;
            }
        }
        if (-e $stderr) {
            if (!EnsureEqual($stderr, "mybin.stderr")) {
                print "\e[1;31m\t> Test Failure\e[0m\n";
                $endres = 1;
                next;
            }
        }
        if (!TestEnd()) {
            print "\e[1;31m\t> Test Failure\e[0m\n";
            $endres = 1;
            next;
        }
        print "\e[1;32m\t> Test Passed\e[0m\n";
    }
}

unlink("mybin.stdout");
unlink("mybin.stderr");

exit($endres);
