#!/usr/bin/env zsh
#
# Copyright 2017-2018 Henry Chang
#
# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

__zic_fzf_prog() {
    if [[ -n "$TMUX_PANE" && "${FZF_TMUX:-0}" != 0 && ${LINES:-40} -gt 15 ]]; then
        echo "fzf-tmux -d${FZF_TMUX_HEIGHT:-40%}"
    else
        echo "fzf"
    fi
}

__zic_calc_lenght() {
    if [ "$1" = "/" ]; then
        echo 0
    else
        echo $(echo -n "$1" | wc -c)
    fi
}

__zic_list_subdirs() {
    local length=$(__zic_calc_lenght "$1")

    # lists subdirs
    # removes base path
    # removes empty lines
    local subdirs=$(
        find -L "$1" -mindepth 1 -maxdepth 1 -type d 2>/dev/null \
        | cut -b $(( ${length} + 2 ))- \
        | command sed '/^$/d'
    )

    echo "$subdirs"
}

__zic_matched_subdir_list() {
    if [ "$zic_case_insensitive" = "true" ]; then
        setopt nocasematch
    fi
    
    if [[ "$1" == */ ]]; then
        local dir="$1"
        
        if [[ "$dir" != / ]]; then
            dir="${dir: : -1}"
        fi
             
        local subdirs=($(echo $(__zic_list_subdirs "$dir") | xargs -n 1))

        local regex;
        if [ "$zic_ignore_dot" == "true" ]; then
            regex="^.*$"
        else
            regex="^[^\.].*$"
        fi
        
        for line ($subdirs); do
            [[ "$line" =~ "$regex" ]] && echo "$line"
        done
        
        return
    fi
    
    local seg=$(basename -- "$1")
    local dir=$(dirname -- "$1")
    local subdirs=($(echo $(__zic_list_subdirs "$dir") | xargs -n 1))
    
    local starts_with_seg=$(
        local regex;
        if [ "$zic_ignore_dot" == "true" ]; then
            regex="^\.?$seg.*$"
        else
            regex="^$seg.*$"
        fi
        
        for line ($subdirs); do
            [[ "$line" =~ "$regex" ]] && echo "$line"
        done
    )
    
    if [ -n "$starts_with_seg" ]; then
        echo "$starts_with_seg"
        return
    fi
    
    local regex;
    if [[ "$zic_ignore_dot" == "true" || $seg[1] == "." ]]; then
        regex="^.*$seg.*$"
    else
        regex="^[^\.].*$seg.*$"
    fi
    
    for line ($subdirs); do
        [[ "$line" =~ "$regex" ]] && echo "$line"
    done
}

__zic_fzf_bindings() {
    autoload is-at-least
    local fzf=$(__zic_fzf_prog)
    
    if $(is-at-least '0.21.0' $(${=fzf} --version)); then
        echo 'shift-tab:up,tab:down,bspace:backward-delete-char/eof'
    else
        echo 'shift-tab:up,tab:down'
    fi
}

_zic_list_generator() {
    __zic_matched_subdir_list "${(Q)@[-1]}" | sort -fiu
}

_zic_complete() {
    setopt localoptions nonomatch
    
    local list=$(_zic_list_generator $@)
    
    if [ -z "$list" ]; then
        zle ${__zic_default_completion:-expand-or-complete}
        return
    fi
    
    local fzf=$(__zic_fzf_prog)
    local fzf_bindings=$(__zic_fzf_bindings)
    
    local match
    if [ $(echo $list | wc -l) == 1 ]; then
        match=${(q)list}
    else
        local fzf_opts="--height ${FZF_TMUX_HEIGHT:-40%} \
            --reverse $FZF_DEFAULT_OPTS $FZF_COMPLETION_OPTS \
            --bind '${fzf_bindings}'"
        
        # call fzf with $list of options and save the quoted result on single line
        match=$(
            echo -n ${(q)"$(echo $list | FZF_DEFAULT_OPTS=$fzf_opts ${=fzf})"}
        )
    fi
    
    match=${match% } # remove trailing space
    if [ -n "$match" ]; then
        local tokens=(${(z)LBUFFER}) # split LBUFFER into words
        local command="${tokens[1]}"
        local input="${tokens[2]}"

        local base="${@}"

        # if user enters `path/to/fold` remove the `fold`
        # so `folder` can be just simply appended later
        if [[ "$base" != */ ]]; then
            if [[ "$base" == */* ]]; then
                base="$(dirname -- "$base")"

                if [[ "${base[-1]}" != / ]]; then
                    base="$base/"
                fi
            else
                base=""
            fi
        fi

        # properly format $base
        if [ -n "$base" ]; then
            base="${(q)base}" # add quotes and escapes

            # if input path starts with ~, then use the ~ in output
            if [ "${input[1]}" = "~" ]; then
                base="${base/#$HOME/~}"
            fi
        fi
        
        # append match to LBUFFER (base ends with a `/`)
        LBUFFER="${command} ${base}${match}/"
    fi

    zle redisplay
    typeset -f zle-line-init >/dev/null && zle zle-line-init
}

zic-completion() {
    setopt localoptions noshwordsplit noksh_arrays noposixbuiltins
    
    local tokens=(${(z)LBUFFER})
    local cmd=${tokens[1]}
    local input=${tokens[2,${#tokens}]/#\~/"$HOME/"}
    
    local regex='^\ *cd$'
    if [[ "$cmd" != "cd" || "$LBUFFER" =~ "$regex" ]]; then
        zle ${__zic_default_completion:-expand-or-complete}
    else
        _zic_complete $input
    fi
}

[ -z "$__zic_default_completion" ] && {
    binding=$(bindkey '^I')
    # $binding[(s: :w)2]
    # The command substitution and following word splitting to determine the
    # default zle widget for ^I formerly only works if the IFS parameter contains
    # a space via $binding[(w)2]. Now it specifically splits at spaces, regardless
    # of IFS.
    [[ $binding =~ 'undefined-key' ]] || __zic_default_completion=$binding[(s: :w)2]
    unset binding
}

zle -N zic-completion
if [ -z $zic_custom_binding ]; then
    zic_custom_binding='^I'
fi
bindkey "${zic_custom_binding}" zic-completion
