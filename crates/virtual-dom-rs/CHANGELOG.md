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

- [fixed] Proper spacing in between text nodes and elements in all cases [PR](TODO: Link here)

## 0.6.5 - Mar 4, 2019

- [added] Start supporting braced text in the `html!` macro [#96](https://github.com/chinedufn/percy/pull/96)
- [removed] Removed the `text!` macro

 ```rust
 let hello = "hello world";
 html! { {hello} }
 ```


## 0.6.4 - Feb 24, 2019

- [fixed] Using the `html!` macro to create an event now uses the fully qualified path to `std::rc::Rc`
- [added] Started adding key support. If a VirtualNode's key attribute changes it will lead to a `Replace` patch.

```rust
// example
html! { <div key="5"></div> }`;
````

## 0.6.1 - Feb 22, 2019

- [fixed] Fix DomUpdater not storing closures for nodes that were created during `Patch::AppendChildren`
 and `Patch::Replace`
  - [Issue](https://github.com/chinedufn/percy/issues/70)
