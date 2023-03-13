#!/usr/bin/env zsh
#
# Copyright 2017-2018 Henry Chang and 2021-2022 Maneren
#
# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

__zic_bin_path="${0:a:h}/bin"
if [[ "$PATH" != *"$__zic_bin_path"* ]]; then
    PATH="$PATH:$__zic_bin_path"
fi
unset __zic_bin_path

if ! which zic-list-dirs 2>&1 >/dev/null; then
    echo "zsh-interactive-cd: Binary not found" >&2
    return 1
fi

zic-setup() {
    local shortcut="$1"

    [ -z $shortcut ] && shortcut='^I' # default is TAB

    if [ -z $__zic_default_completion ]; then
        local binding=$(bindkey $shortcut)

        # if the key isn't bound to anything use ZSH's default completion
        # else use the set completion
        __zic_default_completion=$(
            [ $binding = 'undefined-key' ] && echo "expand-or-complete" || echo $binding[(s: :w)2]
        )
    fi

    bindkey $shortcut __zic-completion
}

function __zic-completion() {
    if [[ "$LBUFFER" != "cd "* ]]; then
        zle $__zic_default_completion
        return
    fi

    local output
    output=$(zic-list-dirs "$LBUFFER" "${(e)LBUFFER}") # second argument is with expanded variables

    if [ ! $? = 0 ]; then
        zle $__zic_default_completion
        return
    fi

    if [ "$LBUFFER" = "$output" ]; then
        return
    fi

    LBUFFER="${output}"

    zle redisplay
    typeset -f zle-line-init >/dev/null && zle zle-line-init
}

zle -N __zic-completion
