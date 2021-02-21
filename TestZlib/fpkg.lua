fpkg.project({
    name = "test-zlib-static",
    version = "1.0.0",
    description = "zlib static library FPKG package",
    configurations = {"Debug", "Release"}
})

function Project:build(profile)
    if not(file.isDirectory("zlib")) then
        command.run("git", {"clone", "https://github.com/madler/zlib.git"})
    end
    if not(file.isDirectory("zlib-"..profile.configuration)) then
        if not(profile.platform == "Windows") then
            command.run("cmake", {"-S", "zlib", "-B", "zlib-"..profile.configuration, "-DCMAKE_BUILD_TYPE="..profile.configuration, "-DCMAKE_INSTALL_PREFIX="..profile.configuration, "-DCMAKE_POSITION_INDEPENDENT_CODE=ON"})
        else
            command.run("cmake", {"-S", "zlib", "-B", "zlib-"..profile.configuration, "-DCMAKE_BUILD_TYPE="..profile.configuration, "-DCMAKE_INSTALL_PREFIX="..profile.configuration})
        end
    end
    command.run("cmake", {"--build", "zlib-"..profile.configuration, "--config", profile.configuration, "--target", "zlibstatic"})
end

function Project:package(profile)
    for _, v in pairs(self.configurations) do
        local tbl = profile;
        tbl.configuration = v;
        self:build(tbl)
        command.run("cmake", {"--build", "zlib-"..v, "--target", "install", "--config", v})
    end
    local filenamed = "libz.a"
    local filename = "libz.a"
    if (profile.platform == "Windows") then
        filenamed = "zlibstaticd.lib"
        filename = "zlibstatic.lib"
    end
    local target = {
        type = "Library", --Either Library or Framework
        includes = { --Only for Library targets
            {"./Release/include", "Release"}, --Relative path, configuration type
            {"./Debug/include", "Debug"}
        },
        binaries = { --Only for Library targets
            {"./Debug/lib/"..filenamed, "Debug"}, --Relative path, configuration type
            {"./Release/lib/"..filename, "Release"}
        }
    }
    return target
end
