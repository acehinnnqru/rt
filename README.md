# rt

`rt` means **repo tree**: a Git repository manager that stores repositories in a predictable directory tree derived from remote URLs.

## What problem it solves

Instead of cloning repositories into random locations, `rt` maps each Git remote to a stable path under one root directory.

Example:

- Remote: `git@github.com:acehinnnqru/rt.git`
- Local path: `{root}/github.com/acehinnnqru/rt`

This structure makes repositories easy to find, script against, and navigate.

## Path mapping rule

Given a Git remote URL, `rt` computes the destination path as:

`{root}/{host}/{owner}/{repo}`

Where:

- `root` is configured in `~/.rt.toml`
- `host` is the Git host (for example `github.com`)
- `owner` is the namespace/user/org
- `repo` is the repository name without `.git`

## Configuration

Configuration file: `~/.rt.toml`

Default config (Unix-like systems):

```toml
root = "{$HOME}/r"

[clone]
depth = 1

[integrations]
zoxide = true
autojump = false
```

### Config keys

- `root` (string): base directory for all managed repositories.
- `clone.depth` (integer): shallow clone depth used during clone operations.
- `integrations.zoxide` (bool): when `true`, add cloned paths to `zoxide`.
- `integrations.autojump` (bool): when `true`, add cloned paths to `autojump`.

## Commands

### Core commands

- `rt clone` (alias: `c`): clone a repository into the computed tree path.
- `rt delete` (alias: `d`): delete a managed repository.
- `rt tree` (alias: `t`): print the repository tree under `root`.
- `rt help`: show command help.

### Worktree commands

`rt worktree` (alias: `wt`) is a command group for `git worktree`.

- `rt worktree add` (aliases: `wt a`, `wa`): create a new worktree.
- `rt worktree list` (aliases: `wt l`, `wl`): list worktrees for the current repository.
- `rt worktree delete` (aliases: `wt d`, `wd`): interactively delete a worktree for the current repository.

## Integrations

- `zoxide`: automatically register cloned paths for faster directory jumping.
- `autojump`: automatically register cloned paths for autojump navigation.

## Dependencies

- `clap`: CLI parsing.
- `tokio`: async runtime.
- `anyhow`: error handling.
- `serde`: serialization/deserialization.
- `toml`: TOML parsing and serialization.

## Contributing

Contributions are welcome. Open an issue or pull request with ideas, bug reports, or improvements.
