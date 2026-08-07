# Writing html!

### Static text

Text that will never change can be typed right into your HTML

```rust
use percy_dom::prelude::*;

html!{
  <div>
    Text goes here
    <span>{"Quoted text"}</span>
  </div>
    
};
```

### Text variables

Text variables must be wrapped in braces.

```rust
use percy_dom::prelude::*;

let text_var = " world";

html! {
  Hello { <div> { text_var } </div> }
}
```

### Attributes

Attributes work just like regular HTML.

```rust
let view = html!{
  <div id='my-id' class='big wide'></div>
};
```

### Event Handlers

```rust
html! {
    <button
      onclick=move|_event: web_sys::MouseEvent| {
        web_sys::console::log_1(&"clicked!".into());
      }
    >
      Click me!
    </button>
}
```

### Nested components

`html!` calls can be nested.

```rust
let view1 = html!{ <em> </em> };
let view2 = html{ <span> </span> }

let parent_view = html! {
  <div>
    { view1 }
    { view2 }
    {
      html! {
        Nested html! call
      }
    }
  </div>
};


let html_string = parent_view.to_string();
// Here's what the String looks like:
// <div><em></em><span></span>Nested html! call</div>
```

### Iterable Children

Any type that implements IntoIter<VirtualNode> can be used as a child element within a block.
  
```rust
let list = vec!["1", "2", "3"]
    .map(|item_num| {
      html! { 
        <li>
          List item number { item_num }
        </li>
      }
    });

html! {
  <ul> { list } >/ul>
}
```

### Comments

You can use Rust comments within your HTML

```rust
html! {
  /* Main Div */
  <div>
    <br />
    // Title
    <h2>Header</h2>
    <br />
  </div>
}
```

### Customizing the Macro

Instead of using `percy-dom`'s `html!` macro, you can create your own using the `define_html_macro!`
macro from the `html-macro` crate.

```rust
use html_macro::define_html_macro;

define_html_macro! {
    /// Builds a `VirtualNode` from a token stream.
    ///
    /// ```
    /// let div: VirtualNode<_> = my_html! { <div> Hello, world. </div> };
    /// ```
    my_html!
    
    // Specify the `virtual_node::RealDom` trait implementation to use.
    // The `virtual-dom` crate implements `RealDom` for `()` and `web_sys::Window`.
    // You can also `impl RealDom for MyType { ... }` yourself.
    // Here we indicate that the `my_html!` macro will return a `VirtualNode<web_sys::Window>`.
    real_dom = web_sys::Window,
    
    // Optionally specify the macro that the generated `my_html!` calls under the hood.
    // By default, it will call the `html_macro` crate's `html_with_config!` macro.
    calls = html_macro::html_with_config,
}

fn render() -> VirtualNode<web_sys::Window> {
    my_html! {
        <span> Quick brown fox. </span>
    }
}
```