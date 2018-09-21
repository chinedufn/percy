# Writing html!

### Text

Text is rendered inside of a block `{}`

```rust
let view = html!{
  <div> {"Text goes here,"} {"or here" " or here!"}</div>
};
```

### Attributes

At this time attributes must end with a `,` due to how our `html!` macro works.

```rust
let view = html!{
  <div id='my-id',></div>
};
```

### Event Handlers

Event handlers begin with a `!` and, like attributes must end with a `,`.

Percy will attach event handlers your DOM nodes via `addEventListener`

So `!onclick` becomes `element.addEventListener('click', callback)`

```rust
pub fn render (state: Rc<State>) -> VirtualNode {
  let state = Rc::clone(&self.state);

  let view = html! {
      <button
        !onclick=move|| {
          state.borrow_mut().msg(Msg::ShowAlert)
        },>
        { "Click me!" }
     </button>
  };

  view
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
    html! {
      {"Nested html! call"}
    }
  </div>
};


let html_string = parent_view.to_string();
// Here's what the String looks like:
// <div><em></em><span></span>Nested html! call</div>
```
