# git-branch-assistant

**git-branch-assistant** is a command line application to manage your git repositories and synchronize branches with their upstreams. There are many other tools with similar functionality[^1]; this one is built to support the workflow I personally prefer. I use it together with my [assistant](https://github.com/skagedal/assistant) tool, but it can be used as a standalone program.

## Git cleanup

The `git-branch-assistant git-clean` command cleans up branches in the git repository of the current working directory.

For each local branch, it compares to upstream and gives you a selection of options depending on current state.

[^1]: See for example [myrepos](https://myrepos.branchable.com/) and its list of [related tools](https://myrepos.branchable.com/related/)
