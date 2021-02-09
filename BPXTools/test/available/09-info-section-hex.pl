$Test = {
    Name => "Info (Section + Hex)",
    Command => "-f test/available/test.bpx info -d 0 -x",
    Description => "Test the info command",
    Status => 0
};

sub TestBegin {

}

sub TestEnd {
    return 1;
}
