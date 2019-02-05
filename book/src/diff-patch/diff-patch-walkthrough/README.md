# Diff / Patch Walkthrough

From a user's perspective, rendering on the client side looks roughly like this:

```rust
// Create a first virtual DOM in application memory then
// use this description to render into the real DOM
let old_vdom = html! { <div> Old </div> };
dom_updater.update(old_vdom);

// Create a second virtual DOM in application memory then
// apply a minimal set of changes to the DOM to get it to look like
// this second virtual DOM representation
let new_vdom = html! { <div> New </div> }
dom_updater.update(new_vdom);


// Create a thid virtual DOM in application memory then
// apply a minimal set of changes to the DOM to get it to look like
// this second virtual DOM representation
let new_vdom = html! { <div> <span>Very New</span> </div> }
dom_updater.update(new_vdom);
```

On the code side of things, the process is

1. Compare the old virtual DOM with the new virtual DOM and generate a `Vec<Patch<'a>>`

2. Iterate through `Vec<Patch<'a>>` and apply each of those patches in order to update the real DOM
that the user sees.

## Diffing

Let's say that you have an old virtual dom that you want to update using a new virtual dom.

 ```ignore
     Old vdom             New vdom

     ┌─────┐             ┌─────┐
     │ Div │             │ Div │
     └─────┘             └─────┘
        │                   │
   ┌────┴─────┐        ┌────┴─────┐
   ▼          ▼        ▼          ▼
┌────┐     ┌────┐   ┌────┐     ┌────┐
│Span│     │ Br │   │Img │     │ Br │
└────┘     └────┘   └────┘     └────┘
```

In our example the only thing that has changed is that the `Span` has become a `Img`.

So, we need to create a vector of patches that describes this.

Our diffing algorithm will recursively iterate through the virtual dom trees and generate a vector
of patches that looks like this:

```rust
// Our patches would look something like this:
let patches = veec![
    // The real generated patch won't use the `html!` macro,
    // this is just for illustration.
    Patch::Replace(1, html! { <span> </span> }),
];
```

This patch says to replace the node with index of 1, which is currently a `<br>` with a `<span>`.

How does the diffing algorithm determine the index?

As we encounter nodes in our old virtual dom we increment a node index, the root node being index 0.
Nodes are traversed depth first by recursively diffing children before proceeding to siblings.

 ```ignore
// Nodes are indexed depth first.

             .─.
            ( 0 )
             `┬'
         ┌────┴──────┐
         │           │
         ▼           ▼
        .─.         .─.
       ( 1 )       ( 4 )
        `┬'         `─'
    ┌────┴───┐       │
    │        │       ├─────┬─────┐
    ▼        ▼       │     │     │
   .─.      .─.      ▼     ▼     ▼
  ( 2 )    ( 3 )    .─.   .─.   .─.
   `─'      `─'    ( 5 ) ( 6 ) ( 7 )
                    `─'   `─'   `─'
 ```

## Patching


There are several different types of patches that are described in our `Patch` enum.

 ```rust
{{#include ../../../../crates/virtual-dom-rs/src/patch/mod.rs}}
 ```

When patching we iterate over our vector of patches, look at the node index for the patch, then
traverse the real DOM in order to find the corresponding DOM element.

So if a patch applies to the node with index 4, we'll start at our root node (node 0) and crawl it's children
and it's children's children until we've gone through node 1, 2 and 3.
