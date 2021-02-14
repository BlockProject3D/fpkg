PackageInfo = {
    Name = "test-zlib-static",
    Version = "1.0.0",
    Description = "zlib static library FPKG package",
    Configurations = {"Debug", "Release"}
}

function Build(profile)
    if not(file.IsDirectory("zlib")) then
        command.Run("git", {"clone", "https://github.com/madler/zlib.git"})
    end
    if not(file.IsDirectory("zlib-"..profile.Configuration)) then
        command.Run("cmake", {"-S", "zlib", "-B", "zlib-"..profile.Configuration, "-DCMAKE_BUILD_TYPE="..profile.Configuration, "-DCMAKE_INSTALL_PREFIX="..profile.Configuration})
    end
    command.Run("cmake", {"--build", "zlib-"..profile.Configuration, "--config", profile.Configuration, "--target", "zlibstatic"})
end

function Package(profile)
    for _, v in pairs(PackageInfo.Configurations) do
        local tbl = profile;
        tbl.Configuration = v;
        Build(tbl)
        command.Run("cmake", {"--build", "zlib-"..v, "--target", "install"})
    end
    local target = {
        Type = "Library", --Either Library or Framework
        Includes = { --Only for Library targets
            {"./Release/include", "Release"}, --Relative path, configuration type
            {"./Debug/include", "Debug"}
        },
        Binaries = { --Only for Library targets
            {"./Debug/lib/libz.a", "Debug"}, --Relative path, configuration type
            {"./Release/lib/libz.a", "Release"}
        }
    }
    return target
end
