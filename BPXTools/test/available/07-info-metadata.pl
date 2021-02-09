$Test = {
    Name => "Info (Metadata)",
    Command => "-f test/available/test.bpx info -sm",
    Description => "Test the info command",
    Status => 0
};

sub TestBegin {

}

sub TestEnd {
    return 1;
}
