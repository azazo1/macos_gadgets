#!/usr/bin/env bash

_appgen_completion() {
    local cur prev words cword
    _init_completion || return

    # Subcommands and options
    case $prev in
        -e|--executable)
            _filedir
            return 0
            ;;
        -n|--name)
            return 0  # User inputs app name
            ;;
        -i|--icon)
            _filedir "icns"
            return 0
            ;;
        -v|--app-version)
            return 0  # User inputs version
            ;;
        -b|--bundle-id)
            return 0  # User inputs bundle ID
            ;;
        -o|--output)
            _filedir -d
            return 0
            ;;
        -a|--additional-file)
            _filedir
            return 0
            ;;
        -d|--default-location)
            COMPREPLY=($(compgen -W "resources macos contents" -- "$cur"))
            return 0
            ;;
        *)
            ;;
    esac

    # Complete options
    if [[ $cur == -* ]]; then
        COMPREPLY=($(compgen -W "-e --executable -n --name -i --icon -v --app-version -b --bundle-id -o --output -a --additional-file -d --default-location -t --show-terminal -s --single-instance -h --help -V --version" -- "$cur"))
        return 0
    fi

    return 0
}

complete -F _appgen_completion appgen
