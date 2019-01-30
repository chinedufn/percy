# Fixing diff/patch issues

As our virtual dom implementation ages it will become more and more resilient, but while we're still
an experimental library it's possible that the diff/patch algorithm could fail in some scenarios.

If you notice a failure the first step is to open a new issue.

Ideally you include an example start node and end node that isn't working properly.

Let's make up an example here.

```
# Example things that you'd include in your issue.

start: html! { <div> </div>  }

end: html! { <span> </span> }

Observed error: It somehow ends up as <b></b> in my browser!
```

---

If you've opened this issue you've already made a big contribution!

If you'd like to go further, here's how to get to the root of the problem.

## Debugging Failed Diff

The easiest place to start is by adding a new diff test and seeing what patches you get.

```rust
{{#include ../../../crates/virtual-dom-rs/src/diff/diff_test_case.rs:2:}}
```

Diff patch tests get added in `diff.rs`. Here's an example:

```rust
// diff.rs

#[test]
fn add_children() {
   DiffTestCase {
       old: html! { <div> <b></b> </div> },
       new: html! { <div> <b></b> <new></new> </div> },
       expected: vec![Patch::AppendChildren(0, vec![&html! { <new></new> }])],
       description: "Added a new node to the root node",
   }.test();
}
```

To run your new test case:

```sh
# To run just your new diff test
cargo test -p virtual-dom-rs --lib my_new_test_name_here

# To run all diff tests
cargo test -p virtual-dom-rs --lib diff::tests
```

If things are failing then you've found the issue!

Please comment back on your original issue with your findings.

If everything is passing, then it must be a patching issue.

## Debugging Failed Patch

If the diff checked out, then the issue must be in the patching process.

Patches are tested in `crates/virtual-dom-rs/tests/diff_patch.rs`

A patch test case looks like this:

```rust
{{#include ../../../crates/virtual-dom-rs/tests/diff_patch_test_case/mod.rs}}
```

```rust
// Example diff patch test case.
// Found in `crates/virtual-dom-rs/tests/diff_patch.rs`

{{#include ../../../crates/virtual-dom-rs/tests/diff_patch.rs:14:27}}
```

```
# Run just your new diff patch test
wasm-pack test crates/virtual-dom-rs --chrome --headless -- --test diff_patch my_test_name_here

# Run all diff patch tests that contain the word replace
wasm-pack test crates/virtual-dom-rs --chrome --headless -- --test diff_patch replace

# Run all diff patch tests
wasm-pack test crates/virtual-dom-rs --chrome --headless
```

Create your new test case and run it to see if things fail.

If they do, update your original issue with your findings.

## Fixing the problem

Look at the documentation for the diff algorithm and the patch algorithm to get a good sense of where and how our
diffing and patching is implemented. Fixing the problem will require you to dive into that code.

As you go, if you see opportunities to make the code more understandable, DRY or better commented, seize them!

Look through your errors and try to pinpoint the exact place that the bug is stemming from. If you're stuck, continue
to update your issue with your questions and progress and someone will get back to you.
