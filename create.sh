#!/usr/bin/env nix-shell
#!nix-shell -i bash -p bash --pure
#!nix-shell -p aoc-cli
#!nix-shell -p cacert

set -Eeuf -o pipefail
shopt -s inherit_errexit
set -x

main() {
  local day=$1
  local zero_padded_day=$(printf '%02d' "${day}")
  local dir=$(printf 'd%s' "${zero_padded_day}")
  cp -a ./template/. "${dir}"

  sed -i "/^${dir}$/d" ./.gitignore
  sed -i "s/# \(\"${dir}\",\)/\1/" ./Cargo.toml
  sed -i 's/\(println!("day \)\(part \)/\1'"${zero_padded_day}"' \2/' ./"${dir}"/src/main.rs
  sed -i 's/name = "d/&'"${zero_padded_day}"'/' ./"${dir}"/Cargo.toml

  pushd "${dir}"
  aoc download --input-file input.txt -d "${day}"
}

main "$@"
