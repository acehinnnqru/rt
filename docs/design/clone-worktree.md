# Design: `rt clone --worktree` — bare repository with linked worktrees

Status: implemented; kept as the original design draft and verification log. The current,
consolidated description of `rt clone` is [`clone.md`](clone.md); the user-facing guide is
[`../usage.md`](../usage.md). Where this document and the later documents disagree — §7.1's
tree walk, §7.2's repository resolution and §7.3's delete semantics in particular — the later
documents are the implemented design.
Scope: `rt clone`, `rt worktree *`, `rt tree`, `rt delete`

## 1. Motivation

`rt clone <url>` today produces an ordinary clone at `{root}/{host}/{owner}/{repo}`, and
`rt worktree add <branch>` creates a sibling `{repo}-{branch}` next to it. Working on two
branches therefore means two working trees scattered side by side, each with its own object
store, and neither path is derivable from the other.

This design adds a second, explicit layout: **one bare repository per project, with every
checkout as a linked worktree inside it**.

```
r/github.com/xxx/A/            <- bare repository: HEAD, config, objects/, refs/, worktrees/
r/github.com/xxx/A/main/       <- default worktree; .git is a *file* -> ../worktrees/main
r/github.com/xxx/A/fix-login/  <- further worktrees, one per branch
```

What this buys:

- one object store, N checkouts: `git fetch` once and every worktree sees it;
- a stable, predictable path per branch (`A/<branch>`), which is the whole point of `rt`;
- no stashing and no `git switch` when moving between branches.

Non-goals: converting existing clones, submodules inside linked worktrees, and making this
layout the default (flag-only in v1, see §9).

## 2. CLI surface

```sh
rt clone <url>                      # unchanged: ordinary clone at A
rt clone <url> --worktree           # default branch -> A/<default branch>
rt clone <url> --worktree=<branch>  # <branch> must already exist on origin -> A/<branch>
```

clap shape in `src/main.rs`:

```rust
Clone {
    /// Git remote URL
    url: String,
    /// Clone as a bare repository with a linked worktree inside it
    #[arg(long, short = 'w', num_args = 0..=1, require_equals = true, default_missing_value = "")]
    worktree: Option<String>,
}
```

- absent -> `None` -> ordinary clone, today's behaviour;
- `--worktree` -> `Some("")` -> branch is the remote's default branch;
- `--worktree=dev` -> `Some("dev")` -> branch `dev`;
- `require_equals = true` keeps the positional URL unambiguous: `--worktree dev <url>` is a
  parse error instead of a silent argument grab.

The default worktree is named after `refs/remotes/origin/HEAD` (`main`, `master`, `trunk`,
…) and is never hard-coded to `main`. `--worktree=<branch>` requires the branch to exist on
`origin`; creating new branches stays `rt worktree add`'s job (§7.2).

## 3. Clone algorithm

Every git command inherits stdio, as `clone` does today, so credential helpers and progress
output keep working. Everything that can be validated is validated before the first git
command runs, which is what makes rollback (§8) trivial.

0. `remote = Remote::parse(url)`, `dest = mapping::repo_path(&config.root, &remote)`.
1. Bail if `dest` exists — unchanged from today.
2. Probe the default branch: `git ls-remote --symref <url> HEAD`, parse
   `ref: refs/heads/<name>\tHEAD`. Fail clearly when the probe yields nothing and no
   `--worktree=<branch>` was given; with an explicit branch, verify that branch instead and
   use it as the bare repository's HEAD.
3. Resolve the requested branch (`--worktree=<branch>` or the probed default) and reject
   names that collide with git's administrative entries (§5) or that do not exist on
   `origin`.
4. `git init --bare <dest>`
5. `git -C <dest> remote add origin <url>`
6. `git -C <dest> config remote.origin.fetch '+refs/heads/*:refs/remotes/origin/*'`
7. `git -C <dest> fetch origin` — full history; `clone.depth` is deliberately not applied
   here (§4).
8. `git -C <dest> symbolic-ref HEAD refs/heads/<branch>`
   and `git -C <dest> symbolic-ref refs/remotes/origin/HEAD refs/remotes/origin/<branch>`
9. `git -C <dest> worktree add --track -b <branch> <dest>/<branch> origin/<branch>` — pass
   this path **absolute**: git resolves it relative to the `-C` directory, so a relative
   `A/dev` silently lands in `<dest>/A/dev` instead of failing (§13).
10. Integrations: `zoxide` / `autojump` add `<dest>/<branch>` — the path a user can `cd`
    into, not the bare directory.
11. Print `<dest>/<branch>` on stdout, the same scriptable contract as today.

### 3.1 Why `init --bare` + `fetch` instead of `clone --bare`

`git clone --bare` looks like the obvious primitive, but it writes state that is hostile to
this layout. From git-clone(1):

> the branch heads at the remote are copied directly to corresponding local branch heads,
> without mapping them to `refs/remotes/origin/`. When this option is used, neither
> remote-tracking branches nor the related configuration variables are created.

Consequences, in order of nastiness:

1. **No fetch refspec at all, so the mirrored heads freeze.** Measured (§13): after
   `git clone --bare`, the only `remote.*` key is `remote.origin.url` — `remote.origin.fetch`
   is absent. The branch heads are copied into `refs/heads/*` once at clone time, and a later
   `git fetch` has no refspec with which to update them. Adding the natural-looking bare
   refspec `+refs/heads/*:refs/heads/*` is worse rather than better: fetching from a worktree
   that has the branch checked out dies with `fatal: refusing to fetch into branch
   'refs/heads/main' checked out at '<dest>/main'` (measured). Step 6 avoids both problems:
   fetch only ever writes `refs/remotes/origin/*`, never a checked-out branch, so `git pull`
   works in every worktree and the branch created in step 9 ends up with a real upstream
   (`origin/<branch>`).
2. **Mirrored local branches block `-b`.** Step 9 creates the branch; refs mirrored by a
   bare clone would make it fail with `a branch named 'x' already exists`. `init --bare` plus
   `fetch` never creates local branches.
3. **`--depth` drags `--single-branch`.** git-clone(1): `--depth` *"Implies `--single-branch`
   unless `--no-single-branch` is given"*. Since `clone.depth` defaults to `1`, a naive
   `clone --bare --depth 1` produces a repository holding exactly one branch — measured: the
   resulting repo has the single ref `refs/heads/main`, so every later `rt worktree add`
   would fail. `git fetch --depth` carries no such implication: step 7 would give every branch
   its own shallow history. Ref coverage is only half the story, though — shallow history
   itself breaks the layout (§4), which is why step 7 passes no `--depth` at all.
4. **`init.defaultBranch` would lie.** `git init --bare` writes a hard-coded
   `HEAD -> refs/heads/main|master`; step 8 rewrites it to the branch the remote actually
   uses, so `git -C A symbolic-ref --short HEAD` and commit-ish-less `git worktree add`
   behave correctly.

Cost of this choice: we give up `git clone`'s convenience wiring and own the sequence
ourselves — which is exactly what the tests in §11 pin down. If steps 4–9 hit an unforeseen
git quirk, plan B is `clone --bare --no-single-branch`, then repair the refspec as in step 6
and `git branch -D` every mirrored local head.

### 3.2 Fetch semantics after the clone

`git -C A fetch` updates `refs/remotes/origin/*`; `git pull` inside any worktree works
because its branch was created with `--track` in step 9. `rt` never needs to re-implement
fetching — it only has to not break this wiring.

`worktree.useRelativePaths` stays at its default (absolute `gitdir:` in `A/main/.git`), so a
repository breaks if `root` moves. Enabling it needs the `extensions.relativeWorktrees`
extension and a recent git, which older git versions refuse to read; recorded as an open
question in §12 rather than enabled here.

## 4. Shallow history is incompatible with this layout

`clone.depth` defaults to `1`, and `--worktree` deliberately ignores it: the fetch in step 7
always takes full history. That is not caution, it is a measured incompatibility (§13), and it
is the one place where the existing clone configuration and this layout genuinely contradict
each other.

`git fetch --depth` avoids the `--single-branch` trap of §3.1 item 3, so every branch is still
present as a ref. What no refspec can fix is that the branches' **common ancestor is missing
from the object store**, and every operation that needs one breaks:

| Operation on a depth-1 bare repository with two worktrees | Measured result |
| --- | --- |
| `git merge origin/dev` in `A/main` | `fatal: refusing to merge unrelated histories` |
| `git merge-base origin/main origin/dev` | no output, exit 1 |
| `git pull --ff-only` in a worktree | works — a fast-forward needs no merge base |
| plain `git fetch origin` after new upstream commits | pulls the new tip, repository stays shallow |
| `git fetch --unshallow origin`, then the same merge | works |

A shallow repository never deepens on its own, so a `--worktree` clone made with the default
config would look healthy: it clones fast, fetches and fast-forwards normally, and then fails
the first time two worktrees are actually merged or rebased — the very operation this layout
exists for. Deferring the failure to merge time is worse than paying for history up front.

Decision: `--worktree` fetches full history and prints a single warning line when it overrides
a non-zero `clone.depth`, naming the reason. Shallow worktree layouts stay reachable by hand
(`git -C A fetch --depth=<n>`); an explicit opt-in (`--depth=<n>` flag or a
`[clone] worktree_depth` key) is an open question in §12, not part of v1.

Rejected alternatives:

- **Error out when `clone.depth > 0`.** Since `1` is the default, this would make `--worktree`
  unusable out of the box and force a config edit before first use.
- **Honour `clone.depth` and warn.** That is precisely the silent-failure design described
  above, shipped on purpose.
- **A "large enough" `--depth=N`.** Any depth truncates: branches whose merge base is older
  than `N` break in exactly the same way, so only full history is reliably safe.

## 5. Reserved worktree names

The worktree path is a child of the git directory, so the first path component of a branch
name shares a namespace with git's own administrative entries (gitrepository-layout(5)):

```
HEAD  config  config.worktree  description  index  packed-refs  shallow  commondir  gitdir
objects/  refs/  branches/  hooks/  info/  logs/  remotes/  modules/  worktrees/
```

A branch called `objects` or `worktrees` is valid to git, but checking it out at `A/objects`
or `A/worktrees` would collide with the object store or the worktree registry. `rt clone
--worktree` therefore rejects such names before creating anything, with an error that names
the offending component. The same constant is reused by `rt tree` (§7.1).

Git does not catch these for us, and one of them is not caught at all — measured (§13):

- `objects`: the add fails with `fatal: '<dest>/objects' already exists`, but only after
  creating a stray local branch `objects`; the check works by accident, because the directory
  happens to be non-empty.
- `worktrees`: the add **succeeds** (exit 0). The worktree directory is named `worktrees`, so
  git registers it at `$GIT_DIR/worktrees/worktrees/` — the git administration directory ends
  up *inside the checkout it administers*, and `git worktree prune` afterwards reports
  `Removing worktrees/.git: not a valid directory`.

The reserved-name guard is therefore a correctness requirement, not a nicer error message.

Branch names containing `/` (`feature/login`) map to `A/feature/login`: only the first
component is checked against the reserved list, and the intermediate directories are not
worktrees themselves. Flattening (`A/feature-login`) was rejected because it would make the
worktree path underivable from the branch name, which is the property users actually want.

## 6. Layout facts `rt` may rely on

- `A` is bare: `git -C A rev-parse --is-bare-repository` prints `true`, and `A/.git` does not
  exist. Detection from the filesystem alone: `A/HEAD` is a file while `A/objects` and
  `A/refs` are directories.
- `A/<branch>` is a linked worktree: `.git` is a *file* whose content is
  `gitdir: <A>/worktrees/<id>`, which is what `rt tree` already treats as a leaf.
- The worktree registry lives in `A/worktrees/<id>/`, documented in git-worktree(1) DETAILS,
  with the absolute path of the worktree in the `gitdir` file of each entry.

## 7. Changes to existing commands

### 7.1 `rt tree` (a bug fix this feature makes mandatory)

`is_repo()` only looks for `dir.join(".git")`, so a bare repository is not recognised as a
leaf: `rt tree` would descend into `A/objects/**` and `A/refs/**` and print thousands of
nonsense lines. This was already broken for any bare repository under `root` before the
feature; the feature just made it the common case.

Plan:

- add `is_bare(dir)`: `dir/HEAD` is a file, `dir/objects` and `dir/refs` are
  directories;
- when a directory is bare, print it and then descend, skipping git's administrative entries
  (§5) — linked worktrees are direct children and stop the walk themselves;
- share one `git::ADMIN_ENTRIES` constant with the reserved-name check.

Resulting output:

```
github.com
└── xxx
    └── A
        ├── fix-login
        └── main
```

Alternative considered: enumerate `A/worktrees/*/gitdir` to list exactly the registered
worktrees. More precise, but it couples the tree walker to git internals and can print paths
outside `root`; keep it as a follow-up if stray directories inside the bare repository
become a problem.

### 7.2 `rt worktree add <branch>` — resolve the project, stop guessing from cwd

Today it derives `{parent}/{repo_name}-{branch}` from the current directory, so running it
inside `A/main` would produce `A/main-hotfix`. New rule: resolve the project root the way
`rt` already resolves everything else.

1. `git remote get-url origin` in the current repository -> `Remote::parse` ->
   `mapping::repo_path(&config.root, &remote)`.
2. If that path is a bare repository (§6), the project uses this layout: create
   `git worktree add --track -b <branch> <dest>/<branch> origin/<branch>` when
   `origin/<branch>` exists, otherwise create the branch from `origin/HEAD`.
3. Otherwise keep today's sibling `{parent}/{repo}-{branch}` behaviour, so existing users see
   no change.
4. Still print the created path.

At the time of writing this design, the command's `Config` parameter was unused; the
implemented command uses it only for the integrations.

### 7.3 `rt delete`

- `rt delete <url|path>` keeps using `remove_dir_all`, which removes the bare repository and
  every worktree with it. Print the path as before.
- `rt delete` with no argument inside a linked worktree deleted just that directory at the
  time of writing and left a dangling `A/worktrees/<id>` registration behind. The target
  there is a worktree, not a repository, so use `git worktree remove <path>` instead: git
  refuses when the tree is dirty (better than silently discarding work) and prunes the
  registration itself. Removing the whole project still requires naming it (`rt delete A` or
  the URL).
- Deleting a bare repository that still has registered worktrees stays allowed and takes the
  worktrees with it.

Current behaviour: every deletion is confirmed (with a second question when a checkout holds
uncommitted work) unless `--force`; see [`../usage.md`](../usage.md) §5.

## 8. Failure and rollback

Because validation happens before the repository is created, the repository on disk was
created by this run and may be removed on failure.

| Failure | State on disk | Behaviour |
| --- | --- | --- |
| bad URL / unknown branch / reserved name | no repository at `dest` (parent directories may exist) | error before the repository is created |
| `init --bare`, `remote add`, `config` | empty `dest` | remove `dest`, propagate the error |
| `fetch` (network, auth, empty remote) | bare `dest`, no refs | remove `dest`, surface git's stderr |
| `worktree add` | bare `dest` with refs | remove `dest`, propagate |

Removing what we created keeps `rt clone` retryable: leaving a half-built `A` behind would
make the next attempt fail with `repository already exists at ...`. If the cleanup itself
fails, report the original error together with the leftover path.

## 9. Configuration

No new keys in v1: `--worktree` is opt-in per invocation, so nothing changes for existing
users and no `--no-worktree` counter-flag is needed. `clone.depth` keeps applying to ordinary
clones and is overridden, with a warning, on the worktree path (§4). A future
`[clone] worktree = true` (turning the bare layout into the default) is deliberately out of
scope until the flag has seen use; `README.md` and [`../usage.md`](../usage.md) document the
flag as implemented.

## 10. Compatibility

- Ordinary `rt clone` is untouched; existing clones stay ordinary clones.
- No conversion command: turning an existing clone into this layout is not supported, and
  `rt worktree add` keeps producing siblings for ordinary clones (§7.2 step 3).
- `clone.depth`, `~/.rt.toml` and the path mapping rule are unchanged.

## 11. Testing

Unit tests stay next to the code; anything that needs a real git process goes into `tests/`.
The crate is binary-only, so integration tests drive the built binary through
`env!("CARGO_BIN_EXE_rt")` and cannot `use rt::...`.

Each integration test gets a throwaway `HOME` (so `~/.rt.toml` is ours), a throwaway root, and
a fixture origin created with `git init` plus one commit and at least a second branch,
addressed as a `file://` URL; temp state lives under `std::env::temp_dir()` and is removed by
a `Drop` guard. Fixture commits need `-c user.email=... -c user.name=...
-c commit.gpgsign=false`: on a machine with `commit.gpgsign = true` the fixture fails with
`gpg failed to sign the data` before a single assertion runs (hit while writing §13).

| # | Case | Pins |
| --- | --- | --- |
| 1 | `clone --worktree` leaves `A` bare and `A/<default>` a linked worktree | **the assumption that a worktree may live inside the bare git directory** — confirmed by hand on git 2.55.0 (§13); the test keeps it true |
| 2 | `git -C A/<default> pull --ff-only` after a new upstream commit | §3.1 item 1 (fetch/pull wiring) |
| 3 | With `clone.depth = 1`, `rt worktree add <other-branch>` succeeds | §3.1 item 3 (`--single-branch` regression) |
| 4 | `--worktree=<branch>` selects it; `--worktree=<unknown>` fails and `A` does not exist | validation before side effects |
| 5 | `--worktree=objects` is rejected and `A` does not exist | §5 reserved names |
| 6 | Unreachable URL fails and leaves no `A` | §8 rollback |
| 7 | `rt tree` prints `A` with its worktrees and no `objects`/`refs` noise | §7.1 |
| 8 | `rt clone <url>` without the flag still produces an ordinary clone | no regression |
| 9 | With the default config (`clone.depth = 1`), `clone --worktree` still yields full history: `git -C A/<default> merge origin/<other>` succeeds | §4 (shallow history breaks cross-worktree merges) |

Implemented in `src/git.rs` (layout facts and command helpers), `src/commands/clone.rs`,
`src/commands/tree.rs`, `src/commands/worktree/add.rs` and `src/commands/delete.rs`, with the
integration suite in `tests/` (binary-only crate, so the tests drive `CARGO_BIN_EXE_rt`; see
`tests/common/mod.rs` for the fixture). Unit tests cover bare detection, the reserved-name
list and the clone pre-flight checks. The measurements from §13 are now assertions: cases 1–3
and 9 in particular fail if git's behaviour ever changes underneath them.

Two things the implementation added to the design, both found by the tests:

- `git remote get-url` expands `url.*.insteadOf`, so a repository reached through a rewrite
  resolves to a different path than the one `rt clone` mapped. `rt worktree add` reads
  `git config --get remote.origin.url` instead (§7.2).
- `git init` and `git worktree add` write progress to **stdout**, which would pollute the
  scriptable path that `rt` prints; both run with `--quiet`.

## 12. Decisions and open questions

Decided:

- the default worktree is named after the remote's default branch, not hard-coded to `main`;
- the bare repository lives at `A` itself (not `A/.bare`), so `A/<branch>` is a direct child
  and every checkout is a linked worktree;
- `--worktree` ignores `clone.depth` and fetches full history, warning once, because shallow
  history breaks merges between worktrees (§4);
- §7.1 descends into a bare repository while skipping git's administrative entries, so `rt
  tree` shows `A` with one child per worktree;
- §7.3 removes a linked worktree with `git worktree remove`, which refuses unclean trees and
  prunes the registration; naming the repository (path or URL) still deletes everything, and
  an existing path is no longer mistaken for a remote URL;
- this document first, implementation afterwards.

Open:

- whether `worktree.useRelativePaths` should be enabled for repositories under a movable
  `root`, given the git-version requirement;
- whether the bare layout ever becomes a configuration default (§9);
- an explicit shallow opt-in for this layout (`--depth=<n>` flag or a `[clone] worktree_depth`
  key), deliberately absent from v1 (§4).

## 13. Verification log

Every load-bearing claim above was executed by hand before this document was written, because
the layout rests on git behaviour rather than on our own code. Environment: git 2.55.0, macOS,
2026-09-11. Fixture: a local `git init -b main` repository with one commit and branches
`dev`, `objects`, `worktrees`, addressed as a `file://` URL. The §4 rows use a second fixture
with real divergence — 3 commits on `main`, 2 on `dev`, merge base at the root commit — so the
common ancestor sits beyond a depth-1 boundary.

| Claim | Command | Result |
| --- | --- | --- |
| §1 / §11 case 1 a worktree may live inside the bare repository | `git clone --bare <url> A`, then `git -C A worktree add <abs>/A/main main` | exit 0; `A/main/.git` holds `gitdir: <A>/worktrees/main`; `git -C A rev-parse --is-bare-repository` prints `true` |
| §3.1.1 `clone --bare` leaves no fetch refspec | `git -C A config --get-regexp '^remote\.'` | only `remote.origin.url` |
| §3.1.1 the pull trap | refspec set to `+refs/heads/*:refs/heads/*`, then `git -C A/main fetch origin` | `fatal: refusing to fetch into branch 'refs/heads/main' checked out at '<A>/main'` |
| §3.1.3 `--depth` implies `--single-branch` | `git clone --bare --depth 1 <url> C` | a single ref, `refs/heads/main` |
| §3 steps 4–9 work end to end | the algorithm above with `clone.depth = 1` | `refs/heads/main` plus `refs/remotes/origin/{HEAD,dev,main}`; `main@{upstream}` = `origin/main`; a second worktree for `dev` exits 0 |
| §3 step 9, absolute paths | `git -C E worktree add E/rel-1 dev` | exit 0, silently created `<E>/E/rel-1` |
| §5 reserved name `objects` | `git -C F worktree add -b objects <abs>/F/objects origin/objects` | exit 128, `fatal: '<F>/objects' already exists`; a stray branch `objects` is left behind |
| §5 reserved name `worktrees` | `git -C G worktree add -b worktrees <abs>/G/worktrees origin/worktrees` | exit 0; registry `G/worktrees/worktrees/` created inside the checkout; `git worktree prune -n -v` then reports `Removing worktrees/.git: not a valid directory` |
| §4 depth 1 breaks cross-branch merges | depth-1 fetch, two worktrees, `git merge origin/dev` in `main` | `fatal: refusing to merge unrelated histories`; `git merge-base origin/main origin/dev` prints nothing, exit 1 |
| §4 a shallow repository never deepens by itself | plain `git fetch origin` after a new upstream commit | new tip fetched, `shallow` file still present, merge base still missing; `git pull --ff-only` in the worktree succeeds (exit 0) |
| §4 the escape hatch works | `git fetch --unshallow origin`, then the same merge | full history restored, merge succeeds |

Three further measurements came out of implementing the `rt` side, and each one is now covered
by a test:

| Claim | Command | Result |
| --- | --- | --- |
| `git remote get-url` rewrites URLs | with `url."git@example.com:".insteadOf` configured, `git remote get-url origin` | prints the rewritten `file://…` URL; `git config --get remote.origin.url` prints the configured one |
| `git init` and `git worktree add` write progress to stdout | `git worktree add -b q1 <path> main`, with and without `--quiet` | without `--quiet`: `HEAD is now at …` on stdout; with it: stdout empty |
| `git worktree remove` guards and prunes | remove a dirty worktree, then a clean one | dirty: `fatal: '…' contains modified or untracked files, use --force to delete it`; clean: directory gone and the `worktrees/<id>` entry pruned |

Not executed: `worktree.useRelativePaths` (§12).
