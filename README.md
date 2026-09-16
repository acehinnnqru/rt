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
- `rt worktree` (alias: `wt`): manage the worktrees of the current repository.
- `rt help`: show command help.

### One repository, one worktree per branch

`rt clone --worktree` keeps a single object store per project and gives every branch its own
checkout inside it:

```sh
rt clone --worktree git@github.com:owner/repo.git      # -> {root}/github.com/owner/repo/main
rt clone --worktree=dev git@github.com:owner/repo.git  # -> {root}/github.com/owner/repo/dev
```

```
{root}/github.com/owner/repo/            bare repository: HEAD, config, objects/, refs/, worktrees/
{root}/github.com/owner/repo/main/       checkout of the default branch
{root}/github.com/owner/repo/fix-login/  checkout of another branch
```

- the worktree is named after the remote's default branch (`main`, `master`, `trunk`, ...),
  unless a branch is given with `--worktree=<branch>`;
- `clone.depth` does not apply to `--worktree`: shallow history cannot merge branches that
  diverged, which is the point of the layout;
- plain `rt clone` is unchanged and still produces an ordinary clone at the mapped path.

### Worktree commands

`rt worktree` (alias: `wt`) manages the checkouts of the repository you are standing in:
`<repo>/<branch>` in the bare layout above, and a `<repo>-<branch>` sibling when the clone is
ordinary. Every command resolves the repository from the current directory, so it works from
a subdirectory and from any of the checkouts.

| Command | What it does |
| --- | --- |
| `rt wt add <branch>` (alias `a`) | Create the checkout. Fetches `origin` first, so a branch created upstream after the clone is usable: `origin/<branch>` is tracked when it exists, an existing local branch is checked out untouched, and anything else is a new branch off the remote's default. `--from <ref>` starts from an explicit reference, `--no-fetch` skips the fetch. |
| `rt wt list` (alias `l`) | One line per checkout, `<branch>\t<path>`, sorted by branch, with `locked` / `prunable` markers; `--paths` prints the paths only. |
| `rt wt remove [<branch>] [--force]` (aliases `rm`, `delete`) | Remove that checkout; without a branch it asks which one. Without `--force` git refuses a worktree with uncommitted changes. |
| `rt wt prune` | Collect registrations whose directory was deleted by hand. |
| `rt wt path <branch>` | Print the path of a branch's checkout, for `cd $(rt wt path dev)`. |
| `rt wt lock <branch> [--reason <text>]` / `rt wt unlock <branch>` | Protect a checkout from `prune` and `remove`. |
| `rt wt move <branch> <path>` | Move a checkout to another directory; the branch keeps its name. |

`rt delete` is the other direction: naming a path or URL deletes the whole repository with
every worktree in it, while `rt delete` inside a linked worktree (like a plain
`git worktree remove`) deletes that worktree only, refusing to discard uncommitted work.

## Integrations

- `zoxide`: automatically register cloned paths for faster directory jumping.
- `autojump`: automatically register cloned paths for autojump navigation.

## Dependencies

- `clap`: CLI parsing.
- `tokio`: async runtime.
- `anyhow`: error handling.
- `serde`: serialization/deserialization.
- `toml`: TOML parsing and serialization.

## Documentation

- [`docs/usage.md`](docs/usage.md) — the full user guide: configuration, path mapping,
  every command, workflows, the stdout/stderr contract and troubleshooting.
- [`docs/design/clone.md`](docs/design/clone.md) — why `rt clone` maps and builds
  repositories the way it does.
- [`docs/design/worktree-commands.md`](docs/design/worktree-commands.md) — the `rt worktree`
  command group.
- [`docs/README.md`](docs/README.md) — index of the above, and the conventions they use.

## Contributing

Contributions are welcome. Open an issue or pull request with ideas, bug reports, or improvements.
