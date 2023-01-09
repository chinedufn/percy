# Compile Time Errors

The `html-macro` provides compile time errors to help catch mistakes.

Every compile time error is tested in `crates/html-macro-ui` using the [trybuild](https://github.com/dtolnay/trybuild)
crate.

If you have an idea for an error that you don't see here [open an issue!](https://github.com/chinedufn/percy/issues/new)

Here are a few examples:

#### Wrong closing tag

You've opened with one tag but are attempting to close with another.

```rust
{{#include ../../../../crates/html-macro-test/src/tests/ui/wrong_closing_tag.rs}}
```

```
{{#include ../../../../crates/html-macro-test/src/tests/ui/wrong_closing_tag.stderr}}
```

#### Should be self closing tag

The tag that you are trying to use is a self closing tagl

```rust
{{#include ../../../../crates/html-macro-test/src/tests/ui/should_be_self_closing_tag.rs}}
```

```
{{#include ../../../../crates/html-macro-test/src/tests/ui/should_be_self_closing_tag.stderr}}
```

#### Invalid HTML tag

You're trying to use a tag that isn't in the HTML specification.
This might happen if you've made a typo.

```rust
{{#include ../../../../crates/html-macro-test/src/tests/ui/invalid_html_tag.rs}}
```

```
{{#include ../../../../crates/html-macro-test/src/tests/ui/invalid_html_tag.stderr}}
```

#### on create element without key

You set the `on_create_element` but did not set a key.

```rust
{{#include ../../../../crates/html-macro-test/src/tests/ui/on_create_element_without_key.rs}}
```

```
{{#include ../../../../crates/html-macro-test/src/tests/ui/on_create_element_without_key.stderr}}
```

#### on remove element without key

You set the `on_remove_element` but did not set a key.

```rust
{{#include ../../../../crates/html-macro-test/src/tests/ui/on_remove_element_without_key.rs}}
```

```
{{#include ../../../../crates/html-macro-test/src/tests/ui/on_remove_element_without_key.stderr}}
```
