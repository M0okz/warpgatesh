# Loaded explicitly by `warpgatesh shell init zsh`.
typeset -gA _warpgatesh_owned

_warpgatesh_refresh() {
  emulate -L zsh
  local listing target body
  local -a targets
  # A failed read preserves the previous functions; this never synchronizes.
  listing=$(command warpgatesh shell aliases) || return
  targets=("${(@f)listing}")
  for target in "${(@k)_warpgatesh_owned}"; do
    if [[ ${functions[$target]-} == ${_warpgatesh_owned[$target]} ]]; then
      unfunction -- "$target"
    fi
  done
  _warpgatesh_owned=()
  for target in "${targets[@]}"; do
    # Validate before embedding a target in executable shell code.
    [[ $target == [a-zA-Z0-9]* && $target != *[^a-zA-Z0-9._-]* ]] || continue
    # Preserve builtins, aliases, functions, reserved words and executables.
    whence -w -- "$target" >/dev/null 2>&1 && continue
    body="command warpgatesh ${(q)target} -- \"\$@\""
    functions[$target]=$body
    _warpgatesh_owned[$target]=${functions[$target]}
  done
}

autoload -Uz add-zsh-hook
add-zsh-hook -d precmd _warpgatesh_refresh
add-zsh-hook precmd _warpgatesh_refresh
_warpgatesh_refresh
