$Test = {
    Name => "Info (Metadata + Hex)",
    Command => "-f test/available/test.bpx info -smx",
    Description => "Test the info command",
    Status => 0
};

sub TestBegin {

}

sub TestEnd {
    return 1;
}
