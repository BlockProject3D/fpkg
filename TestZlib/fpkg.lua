Package = {
    ["Name"] = "zlib",
    ["Version"] = "1.0",
    ["Description"] = "zlib library FPKG package",
    ["Configurations"] = {"Debug", "Release"}
}

function Build(tbl)
    command.Run("git", {"clone", "https://github.com/madler/zlib.git"})
    command.Run("cmake", {"-S", "zlib", "-B", "zlib", "-DCMAKE_BUILD_TYPE="..tbl.Configuration})
    command.Run("cmake", {"--build", "zlib", "--config", tbl.Configuration, "--target", "zlib", "--target", "zlibstatic"})
end
