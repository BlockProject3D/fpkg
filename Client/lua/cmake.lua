local lib = {}

function lib.build(profile, info)
    local args = {}
    if (info.buildPrefix == nil) then
        info.buildPrefix = "build"
    end
    local builddir = info.buildPrefix.."-"..profile.configuration
    if not(file.isDirectory(builddir)) then
        if not(info.sourceDirectory == nil) then
            table.insert(args, "-S")
            table.insert(args, info.sourceDirectory)
        end
        table.insert(args, "-B")
        table.insert(args, builddir)
        if not(profile.compilerName == "MSVC") then
            table.insert(args, "-DCMAKE_BUILD_TYPE="..profile.configuration)
        end
        table.insert(args, "-DCMAKE_INSTALL_PREFIX="..profile.configuration)
        if not(info.cmakeArgs == nil) then
            for _,v in pairs(info.cmakeArgs) do
                table.insert(args, v)
            end
        end
        command.run("cmake", args)
    end
    if not(info.targets == nil) then
        for _, v in pairs(info.targets) do
            args = {"--build", builddir, "--config", profile.configuration, "--target", v}
            command.run("cmake", args)
        end
    elseif not(info.target == nil) then
        args = {"--build", builddir, "--config", profile.configuration, "--target", info.target}
        command.run("cmake", args)
    else
        args = {"--build", builddir, "--config", profile.configuration}
        command.run("cmake", args)
    end
end

function lib.buildAllConfigurations(profile, info)
    for _, v in pairs(Project.configurations) do
        profile.configuration = v
        lib.build(profile, info)
    end
end

return lib