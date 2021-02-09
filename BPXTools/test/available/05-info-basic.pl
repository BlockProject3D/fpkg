$Test = {
    Name => "Info (Basic)",
    Command => "-f test/available/test.bpx info",
    Description => "Test the info command",
    Status => 0
};

sub TestBegin {

}

sub TestEnd {
    return 1;
}
