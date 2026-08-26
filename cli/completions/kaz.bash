# bash completion for kaz. Hand-written (D-C8: help is a product surface); the
# packaging test guards it against drift with the subcommand table.
#
# Install: source this file, or drop it in a bash-completion.d directory.

_kaz() {
    local cur prev
    cur="${COMP_WORDS[COMP_CWORD]}"
    prev="${COMP_WORDS[COMP_CWORD-1]}"

    local subcommands="line scatter bar hist count density ecdf box violin hist2d heatmap"
    local flags="-o -O -d -H --fmt -w -h -t --xlabel --ylabel --xlim --ylim \
        --log-x --log-y --time-x --bins --colormap --midpoint --log-color --labels-x --labels-y --reduce --cols --by --emit-code \
        --color --charset --pixels -q \
        --live --window --fps --rate --version --help"

    case "$prev" in
        --color|--pixels)
            COMPREPLY=( $(compgen -W "auto always never" -- "$cur") ); return ;;
        --charset)
            COMPREPLY=( $(compgen -W "auto ascii half quad sextant braille octant" -- "$cur") ); return ;;
        --reduce)
            COMPREPLY=($(compgen -W "mean max min median" -- "$cur"))
            return
            ;;
        --colormap)
            COMPREPLY=( $(compgen -W "viridis magma cividis greys red-blue purple-orange" -- "$cur") ); return ;;
        --fmt)
            COMPREPLY=( $(compgen -W "y xy xyy xyxy yx" -- "$cur") ); return ;;
        -o)
            COMPREPLY=( $(compgen -f -- "$cur") ); return ;;
    esac

    # The first word after `kaz` is the chart subcommand.
    if [ "$COMP_CWORD" -eq 1 ]; then
        COMPREPLY=( $(compgen -W "$subcommands" -- "$cur") )
        return
    fi

    if [[ "$cur" == -* ]]; then
        COMPREPLY=( $(compgen -W "$flags" -- "$cur") )
    else
        COMPREPLY=( $(compgen -f -- "$cur") )
    fi
}

complete -F _kaz kaz
