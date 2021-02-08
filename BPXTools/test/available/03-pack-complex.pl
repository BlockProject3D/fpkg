use File::Copy;

$Test = {
    Name => "Pack (COMPLEX)",
    Command => "-f test.bpx pack test/bpxdbg",
    Description => "Test the pack command",
    Status => 0
};

sub TestBegin {
    copy("target/debug/bpxdbg", "test/bpxdbg");
}

sub TestEnd {
    return 1;
}
