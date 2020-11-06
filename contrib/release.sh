#!/usr/bin/env bash
set -euoE pipefail

function tag() {
    tag="${1}"

    if git tag | grep "$tag"
    then
        return
    fi

    git tag "$tag"
    git push --tags
}

function create_release() {
    version="${1}"

    if gh release list | grep "$version"
    then
        return
    fi
}

function release() {
    version="${1}"

    create_release "$version"
    cargo build --release
    gh release upload "$version" target/release/bpd
}

function main() {
    pushd $(git rev-parse --show-toplevel)
    current_version=$(grep -E '^version = ' Cargo.toml | awk -F ' = ' '{print $2}' | tr -d '"')
    tag "$current_version"
    release "$current_version"
    popd
}

main $@
