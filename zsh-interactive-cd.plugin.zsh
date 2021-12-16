#!/usr/bin/env zsh
#
# Copyright 2017-2018 Henry Chang
#
# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

__zic_calc_lenght() {
    if [ "$1" = "/" ]; then
        echo 0
    else
        echo -n "$1" | wc -c
    fi
}

__zic_list_subdirs() {
    local length=$(__zic_calc_lenght "$1")
    
    local grep_opts
    if [ "$zic_case_insensitive" = "true" ]; then
        grep_opts="-i"
    fi
    
    # lists subdirs
    # removes base path
    # filters by the regex
    # TODO: experiment with removing the grep and using only `find -regex`
    find -L "$1" -mindepth 1 -maxdepth 1 -type d 2>/dev/null \
    | command cut -b $(( ${length} + 2 ))- \
    | command grep "$grep_opts" -E "$2" \
}

__zic_regex_escape() {
    sed 's/[^^]/[&]/g; s/\^/\\^/g' <<< "$1"
}

__zic_matched_subdir_list() {
    # if ends with /
    if [[ "$1" == */ ]]; then
        local dir="$1"
        
        # if $dir isn't just /, remove the traling /
        if [[ "$dir" != / ]]; then
            dir="${dir:0:-1}"
        fi
        
        local regex;
        if [ "$zic_ignore_dot" = "true" ]; then
            regex="^.+$"
        else
            regex="^[^.].*$"
        fi
        
        __zic_list_subdirs "$dir" "$regex"
        
        return
    fi
    
    # escape characters in the basename to be regex-safe
    # (can be bypassed, but those chars that can't be in filnames anyway)
    local seg=$(basename -- "$1")
    local dir=$(dirname -- "$1")
    local escaped=$(__zic_regex_escape $seg)
    local regex

    if [ "$zic_ignore_dot" = "true" ]; then
        regex="^[.]?$escaped.*$"
    else
        regex="^$escaped.*$"
    fi

    local starts_with_seg=$(__zic_list_subdirs "$dir" "$regex")
    
    if [ -n "$starts_with_seg" ]; then
        echo "$starts_with_seg"
        return
    fi

    # if first character of input ($1) is .,
    # force starting . in the regex
    if [ "${seg:0:1}" = "." ]; then
        escaped=$(__zic_regex_escape "${seg:1}")
        regex="^[.].*$escaped.*$"
    elif [ "$zic_ignore_dot" = "true"  ]; then
        regex="^.*$escaped.*$"
    else
        regex="^[^.].*$escaped.*$"
    fi
    
    __zic_list_subdirs "$dir" "$regex"
}

__zic_fzf_bindings() {
    autoload is-at-least
    if $(is-at-least '0.21.0' $(fzf --version)); then
        echo 'shift-tab:up,tab:down,bspace:backward-delete-char/eof'
    else
        echo 'shift-tab:up,tab:down'
    fi
}

_zic_list_generator() {
    __zic_matched_subdir_list "${(Q)@[-1]}"
}

_zic_complete() {
    setopt localoptions nonomatch
    
    local list=$(_zic_list_generator $@ | xargs -n 1 | sort -fiu)
    
    if [ -z "$list" ]; then
        zle ${__zic_default_completion:-expand-or-complete}
        return
    fi
    
    local match
    # if there is only one match return it
    # else run fzf
    if [ $(wc -l <<< "$list") = 1 ]; then
        match="${(q)list}"
    else
        local fzf_opts="--height ${FZF_TMUX_HEIGHT:-"40%"} \
            --reverse $FZF_DEFAULT_OPTS $FZF_COMPLETION_OPTS \
            --bind '$(__zic_fzf_bindings)'"
        
        # call fzf with $list of options
        match=$(echo -n $(FZF_DEFAULT_OPTS=$fzf_opts fzf <<< "$list"))
    fi
    
    match=${match% } # remove trailing space
    if [ -n "$match" ]; then
        local tokens=(${(z)LBUFFER}) # split LBUFFER into words
        local cmd="${tokens[1]}"
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
        LBUFFER="${cmd} ${base}${match}/"
    fi
    
    zle redisplay
    typeset -f zle-line-init >/dev/null && zle zle-line-init
}

zic-completion() {
    setopt localoptions noshwordsplit noksh_arrays noposixbuiltins
    
    local tokens=(${(z)LBUFFER})
    local cmd="${tokens[1]}"
    local input="${tokens[2,${#tokens}]}"
    input=${input/#\~/$HOME}
    
    if [ "$cmd" != "cd" ]; then
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
