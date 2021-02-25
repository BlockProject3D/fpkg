fpkg.project({
    name = "test-zlib-static",
    version = "1.0.0",
    description = "zlib static library FPKG package",
    configurations = {"Debug", "Release"}
})

fpkg.addLuaPath("./")
local cmake = require("cmake")

function Project:build(profile)
    if not(file.isDirectory("zlib")) then
        command.run("git", {"clone", "https://github.com/madler/zlib.git"})
    end
    cmake.build(profile, {
        sourceDirectory = "zlib",
        cmakeArgs = {
            "-DCMAKE_POSITION_INDEPENDENT_CODE=ON"
        },
        target = "zlibstatic"
    })
end

function Project:package(profile)
    cmake.buildAllConfigurations(profile, {
        sourceDirectory = "zlib",
        cmakeArgs = {
            "-DCMAKE_POSITION_INDEPENDENT_CODE=ON"
        },
        targets = {"zlibstatic", "install"}
    })
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
