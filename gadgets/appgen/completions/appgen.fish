function __fish_appgen_no_subcommand
    set cmd (commandline -opc)
    if [ (count $cmd) -eq 1 ]
        return 0
    end
    return 1
end

# Options
complete -c appgen -s e -l executable -d "Path to the executable file to package" -r -f -a "(__fish_complete_path)"
complete -c appgen -s n -l name -d "Name of the app (without .app extension)" -r
complete -c appgen -s i -l icon -d "Optional icon file path (.icns format)" -r -f -a "*.icns"
complete -c appgen -s v -l app-version -d "Optional app version" -r -f -a "1.0.0"
complete -c appgen -s b -l bundle-id -d "Optional bundle identifier" -r -f -a "com.example.app"
complete -c appgen -s o -l output -d "Output directory" -r -f -a "(__fish_complete_directories)"
complete -c appgen -s a -l additional-file -d "Additional files to include" -r -f -a "(__fish_complete_path)"
complete -c appgen -s d -l default-location -d "Default location for additional files" -r -f -a "resources macos contents"
complete -c appgen -s t -l show-terminal -d "Show terminal window when the application runs" -f
complete -c appgen -s s -l single-instance -d "Enable single instance mode" -f
complete -c appgen -s h -l help -d "Show help message" -f
complete -c appgen -s V -l version -d "Show version information" -f
