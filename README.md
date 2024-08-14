# agrm

A Git Repositories Manager

`agrm` will clone a bare repository into a directory which named as `{root}/{git platform}/{namespace}/{name}/.bare`.

The params in the directory name:
- `root` is the root from config file `{$HOME}/.agrm.toml`
- `git platform` means the target platform in the provided repository url/ssh.
- `namespace` and `name` are also extract from the provided repository url/ssh.

## Integrations

- `zoxide`: `agrm` will automatically add the cloned path to the `zoxide` database.

## Default Config

### For unix-like os

```toml
root = "{$HOME}/agrm"

[integrations]
zoxide = true
```

### For windows

```toml
root = "{$USERPROFILE}/agrm"

[integrations]
zoxide = true
```

## Contributing

Welcome to contribute if you have any ideas, issues, and stuff.

