PackageInfo = {
    Name = "zlib-static",
    Version = "1.0",
    Description = "zlib library FPKG package",
    Configurations = {"Debug", "Release"}
}

function Build(profile)
    command.Run("git", {"clone", "https://github.com/madler/zlib.git"})
    command.Run("cmake", {"-S", "zlib", "-B", "zlib-"..profile.Configuration, "-DCMAKE_BUILD_TYPE="..profile.Configuration, "-DCMAKE_INSTALL_PREFIX="..profile.Configuration})
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
        Type = "Library", --Either Library or SDK
        Includes = { --Only for Library targets
            {"./Release/include", "Release"}, --Relative path, configuration type
            {"./Debug/include", "Debug"}
        },
        Binarries = { --Only for Library targets
            {"./Debug/lib", "Debug"}, --Relative path, configuration type
            {"./Release/lib", "Release"}
        }
    }
    return target
end
