# percy-preview

`percy-preview` helps you render an interactive preview of any view component in any state.

## Usage

Create a preview crate that depends on your application crate.

```
name = "my-app-preview"

[lib]
crate-type = ["cdylib"]

[dependencies]
my-app = {path = "/path/to/my-app"}
percy-preview-app = {version = "0.1"}
```

Your preview crate uses `percy-preview-app` to create an application that can render the `Vec<Preview>` that your app crate exposes.

```
// TODO... illustrate this.. actually better yet add an example preview application to the crate... and just let this documentation
// focus on explaining the high level details before linking you off to the full example.
```

Here's an example of a function that can be used to preview one of your views.

```rust
use percy_dom::prelude::*;

struct MyView {
    count: u8,
    increment_count: Box<dyn FnMut() -> ()>
}

impl View for MyView {
	fn render(self) -> VirtualNode {
        let MyView { count, mut increment_count } = self;

		html! {
		    <div
		        on_click=move || { increment_count() }
		    >
		        The count is { count }
		    </div>
		}
	}
}

#[cfg(feature = "preview")]
mod previews {
    use super::MyView;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;

    use percy_dom::prelude::*;
    use percy_preview::{Preview, Rerender};

    pub fn preview_my_view(rerender: Rerender>) -> Preview {
        let count = Arc::new(AtomicU32::new(0));
        let count_clone = count.clone();

        let increment_count = Box::new(|| {
            count_clone.fetch_add(1, Ordering::SeqCst);
            rerender()
        });

        let render = move || {
            let view = MyView {
                count: count.load(Ordering::SeqCst),
                increment_count,
            };

            html! {
                <div> { view } </div>
            }
        };

        Preview {
            name: "My View".to_string(),
            render: Box::new(render),
        }
    }
}
```
