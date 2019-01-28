# Compile Time Errors

The `html-macro` provides compile time errors to help catch mistakes.

Every compile time error is tested in `crates/html-macro-ui` using the [compiletest-rs](https://github.com/laumann/compiletest-rs)
crate.

If you have an idea for an error that you don't see here [open an issue!](https://github.com/chinedufn/percy/issues/new)

#### Wrong closing tag

You've opened with one tag but are attempting to close with another.

```rust
{{#include ../../../crates/html-macro-ui/wrong_closing_tag.rs}}
```

```
{{#include ../../../crates/html-macro-ui/wrong_closing_tag.stderr}}
```

#### Should be self closing tag

The tag that you are trying to use is a self closing tagl

```rust
{{#include ../../../crates/html-macro-ui/should_be_self_closing_tag.rs}}
```

```
{{#include ../../../crates/html-macro-ui/should_be_self_closing_tag.stderr}}
```
