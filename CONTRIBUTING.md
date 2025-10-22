# Maintainer Guide

This document is for maintainers (core contributors with write access).

It defines how we work together — ensuring stable releases, consistent history, and respectful collaboration.

---

## 🧩 Roles

| Role | Responsibility |
|------|----------------|
| **Maintainer** | Reviews PRs, merges to `dev`, ensures code quality |
| **Lead Maintainer** | Approves releases, manages roadmap, reviews key design decisions |
| **Contributor** | Submits PRs via forks |

All maintainers are equal in rights, but each has clear accountability for the areas they work on.

---

## 🌿 Branch Strategy

| Branch | Purpose | Rules |
|---------|----------|-------|
| `main` | Stable, tagged releases | Protected; no direct pushes |
| `dev` | Active development | Reviewed merges only |
| `feat/*` | New features | Squash or rebase into `dev` |
| `fix/*` | Bug fixes | Same as feature branches |
| `release/*` | Pre-release stabilization | Tag + merge into `main` |

---

## 🔁 Merge Rules

### ✅ Allowed
- Squash or rebase merges (no merge commits)
- PRs reviewed by **another maintainer**
- Hotfix commits directly to `main` *only if urgent* and agreed upon

### 🚫 Not allowed
- Force pushes to `main`
- Merging unreviewed or failing builds
- Long-lived personal branches

---

## 🔍 Review Process

1. Each PR must be reviewed by **one other maintainer**.
2. Keep discussions respectful and technical.
3. If both maintainers agree → merge via squash/rebase.
4. If disagreement → pause merge, discuss in Discord/Issue until consensus.

---

## 🪪 Branch Protection Settings (Recommended)

- Require pull request reviews
- Require status checks to pass
- Require linear history
- Disallow direct pushes to `main`
- Allow admins to bypass for emergencies only

---

## 🧾 Release Process

1. Create branch:
   ```bash
   git switch -c release/vX.Y.Z
````

2. Update `CHANGELOG.md`.
3. Run full test/build suite.
4. Tag the release:

   ```bash
   git tag -a vX.Y.Z -m "Release vX.Y.Z"
   git push origin vX.Y.Z
   ```
5. Merge into `main`, then back into `dev`.

---

## 🧭 Syncing Changes Between Maintainers

When both maintainers are active:

```bash
git fetch origin
git rebase origin/dev
```

Before pushing:

```bash
git push --force-with-lease
```

*(never `--force` without `--lease`)*

---

## 🗂️ Documentation & Communication

* Discuss roadmap openly in issues or discussions.
* Prefer asynchronous communication (issues, PRs) over private chat.
* Use `TODO.md` or project board for tracking upcoming work.

---

## 🧘 Guidelines

* Maintain a professional tone in reviews.
* Keep the repo history readable.
* Small, focused PRs > large, messy ones.
* Always document design or behavioral changes.

---

## ❤️ Thank You

Maintaining open source is hard work.
Always remember — quality, transparency, and mutual respect matter more than speed.

```
