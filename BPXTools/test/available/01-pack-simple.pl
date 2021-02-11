$Test = {
    Name => "Pack (SIMPLE)",
    Command => "-f test.bpx pack test/LICENSE.txt",
    Description => "Test the pack command",
    Status => 0
};

sub TestBegin {
    CRLFToLF("../LICENSE.txt", "test/LICENSE.txt");
}

sub TestEnd {
    return EnsureEqual("test.bpx", "test/available/test.bpx");
}
