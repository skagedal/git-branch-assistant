#!/usr/bin/env bash
set -euo pipefail

create-temp-dir() {
    local dir
    dir=$(mktemp -d)
    echo "$dir"
}

create-upstream-ahead() {
    local upstreams
    local repos

    upstreams="$1"
    repos="$2"

    testcase="upstream-ahead"

    git init -q "${upstreams}/${testcase}"

    cd "${upstreams}/${testcase}"
    echo "Initial commit" > file.txt
    git add file.txt
    git commit -q -m "Initial commit"

    cd "${repos}"
    git clone "${upstreams}/${testcase}" "$testcase"
    
    cd "${upstreams}/${testcase}"
    echo "Upstream commit 1" >> file.txt
    git add file.txt
    git commit -q -m "Upstream commit 1"

    cd "${repos}/${testcase}"
    git fetch origin
}

temp_dir=$(create-temp-dir)
upstreams="${temp_dir}/upstreams"
repos="${temp_dir}/repos"
mkdir -p "$upstreams"
mkdir -p "$repos"

create-upstream-ahead "$upstreams" "$repos"
echo "Created test cases"
echo "$repos"
