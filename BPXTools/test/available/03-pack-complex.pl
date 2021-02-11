use File::Copy;

$Test = {
    Name => "Pack (COMPLEX)",
    Command => "-f test.bpx pack test/bpxdbg",
    Description => "Test the pack command",
    Status => 0
};

sub TestBegin {
    #Again another reason why perl is bad: all other high level languages and even C++ uses a single comparision operator for eveything
    #Perl must make things more complicated than they are supposed to be! F*** you perl!
    if ($^O eq "MSWin32") {
        copy("target/debug/bpxdbg.exe", "test/bpxdbg");
    } else {
        copy("target/debug/bpxdbg", "test/bpxdbg");
    }
}

sub TestEnd {
    return 1;
}
