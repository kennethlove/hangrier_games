#!/bin/zsh
set -e
cd /Users/klove/ghq/github.com/kennethlove/hangrier_games
just fmt
git add -A
git commit --amend --no-edit
git push --force-with-lease
