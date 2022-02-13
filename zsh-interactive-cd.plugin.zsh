#!/usr/bin/env zsh
#
# Copyright 2017-2018 Henry Chang and 2021 Maneren
#
# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

zic-completion() {
setopt localoptions noshwordsplit noksh_arrays noposixbuiltins

local tokens=(${(z)LBUFFER})
local cmd="${tokens[1]}"

    # if the command isn't cd
    # or if the there is no space after the cd
    # implying that user wanted to complete
    # the command name rather that path
    # then use ZSH's default completion
    [[ "$cmd" != "cd" || LBUFFER =~ "^cd$" ]] && {
      zle $__zic_default_completion
          return
        }

      local input="${tokens[2,${#tokens}]}"

    # account for special inputs
    input=${input/#%\~/"$HOME/"} # if $input is only "~"
    input=${input/#\~/$HOME} # if $input starts with "~"
    input=${input/#./"./."} # if $input starts with "."

    _zic_complete $input
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

#################
### functions ###
#################

_zic_complete() {
  setopt localoptions nonomatch

  local list=$(__zic_matched_subdir_list ${(Q)1} | xargs -n 1 | sort -fiu)

  [ -z "$list" ] && {
    zle $__zic_default_completion
      return
    }

    # if there is only one match return it
    # else run fzf
    local match=$(
    [ $(wc -l <<< "$list") = 1 ] \
      && echo "${(q)list}" \
      || fzf --height "40%" --reverse --bind $(__zic_fzf_bindings) <<< "$list"
    )

    match=${match% } # remove trailing space
    [ -n "$match" ] && __zic_show_result "$match"

    zle redisplay
    typeset -f zle-line-init >/dev/null && zle zle-line-init
  }

__zic_show_result() {
  local tokens=(${(z)LBUFFER}) # split LBUFFER into words
  local cmd="${tokens[1]}"
  local input="${tokens[2]}"

  local base="$input"

    # if user enters 'path/to/fold' remove the 'fold'
    # so 'folder' can be just simply appended later
    [[ "$base" != */ ]] && base=$([[ "$base" == */* ]] && echo "$(dirname -- "$base")/")


    [ -n "$base" ] && base="${(q)base}" # add quotes if needed and escape chars

    base="${base/#'\~'/~}" # unescape starting tilde

    # append match to LBUFFER (base always ends with a '/')
    LBUFFER="${cmd} ${base}${match}/"
  }

__zic_matched_subdir_list() {
  # constructs a regex and calls __zic_list_subdirs
  # if input is full path, then then return all subdirs
  # else try searching for subdirs that start with input
  # or for those that include the input as substring as a last resort

    # $zic_case_insensitive and $zic_ignore_dot applies to the search

    local regex # __zic_list_subdirs prefixes with '^' and suffixes with '.*$'

    # if ends with /
    [[ "$1" == */ ]] && {
      local dir="$1"

        # if $dir isn't just /, remove the traling /
        [ "$dir" != / ] && dir="${dir:0:-1}"

        regex=$([ "$zic_ignore_dot" = "true" ] && echo "." || echo "[^.]")

        __zic_list_subdirs "$dir" "$regex"

        return
      }

    local seg=$(basename -- "$1")
    local dir=$(dirname -- "$1")

    # escape characters in the basename to be regex-safe
    # (can be bypassed, but with chars that can't be in filnames anyway)
    local escaped=$(__zic_regex_escape $seg)


    regex=$([ "$zic_ignore_dot" = "true" ] && echo "[.]?$escaped" || echo "$escaped")

    local starts_with_seg=$(__zic_list_subdirs "$dir" "$regex")

    [ -n "$starts_with_seg" ] && {
      echo "$starts_with_seg"
          return
        }

    # if first character of input ($1) is .,
    # force starting . in the regex
    if [ "${seg[1]}" = "." ]; then
      escaped=$(__zic_regex_escape "${seg:1}")
      regex="[.].*$escaped"
    elif [ "$zic_ignore_dot" = "true"  ]; then
      regex=".*$escaped"
    else
      regex="[^.].*$escaped"
    fi

    __zic_list_subdirs "$dir" "$regex"
  }

__zic_fzf_bindings() {
  autoload is-at-least

  local version=$(fzf --version)
  local optional=$(is-at-least '0.21.0' $version && echo ',bspace:backward-delete-char/eof')

  echo "shift-tab:up,tab:down${optional}"
}

__zic_list_subdirs() {
  local base="$1"
  local length=$(__zic_calc_lenght "$base")

  local find_opts=$(
  [ "$zic_case_insensitive" = "true" ] && echo "-iregex" || echo "-regex"
)

local escaped=$([ "$base" != "/" ] && __zic_regex_escape "$base")
local regex="^${escaped}[/]$2.*$"

    # lists subdirs
    # removes base path
    # filters by the regex
    find -L "$base" -regextype "posix-extended" $find_opts "$regex" \
      -mindepth 1 -maxdepth 1 -type d 2>/dev/null \
      | command cut -b $(( ${length} + 2 ))-
    }

  __zic_regex_escape() {
    sed 's/[^^]/[&]/g; s/\^/\\^/g' <<< "$1"
  }

__zic_calc_lenght() {
  [ "$1" = "/" ] && echo 0 || { printf "$1" | wc -c }
}
