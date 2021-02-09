$Test = {
    Name => "Info (Section)",
    Command => "-f test/available/test.bpx info -d 0",
    Description => "Test the info command",
    Status => 1
};

sub TestBegin {

}

sub TestEnd {
    return 1;
}
