$Test = {
    Name => "Info (SHT)",
    Command => "-f test/available/test.bpx info -s",
    Description => "Test the info command",
    Status => 0
};

sub TestBegin {

}

sub TestEnd {
    return 1;
}
