$Test = {
    Name => "Unpack (COMPLEX)",
    Command => "-f test.bpx unpack",
    Description => "Test the unpack command",
    Status => 0
};

sub TestBegin {

}

sub TestEnd {
    my $res = EnsureEqual("bpxdbg", "test/bpxdbg");
    unlink("bpxdbg");
    unlink("test/bpxdbg");
    return $res;
}
