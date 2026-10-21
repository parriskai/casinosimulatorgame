#!/usr/bin/env python3
"""
Generate a filtered Mermaid gitGraph from a Git repository.

Keeps commits that:
  * meet an insertion/deletion threshold,
  * add or delete files,
  * immediately precede a branch divergence,
  * are merge commits (optional; enabled by default).

Usage:
  python git_mermaid.py
  python git_mermaid.py --repo . --threshold 40 --output history.mmd
  python git_mermaid.py --threshold 100 --no-merges
  python git_mermaid.py --all-branches

Notes:
  * Uses `git log --all --topo-order` to inspect reachable history.
  * A Mermaid gitGraph is a simplified visualization: omitted commits are
    represented by edges between the retained commits.
  * Branches are inferred from commits with multiple children in the selected
    history. Complex criss-cross histories may not render exactly like Git.
"""

from __future__ import annotations

import argparse
import html
import json
import subprocess
import sys
from collections import defaultdict
from dataclasses import dataclass
from pathlib import Path
from typing import Optional


@dataclass
class Commit:
    sha: str
    parents: list[str]
    subject: str
    author: str
    date: str
    insertions: int = 0
    deletions: int = 0
    files_changed: int = 0
    added_files: int = 0
    deleted_files: int = 0
    merge: bool = False


def git(repo: str, *args: str, check: bool = True) -> str:
    result = subprocess.run(
        ["git", "-C", repo, *args],
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )
    if check and result.returncode:
        raise RuntimeError(result.stderr.strip() or f"git {' '.join(args)} failed")
    return result.stdout


def get_commits(repo: str, all_branches: bool) -> dict[str, Commit]:
    revision = ["--all"] if all_branches else ["HEAD"]
    # NUL-separated fields avoid breaking on punctuation in subjects/authors.
    fmt = "%H%x00%P%x00%s%x00%an%x00%aI"
    raw = git(repo, "log", *revision, "--topo-order", f"--format={fmt}")
    commits: dict[str, Commit] = {}

    for record in raw.splitlines():
        fields = record.split("\0")
        if len(fields) < 5:
            continue
        sha, parents_text, subject, author, date = fields[:5]
        parents = parents_text.split() if parents_text else []
        commits[sha] = Commit(
            sha=sha,
            parents=parents,
            subject=subject,
            author=author,
            date=date,
            merge=len(parents) > 1,
        )

    for sha, commit in commits.items():
        # --numstat gives per-file additions/deletions. Binary files use '-'.
        # numstat and name-status are collected separately for reliable parsing.
        numstat = git(repo, "diff-tree", "--root", "--no-commit-id", "-r",
                      "--numstat", "-M", sha)
        for line in numstat.splitlines():
            parts = line.split("\t")
            if len(parts) < 3:
                continue
            add, delete = parts[0], parts[1]
            try:
                commit.insertions += int(add)
                commit.deletions += int(delete)
            except ValueError:  # binary file
                pass

        name_status = git(repo, "diff-tree", "--root", "--no-commit-id", "-r",
                          "--name-status", "-M", sha)
        for line in name_status.splitlines():
            parts = line.split("\t")
            if not parts:
                continue
            status = parts[0]
            if status.startswith("A"):
                commit.added_files += 1
            elif status.startswith("D"):
                commit.deleted_files += 1
            if status and status[0] in {"A", "D", "M", "R", "C", "T"}:
                commit.files_changed += 1

    return commits


def branch_points(commits: dict[str, Commit]) -> set[str]:
    """Commits with more than one child are immediately before a divergence."""
    children: dict[str, set[str]] = defaultdict(set)
    for sha, commit in commits.items():
        for parent in commit.parents:
            if parent in commits:
                children[parent].add(sha)
    return {sha for sha, kids in children.items() if len(kids) > 1}


def select_commits(
    commits: dict[str, Commit],
    threshold: int,
    keep_merges: bool,
) -> set[str]:
    selected: set[str] = set()
    branch_heads = branch_points(commits)

    for sha, c in commits.items():
        large = c.insertions >= threshold or c.deletions >= threshold
        creates_or_deletes = c.added_files > 0 or c.deleted_files > 0
        if large or creates_or_deletes or sha in branch_heads or (keep_merges and c.merge):
            selected.add(sha)

    # Always include the reachable root(s), giving the graph a starting point.
    for sha, c in commits.items():
        if not any(parent in commits for parent in c.parents):
            selected.add(sha)
    return selected


def mermaid_escape(text: str) -> str:
    return text.replace('"', "'").replace("\n", " ").replace("\r", " ")


def short(sha: str, length: int = 8) -> str:
    return sha[:length]


def render_gitgraph(
    repo: str,
    commits: dict[str, Commit],
    selected: set[str],
    all_branches: bool,
    include_details: bool = True,
) -> str:
    """
    Render a gitGraph using Git's topological order.

    Mermaid gitGraph is command-sequential rather than an arbitrary graph, so
    this renderer uses explicit checkout/branch/merge commands where possible.
    """
    refs_raw = git(repo, "for-each-ref", "--format=%(refname:short) %(objectname)")
    refs: dict[str, str] = {}
    for line in refs_raw.splitlines():
        fields = line.split()
        if len(fields) == 2:
            refs[fields[1]] = fields[0]

    # Map each commit to its nearest selected ancestor(s), skipping omitted commits.
    selected_parents: dict[str, list[str]] = {}
    for sha in selected:
        found: list[str] = []
        stack = list(commits[sha].parents)
        seen: set[str] = set()
        while stack:
            parent = stack.pop(0)
            if parent in seen:
                continue
            seen.add(parent)
            if parent in selected:
                found.append(parent)
            elif parent in commits:
                stack.extend(commits[parent].parents)
        selected_parents[sha] = list(dict.fromkeys(found))

    # Use a stable topological order from git log, oldest first.
    revision = ["--all"] if all_branches else ["HEAD"]
    order_text = git(repo, "rev-list", "--topo-order", "--reverse", *revision)
    order = [sha for sha in order_text.splitlines() if sha in selected]

    if not order:
        return "gitGraph\n    commit id: \"No selected commits\"\n"

    # Mermaid's gitGraph has a linear command stream. The default branch starts
    # at the oldest retained commit; other lines are inferred from divergence.
    children: dict[str, list[str]] = defaultdict(list)
    for sha in order:
        for parent in selected_parents.get(sha, []):
            children[parent].append(sha)

    lines = ["gitGraph"]
    current_branch = "main"
    branch_for_commit: dict[str, str] = {}
    created_branches: set[str] = set()
    merged_pairs: set[tuple[str, str]] = set()
    commit_counter = 0

    def commit_label(sha: str) -> str:
        c = commits[sha]
        label = f"{short(sha)} {c.subject}"
        if include_details:
            label += f" (+{c.insertions}/-{c.deletions})"
            if c.added_files or c.deleted_files:
                label += f" [A{c.added_files}/D{c.deleted_files}]"
        return mermaid_escape(label[:100])

    # Assign labels for the branch heads from Git refs, when possible.
    ref_at_sha: dict[str, str] = {}
    for sha, name in refs.items():
        ref_at_sha.setdefault(sha, name.replace("/", "_").replace("-", "_"))

    first = order[0]
    lines.append(f'    commit id: "{short(first)}" tag: "{commit_label(first)}"')
    branch_for_commit[first] = current_branch
    last_emitted = first

    for sha in order[1:]:
        c = commits[sha]
        parents = selected_parents.get(sha, [])
        if len(parents) > 1:
            # A merge commit: return to the primary parent line and merge another
            # known branch if one is available. The commit itself is then emitted.
            primary = parents[0]
            target_branch = branch_for_commit.get(primary, current_branch)
            if target_branch != current_branch:
                lines.append(f"    checkout {target_branch}")
                current_branch = target_branch
            other_branch = None
            for parent in parents[1:]:
                candidate = branch_for_commit.get(parent)
                if candidate and candidate != current_branch:
                    other_branch = candidate
                    break
            if other_branch and (current_branch, other_branch) not in merged_pairs:
                lines.append(f"    merge {other_branch}")
                merged_pairs.add((current_branch, other_branch))
            lines.append(f'    commit id: "{short(sha)}" tag: "{commit_label(sha)}"')
            branch_for_commit[sha] = current_branch
            last_emitted = sha
            continue

        parent = parents[0] if parents else None
        if parent and len(children[parent]) > 1:
            # First child stays on current line; later children start a branch.
            siblings = children[parent]
            child_index = siblings.index(sha) if sha in siblings else 0
            if child_index > 0:
                name = ref_at_sha.get(sha, f"branch_{short(sha, 5)}")
                name = "".join(ch if ch.isalnum() or ch == "_" else "_" for ch in name)
                if not name or name[0].isdigit():
                    name = f"branch_{name}"
                base = name
                suffix = 2
                while name in created_branches or name == current_branch or name == "main":
                    name = f"{base}_{suffix}"
                    suffix += 1
                lines.append(f"    branch {name}")
                created_branches.add(name)
                current_branch = name
        elif parent and parent in branch_for_commit:
            target = branch_for_commit[parent]
            if target != current_branch and target in created_branches:
                lines.append(f"    checkout {target}")
                current_branch = target

        lines.append(f'    commit id: "{short(sha)}" tag: "{commit_label(sha)}"')
        branch_for_commit[sha] = current_branch
        last_emitted = sha

    return "\n".join(lines) + "\n"


def render_markdown_details(commits: dict[str, Commit], selected: set[str]) -> str:
    """Create a companion Markdown table with clickable local GitHub-style links."""
    lines = [
        "# Filtered Git history",
        "",
        "| Commit | Subject | Author | Date | Changes |",
        "|---|---|---|---|---|",
    ]
    for sha, c in sorted(
        ((sha, commits[sha]) for sha in selected),
        key=lambda pair: pair[1].date,
    ):
        subject = c.subject.replace("|", "\\|")
        author = c.author.replace("|", "\\|")
        lines.append(
            f"| `{short(sha)}` | {subject} | {author} | {c.date} | "
            f"+{c.insertions}/-{c.deletions}; {c.files_changed} files |"
        )
    return "\n".join(lines) + "\n"


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo", default=".", help="Git repository path (default: current directory)")
    parser.add_argument("--threshold", type=int, default=30,
                        help="Keep a commit if insertions OR deletions reach this number (default: 30)")
    parser.add_argument("--output", default="-", help="Output Mermaid file; '-' means stdout")
    parser.add_argument("--details-output", help="Optional Markdown table output path")
    parser.add_argument("--all-branches", action="store_true",
                        help="Inspect all refs instead of only commits reachable from HEAD")
    parser.add_argument("--no-merges", action="store_true", help="Do not automatically keep merge commits")
    parser.add_argument("--no-details", action="store_true",
                        help="Keep commit labels short; omit change counts from labels")
    args = parser.parse_args()

    try:
        git(args.repo, "rev-parse", "--git-dir")
        commits = get_commits(args.repo, args.all_branches)
        selected = select_commits(commits, args.threshold, not args.no_merges)
        mermaid = render_gitgraph(
            args.repo, commits, selected, args.all_branches,
            include_details=not args.no_details,
        )
        if args.output == "-":
            print(mermaid, end="")
        else:
            Path(args.output).write_text(mermaid, encoding="utf-8")
            print(f"Wrote {args.output}: {len(selected)} of {len(commits)} commits retained",
                  file=sys.stderr)

        if args.details_output:
            Path(args.details_output).write_text(
                render_markdown_details(commits, selected), encoding="utf-8"
            )
    except (RuntimeError, OSError) as exc:
        print(f"error: {exc}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
