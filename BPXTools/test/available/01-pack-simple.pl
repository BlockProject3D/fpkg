$Test = {
    Name => "Pack (SIMPLE)",
    Command => "-f test.bpx pack ../LICENSE.txt",
    Description => "Test the pack command",
    Status => 0
};

sub TestBegin {
}

sub TestEnd {
    return EnsureEqual("test.bpx", "test/available/test.bpx");
}
