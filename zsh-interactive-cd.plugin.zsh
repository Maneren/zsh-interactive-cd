#!/usr/bin/env zsh
#
# Copyright 2017-2018 Henry Chang and 2021 Maneren
#
# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

zic-completion() {
  local output
  output=$(__zic_matched_subdir_list $LBUFFER)
  
  if [ ! $? = 0 ]; then
    zle $__zic_default_completion
    return
  fi
  
  LBUFFER="${output}"
  
  zle redisplay
  typeset -f zle-line-init >/dev/null && zle zle-line-init
}

[ -z "$__zic_default_completion" ] && {
  __binding=$(bindkey '^I') # TAB key binding
  # if the key isn't bound to anything use ZSH the default completion
  # else use the set completion
  __zic_default_completion=$(
    [[ $__binding =~ 'undefined-key' ]] \
    && echo "expand-or-complete" \
    || echo $__binding[(s: :w)2]
  )
  
  unset __binding
}

zle -N zic-completion
[ -z $zic_custom_binding ] && zic_custom_binding='^I'
bindkey "${zic_custom_binding}" zic-completion

PATH="$PATH:${0:a:h}/bin"