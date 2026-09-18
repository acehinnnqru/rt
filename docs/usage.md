# rt — usage

`rt` means **repo tree**: a Git repository manager that stores every repository at a path
derived from its remote URL, under a single configurable root.

```sh
rt clone git@github.com:acehinnnqru/rt.git
# -> {root}/github.com/acehinnnqru/rt
```

The point of the tool is that the path is predictable: it can be computed from the remote,
scripted against, and reached from any shell — no "where did I clone that?" step.

- [1. Concepts](#1-concepts)
- [2. Requirements and installation](#2-requirements-and-installation)
- [3. Configuration](#3-configuration)
- [4. The path mapping rule](#4-the-path-mapping-rule)
- [5. Command reference](#5-command-reference)
- [6. Workflows](#6-workflows)
- [7. Output contract](#7-output-contract)
- [8. Behaviour worth knowing](#8-behaviour-worth-knowing)
- [9. Troubleshooting](#9-troubleshooting)

## 1. Concepts

| Term | Meaning |
| --- | --- |
| **root** | The base directory every managed repository lives under (`~/.rt.toml`). |
| **mapped path** | `{root}/{host}/{owner}/{repo}` — where a remote URL lands. |
| **ordinary clone** | `rt clone <url>`: repository and checkout both at the mapped path. |
| **bare layout** | `rt clone <url> --worktree`: a bare repository at the mapped path with one linked worktree per branch inside it. |
| **project** | The repository a `rt wt …` command acts on, resolved by asking git for the common git directory of the current directory. |
| **checkout** | A working tree git registers — the bare repository itself is not one. |

A bare layout for `git@github.com:owner/repo.git` looks like this:

```
{root}/github.com/owner/repo/            bare repository: HEAD, config, objects/, refs/, worktrees/
{root}/github.com/owner/repo/main/       checkout of the default branch; .git is a file
{root}/github.com/owner/repo/fix-login/  checkout of another branch
```

One object store, N checkouts: `git fetch` once and every worktree sees it, and there is no
`git switch` dance when moving between branches. See
[design/clone.md](design/clone.md) for why the layout is built this way.

## 2. Requirements and installation

- **git** on `PATH` (2.31 or newer for the bare layout; the `--path-format=absolute` and
  `--git-common-dir` options `rt` relies on). Developed against git 2.55.
- **A Rust toolchain**, or the Nix devshell from `flake.nix` — the devshell is the single
  source of truth for the toolchain in this repository.

Build and install from a checkout:

```sh
direnv exec . cargo build --release     # or: nix develop, then cargo build --release
install -m755 target/release/rt ~/.local/bin/rt
```

Add the binary to `PATH` and, if you want directory jumping, install
[`zoxide`](https://github.com/ajeetdsouza/zoxide) or `autojump` and enable the matching key
in the config (§3).

## 3. Configuration

Runtime configuration lives in `~/.rt.toml`. A missing file is not an error: `rt` falls back
to the defaults below. `.rt.toml.example` in the repository documents the same keys.

```toml
root = "{$HOME}/r"

[clone]
depth = 1

[integrations]
zoxide = true
autojump = false
```

| Key | Type | Default | Effect |
| --- | --- | --- | --- |
| `root` | string | `$HOME/r` | Base directory for every managed repository. `~`, `$HOME` and `{$HOME}` are expanded. |
| `clone.depth` | integer | `1` | `--depth` for an ordinary clone. `0` means full history. Ignored by `--worktree`. |
| `integrations.zoxide` | bool | `false` | Register the created checkout with `zoxide add` after a clone or `wt add`. |
| `integrations.autojump` | bool | `false` | Same, through `autojump`. |

Notes:

- `rt` never writes `~/.rt.toml`; the file is read at every invocation.
- `root` must exist before `rt tree`; `rt clone` creates intermediate directories itself.
- An integration that is enabled but whose binary is missing makes `rt clone` fail (`zoxide
  command not found in PATH`) *after* the clone succeeded. `rt wt add` treats the same
  failure as a warning on stderr instead.

## 4. The path mapping rule

Given a remote URL, `rt` computes `{root}/{host}/{namespace…}/{repo}`:

| Remote | Mapped path |
| --- | --- |
| `git@github.com:acehinnnqru/rt.git` | `{root}/github.com/acehinnnqru/rt` |
| `https://github.com/acehinnnqru/rt.git` | `{root}/github.com/acehinnnqru/rt` |
| `github.com/acehinnnqru/rt` (short form) | `{root}/github.com/acehinnnqru/rt` |
| `ssh://git@gitlab.com:909/groups/sub/name.git` | `{root}/gitlab.com/groups/sub/name` |
| `https://go.googlesource.com/arch` | `{root}/go.googlesource.com/arch` |

Rules that follow from the implementation:

- the trailing `.git` is stripped, and a nested namespace (GitLab subgroups) becomes nested
  directories — the tree follows the namespace, not a fixed depth;
- a port is parsed but **dropped** from the layout path, so `gitlab.com:909` and
  `gitlab.com` map to the same directory;
- `ssh://`, `git@host:` , `http(s)://` and the short `host/owner/repo` form are all accepted;
- the host is used verbatim otherwise, so `https://github.com/owner/repo` and
  `git@github.com:owner/repo.git` are the *same* repository to `rt`, whichever way you
  cloned it.

## 5. Command reference

```
rt <command>

  clone <url> [-w|--worktree[=<branch>]]      (alias: c)
  delete [<path|url>]                         (alias: d)
  tree                                        (alias: t)
  worktree <command>                          (alias: wt)
```

### `rt clone <url>`

| Invocation | Result |
| --- | --- |
| `rt clone <url>` | Ordinary clone at the mapped path, `--depth {clone.depth}` applied. |
| `rt clone <url> --worktree` | Bare repository at the mapped path, default branch checked out at `<repo>/<default branch>`. |
| `rt clone <url> --worktree=<branch>` | Same, with `<branch>` as the checkout — it must already exist on `origin`. |

`--worktree` takes its value with `=`: `rt clone --worktree dev <url>` is a parse error, so a
positional URL is never mistaken for the branch. The default branch is whatever
`refs/remotes/origin/HEAD` says (`main`, `master`, `trunk`, …) — never hard-coded.

### `rt worktree` (alias `wt`)

Every subcommand resolves the project from the current directory — from a subdirectory, from
any linked worktree, or from the bare repository itself.

| Command | What it does |
| --- | --- |
| `add <branch> [--from <ref>] [--no-fetch]` (alias `a`) | Create the checkout and print its path. Fetches `origin` first, so a branch created upstream after the clone is usable. `--from <ref>` starts from an explicit reference, `--no-fetch` skips the fetch. |
| `list [--paths]` (alias `l`) | One line per checkout: `<branch>\t<path>`, sorted by branch, `locked` / `prunable` appended as extra fields. `--paths` prints paths only. |
| `remove [<branch>] [--force]` (aliases `rm`, `delete`, `d`) | Remove that checkout. Without a branch it asks which one; `--force` discards uncommitted work. |
| `prune` | Collect registrations whose directory was deleted by hand. |
| `path <branch>` | Print a checkout's path, for `cd $(rt wt path dev)`. |
| `lock <branch> [--reason <text>]` / `unlock <branch>` | Protect a checkout from `prune` and `remove`. |
| `move <branch> <path>` | Move a checkout; the branch keeps its name. |

### `rt delete [<path|url>] [--force]`

- `rt delete <url>` — parse the URL, map it, remove the whole repository. In the bare layout
  that takes every checkout inside it along; a checkout that was moved elsewhere with
  `rt wt move` survives on disk.
- `rt delete <path>` — an existing path is used as given, so `rt delete .` or
  `rt delete /abs/path` work and are never mistaken for a remote URL. An argument that does
  not exist is parsed as a remote URL when it looks like one (`scheme://`, an `@`, or two or
  more slashes); otherwise it is treated as a path and `path does not exist: …` is the error.
- `rt delete` — the current directory. Inside a linked worktree this removes **that worktree
  only**, through `git worktree remove`, so the registration is pruned with it. From the bare
  repository or an ordinary clone it removes the whole repository.
- Every deletion is confirmed first, on the terminal: the absolute path, and then, when a
  checkout that is about to go holds work that is not in git yet, that checkout as well — so
  agreeing to the path is not also agreeing to lose uncommitted work. Only an explicit `y` or
  `yes` goes ahead; a newline or the end of the input declines.
- `--force` (`-f`) skips the questions and takes uncommitted work with it. Without a terminal
  there is nobody to ask, so `--force` is required there: a script, pipe or CI run gets
  `refusing to delete without confirmation: pass --force` instead of a hang.

### `rt tree`

Prints `root` and everything below it as an ASCII tree. Ordinary clones and linked worktrees
are leaves (the walk never descends into `src/`, `target/`, …); a bare repository is a
container, so its checkouts are listed and git's own entries (`objects`, `refs`, `worktrees`,
…) are skipped. Hidden directories are skipped everywhere.

```console
$ rt tree
/Users/me/r
└── github.com
    └── owner
        └── repo
            ├── fix-login
            └── main
```

## 6. Workflows

### Clone and work in one repository

```sh
rt clone git@github.com:owner/repo.git
# /Users/me/r/github.com/owner/repo
cd /Users/me/r/github.com/owner/repo
```

Because the path is printable and predictable, the clone and the `cd` are one step:

```sh
cd "$(rt clone git@github.com:owner/repo.git)"
```

### Work on several branches at once

```sh
rt clone --worktree git@github.com:owner/repo.git    # -> {root}/github.com/owner/repo/main
cd {root}/github.com/owner/repo/main
rt wt add fix-login                                  # -> {root}/github.com/owner/repo/fix-login
rt wt add release --from origin/release-2.0
rt wt list
# fix-login   /Users/me/r/github.com/owner/repo/fix-login
# main        /Users/me/r/github.com/owner/repo/main
# release     /Users/me/r/github.com/owner/repo/release
```

Each checkout shares one object store: `git fetch` in any of them serves all of them, and
`git pull` works in each because its branch tracks `origin/<branch>`. Merging or rebasing
between two checkouts needs the branches' common history, which is why `--worktree` fetches
full history regardless of `clone.depth` (§8).

### Jump between checkouts

```sh
cd "$(rt wt path fix-login)"     # scriptable, no table parsing
z "$(rt wt path fix-login)"      # with zoxide
```

With `integrations.zoxide = true`, every path `rt clone` or `rt wt add` creates is registered
with `zoxide add`, so `z fix-login` works even before you have visited it.

### Work in an ordinary clone

`rt wt add` is not limited to the bare layout. In an ordinary clone the new checkout is a
sibling of the main worktree, named `<repo>-<branch>`:

```sh
rt clone git@github.com:owner/repo.git
cd {root}/github.com/owner/repo
rt wt add dev
# -> {root}/github.com/owner/repo-dev
```

`clone.depth` still applies here, so the sibling starts from whatever history the clone has.

### Clean up

```sh
rt wt rm fix-login                # one checkout, refuses a dirty tree
rt wt rm fix-login --force        # ... for real
rm -rf {root}/github.com/owner/repo/scratch
rt wt prune                       # collect the stale registration
rt delete .                       # from inside a checkout: that checkout only, after asking
rt delete git@github.com:owner/repo.git   # the whole project, worktrees included
rt delete --force .               # no questions: for scripts, and for uncommitted work
```

## 7. Output contract

The rule is uniform and scriptable:

- **stdout** carries the result: the created / removed / printed path, the `list` table, the
  `tree` rendering — one record per line, sorted where order is not fixed by git.
- **stderr** carries everything else: warnings, menus, prompts, git's own progress output.

Commands that print a path print an **absolute** path and nothing else on stdout:

| Command | stdout |
| --- | --- |
| `rt clone …` / `rt wt add …` / `rt wt path <branch>` / `rt wt move …` | the absolute path |
| `rt wt list` | `<branch>\t<path>[\tlocked][\tprunable]` per line, sorted by branch |
| `rt wt list --paths` | one absolute path per line |
| `rt wt rm …` | `removed worktree <path>` |
| `rt delete …` | `deleted <path>` or `deleted worktree <path>` |
| `rt tree` | the root, then the tree |

Git's chatty commands are run with `--quiet` precisely so that `rt clone … | xargs cd`-style
scripts keep working, and `rt wt list` order is defined by `rt`, not by
`git worktree list`.

## 8. Behaviour worth knowing

**`clone.depth` does not apply to `--worktree`.** With the default `depth = 1`, `rt clone
--worktree` prints one warning and fetches full history. Shallow history has no common
ancestor, so merging or rebasing two branches checked out side by side fails with
`refusing to merge unrelated histories` — the very operation the bare layout exists for.
Deepening later is possible by hand: `git -C <repo> fetch --unshallow origin`.

**`--worktree` is opt-in and one-way.** Nothing converts an ordinary clone into the bare
layout, and existing clones keep the sibling scheme for `rt wt add`. The flag is per
invocation; there is no config key that turns it into the default.

**Reserved branch names.** In the bare layout the checkout path is a child of the git
directory, so a branch whose first path component is one of git's own entries (`objects`,
`refs`, `worktrees`, `hooks`, `HEAD`, …) is rejected before anything is created. `worktrees`
is not a theoretical case: git accepts the checkout and then registers it *inside the
checkout it administers*. Nested names are fine — `feature/login` maps to
`<repo>/feature/login`, and only the first component is checked.

**`rt wt add` never moves a branch ref.** If `refs/heads/<branch>` exists locally, it is
checked out exactly as it is, even when it trails `origin/<branch>`. If it does not exist
but `origin/<branch>` does, the new branch is created at that (freshly fetched) tip and
tracks it. If neither exists, the branch starts at `origin/HEAD`.

**`--from` needs a fresh name.** `rt wt add <branch> --from <ref>` errors when `<branch>`
already exists locally: `-b` cannot create it, and quietly checking the old branch out would
ignore `--from`.

**A failing fetch is fatal.** `rt wt add` fetches by default and stops on failure, because
the point of the command is a synchronized start point; pass `--no-fetch` deliberately when
offline. A repository without an `origin` remote is simply not fetched — that is not an
error.

**`rt wt rm` refuses the checkout it runs in.** `git worktree remove <cwd>` would happily
delete the directory the calling shell is standing in. Removing *that* checkout is `rt
delete`'s job; `rt wt rm` manages the set from outside. It also refuses a dirty tree unless
`--force` is given.

**Moved worktrees keep their branch name.** `rt wt move dev <path>` relocates the checkout,
so in the bare layout it no longer sits at `<repo>/dev`. Only `add`'s naming rule is
affected: `list`, `path`, `remove`, `lock` and `move` all read git's registry, so they keep
resolving the branch wherever it ended up.

**`rt delete` takes paths as given.** An argument that exists on disk is never parsed as a
URL, which is what makes `rt delete .` safe. A URL argument is mapped through the same rule
as `clone` — so it only works when the remote maps to the same path it was cloned from.

**`rt delete` asks what it is about to do.** The question names the absolute path, not the
argument as typed, so a relative path the shell expanded differently than expected is still
visible before it goes. A checkout inside the doomed directory with uncommitted work is asked
about separately, because losing that work is not what agreeing to a path means. `--force`
answers both questions, and is the only way to answer without a terminal.

## 9. Troubleshooting

| Symptom | Cause and fix |
| --- | --- |
| `repository already exists at <path>` | `rt clone` never overwrites. Remove it (`rt delete <path>`) or clone the other remote spelling (`git@` vs `https://`) — both map to the same path. |
| `could not determine the default branch of <url>` | The remote is empty or unreachable. Pass `--worktree=<branch>` explicitly, or check credentials. |
| `branch 'x' does not exist on <url>` | `--worktree=<branch>` requires the branch to exist on `origin`; creating branches is `rt wt add`'s job. |
| `branch 'x' cannot be a worktree inside the repository` | The name collides with a git directory entry (§8). Pick another name. |
| `not inside a git repository: run this from a checkout managed by rt` | `rt wt …` was run outside a repository. `cd` into a checkout first. |
| `root directory does not exist: <path>` | `rt tree` requires `root` to exist; `mkdir -p` it, or fix `root` in `~/.rt.toml`. |
| `pass a branch name: rt wt remove <branch>` | `rt wt rm` without a branch needs a terminal for its menu; a script or pipe gets this error instead of a hang. |
| `refusing to delete without confirmation: pass --force` | `rt delete` asks on the terminal, and a script, pipe or CI run has none. `--force` is the answer — note that it also discards uncommitted work. |
| `warning: ignoring clone.depth = N for --worktree` | Expected on the bare layout (§8). |
| `warning: zoxide: zoxide command not found in PATH` | `integrations.zoxide` is on but `zoxide` is not installed. The checkout exists; install `zoxide` or turn the key off. |
| Two worktrees cannot be merged | The repository was made shallow by hand (`git -C <repo> fetch --unshallow origin`). `rt` itself never makes a `--worktree` clone shallow. |

## See also

- [`docs/design/clone.md`](design/clone.md) — why `rt clone` does what it does.
- [`docs/design/worktree-commands.md`](design/worktree-commands.md) — the `rt wt` command
  group, its output contract and its measured git behaviour.
- `README.md` at the repository root — the short version and project layout.
