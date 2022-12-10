#!/usr/bin/env zsh
#
# Copyright 2017-2018 Henry Chang and 2021-2022 Maneren
#
# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

PATH="$PATH:${0:a:h}/bin"

if ! which zic-list-dirs 2>&1 >/dev/null; then
    echo "zsh-interactive-cd: Binary not found" >&2
    return 1
fi

[ -z $zic_custom_binding ] && zic_custom_binding='^I' # default is TAB

if [ -z $__zic_default_completion ]; then
    __binding=$(bindkey $zic_custom_binding)

    # if the key isn't bound to anything use ZSH's default completion
    # else use the set completion
    __zic_default_completion=$(
        [ $__binding = 'undefined-key' ] && echo "expand-or-complete" || echo $__binding[(s: :w)2]
    )

    unset __binding
fi

function zic-completion() {
    local output
    output=$(zic-list-dirs "$LBUFFER" "${(e)LBUFFER}") # second argument is with expanded variables

    if [ ! $? = 0 ]; then
        zle $__zic_default_completion
        return
    fi

    LBUFFER="${output}"

    zle redisplay
    typeset -f zle-line-init >/dev/null && zle zle-line-init
}

zle -N zic-completion
bindkey $zic_custom_binding zic-completion
