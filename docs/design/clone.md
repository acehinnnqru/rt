# Design: `rt clone` — where a repository lands and how it is built

Status: implementation record for the code in `src/commands/clone.rs` (git 2.55, 2026-09).
Companion: [`usage.md`](../usage.md) is the user-facing guide; this document is the
rationale. Measurements behind every load-bearing claim are logged in
[`clone-worktree.md`](clone-worktree.md) §13 and [`worktree-commands.md`](worktree-commands.md) §10.

Scope: `rt clone`, the path mapping it uses, both layouts it can produce, and the failure
handling around them.

- [1. What `rt clone` is for](#1-what-rt-clone-is-for)
- [2. Two layouts, one flag](#2-two-layouts-one-flag)
- [3. Path mapping](#3-path-mapping)
- [4. The ordinary clone](#4-the-ordinary-clone)
- [5. The bare layout](#5-the-bare-layout)
- [6. Validation and failure handling](#6-validation-and-failure-handling)
- [7. Configuration surface](#7-configuration-surface)
- [8. Integrations](#8-integrations)
- [9. Compatibility](#9-compatibility)
- [10. Alternatives considered](#10-alternatives-considered)
- [11. Tests that pin this](#11-tests-that-pin-this)
- [12. Open questions](#12-open-questions)

## 1. What `rt clone` is for

The tool exists to make a repository's local path a function of its remote URL, so that it is
derivable rather than remembered. `rt clone` is the command that establishes that invariant:
everything else (`tree`, `delete`, `worktree`) assumes repositories live where the mapping
rule says they do.

Two requirements follow, and they shape every decision below:

1. **`rt clone` must print the checkout path on stdout, and nothing else.** It is the
   `cd "$(rt clone …)"` command; any extra stdout line is a bug.
2. **A failed clone must leave nothing behind.** The mapped path is deterministic, so a
   half-built repository at that path blocks every later attempt with
   `repository already exists at …`.

A non-requirement worth stating: `rt clone` is not a git front-end. It never re-implements
fetching, merging or checkout semantics; it sequences git commands, validates before running
them, and cleans up after itself.

## 2. Two layouts, one flag

| | Ordinary clone (default) | Bare layout (`--worktree`) |
| --- | --- | --- |
| Repository | `<dest>/.git` | `<dest>` itself, bare |
| Checkout | `<dest>` | `<dest>/<branch>`, a linked worktree |
| Branch per checkout | one, switched with `git switch` | one directory per branch |
| Object store | one | one, shared by every checkout |
| History | `clone.depth` | always full (§5.3) |

```sh
rt clone <url>                      # ordinary clone at A
rt clone <url> --worktree           # default branch -> A/<default branch>
rt clone <url> --worktree=<branch>  # <branch> must already exist on origin -> A/<branch>
```

clap shape (`src/main.rs`):

```rust
Clone {
    /// Git remote URL
    url: String,
    /// Clone as a bare repository with the default branch linked as a worktree
    /// inside it (for example `A/main`); accepts an optional branch name
    #[arg(long, short = 'w', num_args = 0..=1, require_equals = true, default_missing_value = "")]
    worktree: Option<String>,
}
```

- absent → `None` → ordinary clone, the pre-existing behaviour;
- `--worktree` → `Some("")` → the branch is the remote's default branch;
- `--worktree=dev` → `Some("dev")` → branch `dev`;
- `require_equals = true` keeps the positional URL unambiguous: `--worktree dev <url>` is a
  parse error instead of a silent argument grab.

The flag is opt-in per invocation. There is no `[clone] worktree = true` key, so no
counter-flag is needed and existing users see no change.

## 3. Path mapping

`src/path/remote.rs` parses the URL; `src/path/mapping.rs` turns it into a path. The mapping
is `{root}/{host}/{namespace…}/{repo}`: nested namespaces become nested directories, `.git`
is stripped, and a port is parsed and then dropped from the host component, so
`gitlab.com:909` and `gitlab.com` are the same directory.

Accepted spellings — all four mapping to the same path for the same host and repository:

| Form | Example |
| --- | --- |
| scp-like SSH | `git@github.com:owner/repo.git` |
| `ssh://` | `ssh://git@gitlab.com:909/groups/sub/name.git` |
| `http(s)://` | `https://github.com/owner/repo` |
| short | `github.com/owner/repo` |

Design consequences that are deliberate, not incidental:

- **Host spelling is not normalised beyond the port.** `git@github.com:owner/repo.git` and
  `https://github.com/owner/repo` are one path. That is what makes `rt clone` correctly
  refuse a repository that is already there, whichever way it was cloned — and it is also
  why a URL argument to `rt delete` resolves against the same rule.
- **Namespace depth is unbounded**, because GitLab subgroups are real; `rt tree` walks to
  whatever depth it finds rather than to a fixed one.
- **No user segment in the path.** `ssh://git@host/...` keeps the host only; the git user is
  not part of a repository's identity.

## 4. The ordinary clone

`clone_plain` is intentionally thin:

```rust
cmd.arg("clone");
if config.clone.depth > 0 {
    cmd.arg("--depth").arg(config.clone.depth.to_string());
}
cmd.arg(url).arg(dest);
```

Everything is delegated to `git clone` — credential helpers, progress, submodule and
`init.defaultBranch` behaviour included. The only `rt` decision here is `clone.depth`
(default `1`), a speed/storage tradeoff that suits the one-checkout-per-repository case.

The consequence is that an ordinary clone's history may be shallow, which is fine for
`git switch`-style work and fatal for the bare layout (§5.3).

## 5. The bare layout

### 5.1 Algorithm

Every git command inherits stdio, as `git clone` does, so credentials and progress keep
working. Everything that can be validated is validated before the first git command runs,
which is what makes rollback (§6) trivial.

```
0. remote = Remote::parse(url); dest = mapping::repo_path(root, remote)
1. bail if dest exists
2. create dest's parent directories
3. default = git ls-remote --symref <url> HEAD     -> refs/heads/<name>
4. branch = requested (--worktree=<branch>) or default
   - reject when git check-ref-format --branch fails
   - reject when the branch is not the default and origin has no refs/heads/<branch>
   - reject when it is a reserved worktree name
5. git init --bare --quiet <dest>
6. git -C <dest> remote add origin <url>
7. git -C <dest> config remote.origin.fetch '+refs/heads/*:refs/remotes/origin/*'
8. git -C <dest> fetch origin                     (full history, always)
9. git -C <dest> symbolic-ref HEAD refs/heads/<default>
   git -C <dest> symbolic-ref refs/remotes/origin/HEAD refs/remotes/origin/<default>
10. git -C <dest> worktree add --quiet --track -b <branch> <abs dest>/<branch> origin/<branch>
11. print <dest>/<branch> on stdout
```

The worktree path is passed **absolute**: git resolves it relative to the `-C` directory, so
a relative `A/dev` silently lands in `<dest>/A/dev` instead of failing (measured). `git init`
and `git worktree add` also run with `--quiet`, because both write progress to *stdout*,
which would pollute the one scriptable line `rt` prints (measured).

### 5.2 Why `init --bare` + `fetch` instead of `clone --bare`

`git clone --bare` looks like the obvious primitive and is the wrong one. It copies the
remote's branch heads directly into `refs/heads/*`, creating neither remote-tracking refs nor
the configuration for them. Three consequences, all measured:

1. **No fetch refspec, so the mirrored heads freeze.** After `git clone --bare` the only
   `remote.*` key is `remote.origin.url`. Adding the natural-looking bare refspec
   `+refs/heads/*:refs/heads/*` is worse: fetching from a checkout that has the branch
   checked out dies with `fatal: refusing to fetch into branch 'refs/heads/main' checked out
   at '<dest>/main'`. The chosen refspec writes only `refs/remotes/origin/*`, never a
   checked-out branch, so `git pull` works in every worktree.
2. **Mirrored local branches block `-b`.** Step 10 creates the branch; a mirrored `refs/heads/*`
   would make it fail with `a branch named 'x' already exists`. `init --bare` plus `fetch`
   never creates local branches.
3. **`--depth` drags `--single-branch`.** git-clone(1): `--depth` implies `--single-branch`
   unless `--no-single-branch` is given. Since `clone.depth` defaults to `1`, a naive
   `clone --bare --depth 1` yields a repository with exactly one branch, so every later
   `rt wt add` would fail.

`git init --bare` additionally writes a hard-coded `HEAD -> refs/heads/main|master`, which
step 9 rewrites to the branch the remote actually uses — so `git -C A symbolic-ref --short
HEAD` and commit-ish-less `git worktree add` behave correctly.

Cost of the choice: `rt` owns the sequence instead of borrowing `git clone`'s wiring. That is
exactly what the integration tests pin down.

### 5.3 Full history is not negotiable

`clone.depth` defaults to `1`, and `--worktree` deliberately ignores it. This is a measured
incompatibility, not caution:

| Operation on a depth-1 bare repository with two checkouts | Result |
| --- | --- |
| `git merge origin/dev` | `fatal: refusing to merge unrelated histories` |
| `git merge-base origin/main origin/dev` | no output, exit 1 |
| `git pull --ff-only` | works — a fast-forward needs no merge base |
| plain `git fetch origin` after new upstream commits | new tip fetched, repository stays shallow |
| `git fetch --unshallow origin`, then the same merge | works |

A shallow repository never deepens on its own, so a shallow bare layout looks healthy —
clones fast, fetches and fast-forwards normally — and then fails the first time two
checkouts are merged or rebased, which is the entire point of the layout. Deferring that
failure to merge time is worse than paying for history once at clone time.

So step 8 passes no `--depth`, and when `clone.depth > 0` the command prints exactly one
warning naming the reason. Shallow worktree layouts stay reachable by hand
(`git -C A fetch --depth=<n>`); an explicit opt-in is an open question (§12).

Rejected alternatives: erroring out when `clone.depth > 0` (would make `--worktree` unusable
with the default config), honouring the depth and warning (that is the silent-failure design
above, shipped on purpose), and a "large enough" `--depth=N` (any depth truncates: a branch
whose merge base is older than `N` breaks identically).

### 5.4 Reserved worktree names

The checkout path is a child of the git directory, so the first path component of a branch
name shares a namespace with git's own administrative entries
(`HEAD`, `config`, `description`, `index`, `packed-refs`, `shallow`, `commondir`, `gitdir`,
and the directories `objects/`, `refs/`, `branches/`, `hooks/`, `info/`, `logs/`,
`remotes/`, `modules/`, `worktrees/`).

Git does not catch these, and one of them is not caught at all (measured):

- `objects`: the add fails with `fatal: '<dest>/objects' already exists`, but only after
  creating a stray local branch `objects` — the check works by accident, because the
  directory happens to be non-empty;
- `worktrees`: the add **succeeds**. The checkout directory is named `worktrees`, so git
  registers it at `$GIT_DIR/worktrees/worktrees/`: the administration directory ends up
  inside the checkout it administers, and `git worktree prune` then reports
  `Removing worktrees/.git: not a valid directory`.

The guard is therefore a correctness requirement, not a nicer error message. `rt` rejects
such names before creating anything, with an error naming the offending component. Only the
first component is compared, so `feature/login` is fine and maps to `A/feature/login` —
flattening it to `A/feature-login` was rejected because it would make the checkout path
underivable from the branch name.

The same constant (`git::ADMIN_ENTRIES`) is reused by `rt tree`, which skips these entries
when listing a bare repository.

### 5.5 Layout facts `rt` relies on

- `A` is bare: `git -C A rev-parse --is-bare-repository` prints `true`, and `A/.git` does not
  exist. Filesystem detection: `A/HEAD` is a file while `A/objects` and `A/refs` are
  directories — true for the `.git` directory of an ordinary clone as well, which is why
  `rt wt …` asks git (`--is-bare-repository`) and only `rt tree`, which cannot spawn a git
  process per directory, uses the heuristic.
- `A/<branch>` is a linked worktree: `.git` is a *file* containing
  `gitdir: <A>/worktrees/<id>`.
- The worktree registry is `A/worktrees/<id>/`, with each entry's `gitdir` file holding the
  absolute path of its checkout.
- `worktree.useRelativePaths` stays at its default (absolute `gitdir:`), so moving `root`
  breaks the links. Enabling it needs the `extensions.relativeWorktrees` extension and a
  recent git, which older git refuses to read — open in §12.

## 6. Validation and failure handling

Validation before side effects means that when a git command does fail, everything on disk
was created by this run and may be removed.

| Failure | State on disk | Behaviour |
| --- | --- | --- |
| bad URL / invalid branch name / unknown branch / reserved name | nothing | error before the first git command |
| `init --bare`, `remote add`, `config` | empty `dest` | remove `dest`, propagate the error |
| `fetch` (network, auth, empty remote) | bare `dest`, no refs | remove `dest`, surface git's stderr |
| `worktree add` | bare `dest` with refs | remove `dest`, propagate |

Rollback is a `Drop` guard (`Rollback` in `src/commands/clone.rs`) armed right after the
destination is known and disarmed on success, so an early `?` cannot leave a half-built
repository behind. This keeps the command retryable: leftovers at the deterministic path
would make the next attempt fail with `repository already exists at …`.

Pre-flight checks, in order (each one fails before anything is created):

1. the destination does not exist — `repository already exists at <path>`;
2. the remote's `HEAD` resolves to a branch — otherwise
   `could not determine the default branch of <url>; pass --worktree=<branch>`;
3. `--worktree=<branch>` names a valid ref (`git check-ref-format --branch`);
4. `--worktree=<branch>` exists on `origin` (unless it *is* the default branch, which step 2
   already proved);
5. the branch is not a reserved worktree name (§5.4).

## 7. Configuration surface

`rt clone` reads two keys, and the bare layout adds none:

| Key | Ordinary clone | `--worktree` |
| --- | --- | --- |
| `root` | destination's base | destination's base |
| `clone.depth` | passed as `--depth` | ignored, with a warning (§5.3) |
| `integrations.*` | §8 | §8 |

`--worktree` is opt-in per invocation, so no new key and no counter-flag. Making the bare
layout a configurable default is deliberately deferred until the flag has seen use (§12).

## 8. Integrations

After a successful clone, the **checkout** path — not the bare directory — is registered
with the enabled hooks:

```rust
if config.integrations.zoxide  { integrations::zoxide::add(&checkout).await?; }
if config.integrations.autojump { integrations::autojump::add(&checkout).await?; }
```

The path a user can `cd` into is the only useful one to register. A hook failure is a hard
error here (the checkout exists, but the command reports failure) — `rt wt add` takes the
opposite decision and downgrades the same failure to a warning. Neither is obviously right;
the inconsistency is recorded rather than resolved (both behaviours are pinned by tests).

## 9. Compatibility

- Ordinary `rt clone` is untouched: same path, same `--depth`, same stdout line.
- Existing clones stay ordinary clones. There is no conversion command, and `rt wt add`
  keeps producing `<parent>/<repo>-<branch>` siblings for them.
- `~/.rt.toml` and the path mapping rule are unchanged by `--worktree`.
- A bare repository created by `rt` is a normal bare repository: `git -C A fetch`, `git
  worktree add`, `git gc` and friends all work in it, because steps 5–9 produce git's own
  wiring rather than a private convention.

## 10. Alternatives considered

| Alternative | Why not |
| --- | --- |
| `git clone --bare` + repair | Writes ref state hostile to the layout; repairing it means recreating what steps 5–9 do anyway (§5.2). |
| `git clone --mirror` | Same mirrored-`refs/heads/*` problem, plus `remote.origin.mirror = true`, which makes every fetch rewrite local refs. |
| Bare repository in a subdirectory (`A/.bare`, checkouts in `A/`) | The mapped path would no longer be the repository, and `A/<branch>` would stop being a direct child; the tree would grow an implementation detail users have to know. |
| A dedicated `rt worktree clone` verb instead of a flag | `--worktree` reads as what it does, and `rt wt add` already covers adding checkouts to an existing project. |
| `clone.depth` honoured with a warning | Ships a layout that fails at the first cross-branch merge (§5.3). |
| Converting existing clones | Format migration on a user's repository, with no safe rollback; out of scope. |

## 11. Tests that pin this

Unit tests live next to the code (`src/path/remote.rs`, `src/path/mapping.rs`,
`src/commands/clone.rs`); anything needing a real git process lives in `tests/`, driving the
built binary through `CARGO_BIN_EXE_rt` because the crate is binary-only. Every test gets a
throwaway `HOME` (so `~/.rt.toml` is ours), a throwaway root, and an offline fixture origin
addressed through a `url.*.insteadOf` rewrite (`tests/common/mod.rs`).

| Case (`tests/clone.rs`) | Pins |
| --- | --- |
| `creates_a_bare_repository_with_the_default_worktree` | `A` is bare, `A/<default>`'s `.git` is a file, upstream and URL are wired |
| `can_pull_after_the_remote_moves` | the fetch refspec (§5.2 item 1) |
| `keeps_every_branch_usable_at_the_default_depth` | no `--single-branch` regression (§5.2 item 3) |
| `worktrees_can_be_merged_despite_the_default_depth` | full history on the worktree path (§5.3) |
| `accepts_an_explicit_branch` | `--worktree=<branch>`, bare `HEAD` untouched |
| `rejects_a_branch_that_is_not_on_the_remote` | validation before side effects |
| `rejects_a_branch_that_git_keeps_for_itself` | reserved names (§5.4) |
| `removes_a_half_built_repository_when_the_remote_is_unreachable` | rollback (§6) |
| `a_plain_clone_is_still_an_ordinary_clone` | no regression, and `clone.depth` still applies |

`tests/tree.rs` covers the rendering of both layouts; `tests/worktree.rs`, `tests/wt.rs` and
`tests/delete.rs` cover what happens to a bare layout afterwards, and are documented in
[`worktree-commands.md`](worktree-commands.md) §11.

## 12. Open questions

- **Relative worktree links.** `worktree.useRelativePaths` would make a bare layout survive
  moving `root`, at the cost of the `extensions.relativeWorktrees` requirement and older git
  refusing to read the repository.
- **A shallow opt-in** for the bare layout (`--depth=<n>` on `clone --worktree`, or a
  `[clone] worktree_depth` key), deliberately absent from v1 (§5.3).
- **The bare layout as a default** (`[clone] worktree = true`), deferred until the flag has
  seen real use.
- **Integration failure severity.** `rt clone` fails when an enabled hook is missing while
  `rt wt add` warns (§8); one of the two should change.
- **`rt tree` and stray directories.** The walk lists every non-hidden child of a bare
  repository, so a stray directory inside `A` shows up as if it were a checkout. Enumerating
  `A/worktrees/*/gitdir` instead would be precise but couples the walker to git internals.
