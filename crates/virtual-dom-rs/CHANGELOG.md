# virtual-dom-rs Changelog

Types of changes:

- `[added]` for new features.
- `[changed]` for changes in existing functionality.
- `[deprecated]` for once-stable features removed in upcoming releases.
- `[removed]` for deprecated features removed in this release.
- `[fixed]` for any bug fixes.
- `[security]` to invite users to upgrade in case of vulnerabilities.

## Not Yet Published

_Here we list notable things that have been merged into the master branch but have not been released yet._

- ...

## 0.6.1 - Feb 22, 2019

- [fixed] Fix DomUpdater not storing closures for nodes that were created during `Patch::AppendChildren`
 and `Patch::Replace`
  - [Issue](https://github.com/chinedufn/percy/issues/70)
