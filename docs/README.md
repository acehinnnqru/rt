# `rt` documentation

`rt` (**repo tree**) is a Git repository manager that stores every repository at a path
derived from its remote URL, under one configured root: `{root}/{host}/{owner}/{repo}`.

## Start here

| Document | What it is |
| --- | --- |
| [`usage.md`](usage.md) | The user guide: concepts, configuration, path mapping, the full command reference, workflows, the stdout/stderr contract, and troubleshooting. |
| [`../README.md`](../README.md) | The short pitch, project layout and build/test commands. |

## Design records

These explain *why* the implementation looks the way it does, including the git behaviour
each decision rests on. They are records of decisions taken, not proposals: the code and
the tests are the authority, and every claim of the form "git does X" was measured (the
measurement logs are in the last section of each document).

| Document | Covers |
| --- | --- |
| [`design/clone.md`](design/clone.md) | `rt clone`: the path mapping rule, the ordinary clone, the `--worktree` bare layout (algorithm, full-history requirement, reserved names), validation and rollback, and the alternatives that were rejected. Start here for the repository layout. |
| [`design/clone-worktree.md`](design/clone-worktree.md) | The original `--worktree` design draft, kept for its verification log: the measured git behaviour behind `init --bare` + `fetch`, shallow-history failure modes, and the `rt tree` / `rt delete` changes the layout forced. |
| [`design/worktree-commands.md`](design/worktree-commands.md) | The `rt worktree` (alias `wt`) command group: project resolution from the current directory, the `add` start-point table, the `list` output contract, removal semantics, and the output-contract rule. |

### Reading order

1. [`usage.md`](usage.md) — what the tool does, from the outside.
2. [`design/clone.md`](design/clone.md) — the layout and the clone algorithm.
3. [`design/worktree-commands.md`](design/worktree-commands.md) — everything that then
   operates on that layout.
4. [`design/clone-worktree.md`](design/clone-worktree.md) — the raw measurements, when a
   claim in the other documents needs backing.

## Conventions used in these documents

- **Path notation.** `A` is a repository's mapped path, i.e.
  `{root}/{host}/{owner}/{repo}`; `A/<branch>` is a checkout inside the bare layout.
- **stdout vs stderr.** Commands print their *result* (a path, a table) to stdout and
  everything else — warnings, menus, git progress — to stderr. Documents that say "print"
  mean stdout unless stated otherwise.
- **Measured claims.** Statements about git's behaviour are marked as such and carry the git
  version they were taken on (2.55.0, macOS). The integration tests are what keeps them
  true.
