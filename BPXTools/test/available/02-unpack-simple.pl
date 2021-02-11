$Test = {
    Name => "Unpack (SIMPLE)",
    Command => "-f test/available/test.bpx unpack",
    Description => "Test the unpack command",
    Status => 0
};

sub TestBegin {

}

sub TestEnd {
    my $res = EnsureEqual("LICENSE.txt", "test/LICENSE.txt");
    unlink("LICENSE.txt");
    unlink("test/LICENSE.txt");
    return $res;
}
