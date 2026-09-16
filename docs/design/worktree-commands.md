# Design: `rt worktree` (alias `wt`) — the checkout commands

Status: implemented; kept as the original design draft and verification log. The current
description of `rt worktree` is this document; the user-facing guide is
[`../usage.md`](../usage.md).
Scope: `rt worktree` / `rt wt` and its interaction with `rt clone --worktree` and `rt delete`
Revises: `docs/design/clone-worktree.md` §7.2 (how the owning repository is resolved), §7.3
(`rt delete` keeps its meaning), §11 case list

## 1. Motivation

`rt clone --worktree` made "one bare repository per project, one linked worktree per branch"
a supported layout, but the `worktree` command group is still the prototype it was before
that feature landed. Measured against the layout, every command is short of what the layout
needs:

| Command | Problem |
| --- | --- |
| `add <branch>` | Cannot see a branch created upstream after the clone (never fetches); cannot choose a start point; **fails when a local branch of the same name exists** |
| `add <branch>` (ordinary clone) | Derives the sibling path from `current_dir()`, so running it in `sub/dir` targets `sub/dir-<branch>` instead of `<repo>-<branch>` |
| `list` | Raw `git worktree list` passthrough: registry order, no branch column contract, no way to ask for paths only |
| `delete` | Interactive only; menu printed to **stdout** (the scriptable stream); no target argument; no `--force`; blocks or fails obscurely when stdin is not a terminal |
| `prune` | Missing — and it matters here: every checkout registers itself in `A/worktrees/`, so a worktree deleted with `rm -rf` leaves a stale registration behind |
| `path`, `lock`, `unlock`, `move` | Missing |

Two defects were measured on git 2.55.0 before writing this document:

```
$ git -C A worktree add --quiet --track -b dev <abs>/A/dev origin/dev   # local branch `dev` exists
fatal: a branch named 'dev' already exists

$ git -C A worktree add --quiet <abs>/A/dev dev                         # check the local branch out instead
exit 0
```

and the README documents `wa` / `wl` / `wd`, which have never existed (`rt wa` →
`unrecognized subcommand`). The README is corrected by this design, not the CLI.

## 2. Decisions

| Question | Decision |
| --- | --- |
| Short form | `wt` becomes a **visible alias** of `rt worktree` (`rt wt add dev`). No second executable, no packaging change. |
| Which repositories | **Bare layout first** (`rt clone --worktree`), ordinary clones keep the sibling layout. |
| Subcommands in v1 | `add`, `list`, `remove`, `prune`, `path`, `lock`, `unlock`, `move` |
| `add` start point | Fetch `origin` by default (`--no-fetch` opts out), `--from <ref>` picks the base explicitly. |
| `add` with a local branch | Check the local branch out **as it is**; `wt add` never moves a branch ref. |
| `clone` flag | Stays `--worktree`; no `--bare` spelling. |

Everything else in `clone-worktree.md` stays as designed: `A` is the bare repository,
`A/<branch>` is a linked worktree, reserved names are rejected, `clone.depth` is ignored on
that path.

## 3. CLI surface

```
rt worktree <command>        rt wt <command>        # same command, visible alias

  add <branch> [--from <ref>] [--no-fetch]        (alias a)
  list [--paths]                                  (alias l)
  remove [<branch>] [--force]                     (aliases rm, delete, d)
  prune
  path <branch>
  lock <branch> [--reason <text>]
  unlock <branch>
  move <branch> <path>
```

clap shape in `src/main.rs`:

```rust
/// Worktree management commands
#[command(visible_alias = "wt")]
Worktree { #[command(subcommand)] command: WorktreeCommands },
```

Subcommand aliases stay short (`rt wt a dev`, `rt wt l`, `rt wt rm dev`) and `delete` remains
an alias of `remove`, so every spelling that worked before keeps working. The README's
`wa` / `wl` / `wd` lines are deleted: they were never implemented, and adding three more
top-level variants for them would double the CLI surface for four keystrokes.

### 3.1 Output contract

One rule, applied everywhere:

- **stdout** carries the result — the created / removed / printed path, or the `list` table,
  one record per line and deterministic;
- **stderr** carries everything else: menus, prompts, warnings, git's own progress.

`add`, `path` and `move` print an absolute path as the only stdout line, which is the same
contract `rt clone` already has.

## 4. Resolving the project

Every subcommand starts by resolving the repository that owns the current directory. This
**revises `clone-worktree.md` §7.2**, which resolved it as
`mapping::repo_path(root, Remote::parse(remote.origin.url))`:

| | URL + root mapping (§7.2) | Asking git (this design) |
| --- | --- | --- |
| `A/main`, `A/feature/x` find `A` | yes | yes |
| cwd is the bare `A` itself | yes | yes |
| `root` moved or the repository relocated | **no** — falls back to the sibling layout and creates `widget-dev` next to `A` | yes |
| cwd is a second, unrelated clone of the same remote | **yes** — silently operates on the managed `A` instead of the repository you are standing in | no — operates on the repository you are standing in |

The new rule asks git instead of re-deriving the path:

1. `git rev-parse --path-format=absolute --git-common-dir` — the common git directory, which
   is `A` for every worktree of the bare layout and `.git` for an ordinary clone. Outside a
   repository this fails; the command then reports
   `not inside a git repository: run this from a checkout managed by rt` instead of letting
   git's own message through.
2. `git -C <common> rev-parse --is-bare-repository`:
   - `true` → **bare layout**, the project directory is `<common>` (= `A`);
   - `false` → **ordinary clone**, the project directory is the *main* worktree, taken from
     the first record of `git worktree list --porcelain` that git itself reports (not
     `--show-toplevel`, which from a linked worktree of that clone names the worktree, and
     not `current_dir()`, which from a subdirectory names the subdirectory).

`rev-parse --is-bare-repository` is what tells a bare repository from the `.git` directory
of an ordinary clone: the filesystem heuristic in `git::is_bare` (a `HEAD` file plus
`objects/` and `refs/`) is true for both, so it stays where it is used today (`rt tree`
walking, where spawning git per directory would be absurd).

Measured (§10) that `--git-common-dir` resolves correctly from `A/main`, from
`A/dev/nested/deep`, from the bare `A` itself and from `A/refs`, and that
`--path-format=absolute` is available (git ≥ 2.31, 2021; the devshell has 2.55).

### 4.1 New-worktree paths

| Layout | Path for `<branch>` | Naming rule |
| --- | --- | --- |
| bare (`rt clone --worktree`) | `A/<branch>` | the branch name *is* the path, which is the point of the layout |
| ordinary clone | `<parent-of-main>/<main-name>-<branch>` | today's sibling scheme, now anchored to the main worktree instead of the current directory |

Reserved-name rejection (a first path component that collides with a git directory entry,
`clone-worktree.md` §5) applies to the bare layout only; in the sibling scheme the branch
name never shares a namespace with a git directory.

## 5. `wt add <branch> [--from <ref>] [--no-fetch]`

```
1. resolve the project (§4)
2. validate <branch>: git check-ref-format --branch; reserved name when the layout is bare
3. compute the target path (§4.1); refuse when it already exists
4. unless --no-fetch: git -C <dir> fetch origin   (skipped when there is no `origin` remote)
5. pick the start point (table below)
6. git -C <dir> worktree add --quiet [--track] [-b <branch>] <abs path> <start point>
7. zoxide / autojump: register <path>; a failure here is a warning on stderr, never a
   failure of the command — the worktree exists and the user asked for a worktree
8. print <path>
```

### 5.1 Start point

| Condition | Command | Rationale |
| --- | --- | --- |
| `--from <ref>` given, and `refs/heads/<branch>` does not exist | `worktree add -b <branch> <path> <ref>` | the explicit base wins; the ref is validated first (`rev-parse --verify --quiet <ref>^{commit}`) so a typo fails before any directory is created |
| `--from` given and the branch already exists | error: `branch '<branch>' already exists; pick another name or drop --from` | `-b` cannot create it, and silently checking the old branch out would ignore `--from` |
| `refs/heads/<branch>` exists, checked out in another worktree | error naming that worktree's path | git's own message is good (`'dev' is already used by worktree at '…'`); ours adds `rt wt list` / `rt wt path` as the way to find it |
| `refs/heads/<branch>` exists, not checked out anywhere | `worktree add <path> <branch>` | **the decision**: the branch is checked out as it is and its ref is not touched, even when it trails `origin/<branch>`. `wt add` creates a working area; it does not move refs the user owns |
| `refs/remotes/origin/<branch>` exists | `worktree add --track -b <branch> <path> origin/<branch>` | the fresh (post-fetch) remote tip, with a real upstream |
| neither exists | `worktree add -b <branch> <path> <origin/HEAD>` | a new branch off whatever the remote calls its default; `remote.origin.HEAD` comes from `clone-worktree.md` §3 step 8, and when it is missing the error points at `--from` |

The two `-b` rows leave git's `branch.autoSetupMerge` alone, so a new branch that starts from
a remote-tracking ref (`--from origin/dev`, and the `origin/HEAD` fallback) ends up tracking
it. That is git's own default for `git switch -c` and `git worktree add -b` alike; measured
in §10, and pinned by tests, because switching it off with `--no-track` would be a silent
deviation from what the same command does outside `rt`.

The order matters and is a bug fix: the previous code checked
`refs/remotes/origin/<branch>` *before* the local branch, so a repository with both ended in
`-b <branch>` and died with `fatal: a branch named 'dev' already exists` (§1). Local before
remote is also what "check out my branch" means to a user standing in a repository.

### 5.2 Fetch

`git fetch origin` runs before the start point is chosen, so a branch created upstream after
the clone is usable without a manual fetch — the single-object-store layout exists so that
one fetch serves every worktree, and `wt add` is the moment to do it.

- `--no-fetch` skips it (offline, or deliberately pinned to the local refs);
- a repository without an `origin` remote is not fetched and is not an error;
- a fetch that *fails* (network, auth, deleted remote) is fatal and suggests `--no-fetch`:
  the user asked for a synchronized start point, and continuing with stale refs is exactly
  the silent staleness this command exists to remove.

`--from <ref>` is resolved after the fetch, so `--from origin/dev` means the just-fetched tip.

## 6. `wt list [--paths]`

Reads `git worktree list --porcelain` in the project (one git call) and lists **checkouts
only**: the bare repository `A` is not a checkout and is never a row, while the main worktree
of an ordinary clone is one and is.

```
dev      /Users/x/r/github.com/acme/widget/dev
main     /Users/x/r/github.com/acme/widget/main
```

- fields are tab-separated; field 1 is the branch name (`(detached)` when the entry has no
  branch), field 2 the absolute path;
- a locked or prunable entry appends the marker as a further tab-separated field
  (`locked`, `prunable`), so the first two fields stay stable;
- rows are sorted by branch name, with detached entries last, by path — `git worktree list`
  order is not a contract, and `rt` output that scripts read must be deterministic;
- `--paths` prints field 2 alone, one path per line, in the same order. The flag is named
  `--paths` rather than `--porcelain` because it emits paths, not git's porcelain records;
  a single path is `rt wt path <branch>`.

## 7. `wt remove [<branch>] [--force]`

With an argument:

1. resolve `<branch>` against the registry — first by branch name, then by path (so
   `rt wt rm A/dev` works, and so does a path git reports differently from the one we would
   compute, e.g. through a `/var` → `/private/var` symlink);
2. unknown target → `no worktree for branch '<branch>'`;
3. the target is the worktree this command runs in → refuse:
   `refusing to remove the worktree this command runs in; run rt delete from inside it, or
   run rt wt remove from another checkout`. Measured: `git worktree remove <current>` exits 0
   and deletes the directory under the caller's shell, which leaves the shell in a deleted
   cwd. `rt delete` is the command whose documented meaning is "delete what I am standing
   in" (`clone-worktree.md` §7.3); `wt remove` manages the set from outside;
4. `git -C <dir> worktree remove [--force] <path>`; without `--force` git refuses a dirty
   tree (`contains modified or untracked files, use --force to delete it`) and the error is
   passed through, path and all;
5. print `removed worktree <path>`.

Without an argument — interactive:

- the menu goes to **stderr**, numbered, listing branch and path of every removable
  worktree (the current one and the bare repository excluded);
- one removable worktree → still ask, printing the same menu; zero → `no removable
  worktrees found` on stdout and exit 0 (today's behaviour);
- the answer is a number **or** a branch name;
- when stdin is not a terminal (`std::io::IsTerminal`), fail immediately with
  `pass a branch name: rt wt remove <branch>` instead of reading a pipe that will never
  answer.

`--force` is only meaningful together with a dirty tree; it discards uncommitted work, which
is why the flag is spelled out and never implied.

## 8. `prune`, `path`, `lock` / `unlock`, `move`

- **`prune`** runs `git -C <dir> worktree prune -v` with inherited stdio, so git's
  `Removing worktrees/<id>: <reason>` lines reach stderr (measured: `prune -v` writes them
  to stderr, not stdout). Exit status is git's. This is the cleanup for a worktree whose
  directory was deleted by hand; `wt list` marks such entries `prunable` first.
- **`path <branch>`** prints the registered absolute path of that branch's worktree; unknown
  branch → `no worktree for branch '<branch>'`. It is the `cd $(rt wt path dev)` accessor
  and needs no lock or table parsing.
- **`lock <branch> [--reason <text>]`** / **`unlock <branch>`** resolve the target exactly as
  `remove` does and print `locked <path>` / `unlocked <path>`. A locked worktree survives
  `prune` and is listed with the `locked` marker; this is the guard for a long-running
  checkout (a release branch, a bisect) that must not be collected. `--reason` is git's own
  free-text field, stored verbatim.
- **`move <branch> <path>`** delegates to `git worktree move` (the second argument is a
  path, relative to the current directory, resolved absolutely before the call) and prints
  the destination. It does **not** rename the branch, so in the bare layout a moved worktree
  no longer sits at `A/<branch>`; `list`, `path`, `remove`, `lock` and `move` keep working
  because every one of them reads git's registry rather than re-deriving the path, and only
  `add`'s naming rule is affected. Moving the worktree this command runs in is allowed —
  git supports it and it is the only way to relocate a single-checkout clone — with a warning
  on stderr that the current directory moves with it.

## 9. Interaction with the other commands

- **`rt clone --worktree`** is unchanged, including `--worktree[=<branch>]` and the refusal
  of reserved names. It can be seen as `wt add` with a bare repository to create first.
- **`rt delete`** is unchanged: naming a path or URL removes a whole repository (every
  worktree with it), and no argument inside a linked worktree removes that worktree.
  `wt remove` and `rt delete` therefore overlap on purpose, from opposite directions:
  "remove that one, from here" versus "remove this one / all of it".
- **`rt tree`** needs no change: it already descends into a bare repository and lists its
  linked worktrees (`clone-worktree.md` §7.1).
- **Integrations** (`zoxide`, `autojump`): `wt add` registers the created path like
  `rt clone` does, but treats a failure as a warning. A missing `zoxide` binary must not fail
  a command whose real work already succeeded. `rt clone`'s existing hard failure on a
  missing `zoxide` is a separate bug and is out of scope here.

## 10. Verification log

git 2.55.0, macOS, measured while writing this document. Fixture: a bare repository `A`
built exactly as `clone-worktree.md` §3 describes, with worktrees `A/main` and `A/dev`.

| Claim | Command | Result |
| --- | --- | --- |
| §4 `--git-common-dir` identifies the project from a worktree | `git -C A/main rev-parse --path-format=absolute --git-common-dir` | `<abs>/A` |
| §4 …from a nested subdirectory | `git -C A/dev/nested/deep rev-parse --path-format=absolute --git-common-dir` | `<abs>/A` |
| §4 …from the bare repository itself, and from its administrative subdirectory | `git -C A rev-parse …`, `git -C A/refs rev-parse …` | `<abs>/A` both |
| §4 …and `--show-toplevel` is not the answer | `git -C A rev-parse --show-toplevel` | `fatal: this operation must be run in a work tree` |
| §5.1 the local-branch defect | `git -C A worktree add --track -b dev <abs>/A/dev origin/dev`, local `dev` present | `fatal: a branch named 'dev' already exists` |
| §5.1 the fix | `git -C A worktree add <abs>/A/dev dev` | exit 0, `refs/heads/dev` unchanged |
| §5.1 branch checked out elsewhere | same branch into a second directory | `fatal: 'dev' is already used by worktree at '<abs>/A/dev'` |
| §5.1 `--from` typo | `git -C A worktree add -b q <abs>/A/q origin/nope` | `fatal: invalid reference: origin/nope` |
| §5.1 `branch.autoSetupMerge` | `git -C A worktree add -b feature <abs>/A/feature origin/dev`, then `git -C A/feature rev-parse --abbrev-ref feature@{upstream}` | `origin/dev` — a start point that is a remote-tracking ref becomes the upstream, which the tests pin |
| §5.1 target directory exists | `git -C A worktree add <abs>/A/taken main` | `fatal: '<abs>/A/taken' already exists` |
| §6 registry format | `git -C A worktree list --porcelain` | blank-line separated records; `worktree`, `HEAD`, `branch refs/heads/<name>`, and `bare` for `A` itself |
| §6 prunable entries are marked | delete `A/dev` by hand, `git -C A worktree list` | `…/A/dev 86f2ac9 [dev] prunable` |
| §7 removing the current worktree is not refused by git | `cd A/main && git worktree remove <abs>/A/main` | exit 0, directory gone — the caller's cwd is deleted |
| §7 dirty worktree | remove without `--force` | `fatal: '…' contains modified or untracked files, use --force to delete it` |
| §8 prune writes to stderr | `git -C A worktree prune -v` after deleting `A/dev` | stdout empty; stderr `Removing worktrees/dev: gitdir file points to non-existent location` |
| §8 lock is recorded | `git -C A worktree lock --reason "release freeze" <abs>/A/main` | porcelain gains `locked release freeze` |
| §8 move keeps the registry consistent | `git -C A worktree move <abs>/A/dev <abs>/A/hotfix` | exit 0, registry lists `A/hotfix`, `A/dev` gone |
| §4 ordinary clone: main worktree is the registry's first record | `git clone` then `git -C plain worktree list --porcelain` | first record `worktree <abs>/plain`, `branch refs/heads/main`, and `<abs>/plain/.git` as the common dir |
| §5.2 fetch without a remote | `git -C <repo-without-origin> fetch origin` | `fatal: 'origin' does not appear to be a git repository` — hence the existence check before fetching |

## 11. Testing

Unit tests stay next to the code (registry parsing, path rules, menu answers, marker
rendering); everything that needs a real git process drives the built binary per
`tests/common/mod.rs`, split by command: `tests/worktree.rs` (`wt add` / `wt remove`),
`tests/wt.rs` (the read-only commands and repository resolution) and `tests/delete.rs`
(`rt delete`, which keeps its own meaning).

| # | Case | Pins |
| --- | --- | --- |
| 1 | `wt add dev` from `A/main`, and from a nested subdirectory of it | §4 resolution; both create `A/dev`, stdout is the path |
| 2 | Upstream gains a branch *after* the clone; `wt add later` | §5.2 fetch; the worktree tracks `origin/later` |
| 3 | Same branch with `--no-fetch` and no local ref | §5.2; a new branch off `origin/HEAD` instead of the upstream tip |
| 4 | `wt add x --from origin/dev` | §5.1 explicit start point; HEAD equals `origin/dev`, upstream wired by `autoSetupMerge` |
| 5 | local `dev` present, `origin/dev` present, both different | §5.1 local wins, ref not moved, no `already exists` failure |
| 6 | `--from` together with a branch that already exists locally | §5.1 error, nothing created |
| 7 | branch checked out in another worktree | §5.1 error names the existing checkout |
| 8 | `wt add objects` in the bare layout | reserved names still rejected |
| 9 | `wt list`, `wt list --paths` | §6 order, tab fields, bare repository absent |
| 10 | `wt path dev`, `wt path nope` | §8 accessor and its error |
| 11 | `wt rm dev`, then `wt rm <path>`, then `wt rm nope` | §7 non-interactive removal, registration pruned, unknown target |
| 12 | `wt rm` with a dirty worktree, with and without `--force` | §7 refusal and override |
| 13 | `wt rm` of the current worktree; `wt rm` with stdin closed; `wt rm` with nothing removable | §7 refusal, no hang without a terminal, empty menu |
| 14 | delete a worktree directory by hand, then `wt prune` | §8 stale registration removed, `prunable` shown first |
| 15 | `wt lock dev --reason x`, `wt list`, `wt unlock dev` | §8 marker, reason, and round trip |
| 16 | `wt move dev <new path>` | §8 registry follows the move, `wt path dev` still resolves |
| 17 | ordinary clone: `wt add dev` from a subdirectory | §4.1 sibling path anchored to the main worktree |
| 18 | `rt wt …` and `rt worktree …`, `rt wt a` / `rt wt l` / `rt wt rm`, `rt worktree delete` | §3 surface |
| 19 | any subcommand outside a repository | §4 error message |

Implemented in `src/commands/worktree/` — `project.rs` for resolution, one module per
subcommand — with the registry parser in `src/git.rs`. Two things the implementation found,
now part of the design above: the `-b` rows are subject to git's `branch.autoSetupMerge`
(§5.1), so `--from origin/dev` yields a tracking branch, and the interactive menu needed an
explicit terminal check because `Command::output` hands every test a closed stdin (§7).

## 12. Open questions

- whether to add top-level `rt wa` / `rt wl` / `rt wd` shortcuts (the README's old promise);
  deliberately not in v1 — `rt wt a` is short enough and keeps one command group;
- whether `wt add` should also *de-register* a removed path from zoxide/autojump;
- whether `wt move` should offer renaming the branch with the directory
  (`wt rename <old> <new>`) instead of leaving the bare layout's path rule broken;
- whether the sibling scheme for ordinary clones should flatten `/` in branch names
  (`widget-feature/login` today, `widget-feature-login` if flattened).
