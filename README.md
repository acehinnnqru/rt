# rt

A Git Repositories Manager manages all repos as a tree. So it names **`r`**`epo` **`t`**`ree` = **`rt`**.

Considering a git url like `git@github.com:acehinnnqru/rt.git`, it would be clone into dir `{config root}/github.com/acehinnnqru/rt`.

The params in the directory name:

- `config root` is the root set in config file `{$HOME}/.rt.toml`.
- `depth` is the depth of the cloned repos.

## Integrations

- `zoxide`: automatically add the cloned path to the `zoxide` database.
- `autojump`: automatically add the cloned path to the `autojump` database.

## Default Config

### For unix-like os

```toml
root = "{$HOME}/r"

[clone]
depth = 1

[integrations]
zoxide = true
```

## Commands

- `rt clone`: alias `c` to clone repos.
- `rt delete`: alias `d` to delete repos. 
- `rt tree`: alias `t` to show repos tree under `{config root}`.
- `rt help`: show help.

### For `git worktree` support

`rt worktree`: alias `wt` is the subgroup command for `git worktree`.

- `rt worktree add`: alias `wt a` or `wa` to add a new worktree.
- `rt worktree list`: alias `wt l` or `wl` to list all worktrees under the current repo.
- `rt worktree delete`: alias `wt d` or `wd` to delete a worktree interactively under the current repo.

## Dependencies

- `clap`: for command line parsing.
- `tokio`: for asynchronous runtime.
- `anyhow`: for error handling.
- `serde`: for serialization and deserialization.
- `serde_toml`: for TOML serialization and deserialization.

## Contributing

Welcome to contribute if you have any ideas, issues, and stuff.
