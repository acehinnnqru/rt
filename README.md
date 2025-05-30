# rt

A Git Repositories Manager manages all repos as a tree. So it names `repo tree` = `rt`.

Considering a git url like `git@github.com:acehinnnqru/rt.git`, it would be clone into dir `{config root}/github.com/acehinnnqru/rt`.

The params in the directory name:

- `config root` is the root set in config file `{$HOME}/.agrm.toml`.

## Integrations

- `zoxide`: automatically add the cloned path to the `zoxide` database.
- `autojump`: automatically add the cloned path to the `autojump` database.

## Default Config

### For unix-like os

```toml
root = "{$HOME}/rt"

[clone]
depth = 1
bare = false

[integrations]
zoxide = true
```

## Commands

- `rt c`: clone repos.
- `rt cb`: clone bare repos.
- `rt d`: delete repos. 
- `rt t`: show repos tree under `{config root}`.

## Contributing

Welcome to contribute if you have any ideas, issues, and stuff.
