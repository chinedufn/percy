# Router

We're working on tooling for view routing. Here's a preview of how it could look:

(Note that we're still thinking through the API so this is just a rough draft)

```rust
struct App {
  state: State,
  vutils: ViewUtils,
  router: ViewRouter
}

impl App {
    pub fn new () -> App {
        let mut router = ViewRouter::new();
        router
          .add_route(my_view)
          .not_found(four_o_four_view);

        App {
            state: State::new(),
            vutils: ViewUtils::new(),
            router
        }
    }
}

#[view_route(path = "/posts/{post_id}/authors/{name}")]
fn author_page (
    state: &State,
    vutils: &ViewUtils,
    post_id: u32,
    name: String
) -> VirtualNode {
    match state.get_post(post_id).get_author(&name) {
        Some(ref author) => {
            html! {
                { format!("Info about {}", author.name) }
            }
        }
        None => "Author does not exist"
    }
}


#[view_route(path = "/posts/{post_id/edit", before = IsAdmin)]
fn edit_post_page (
    state: &State,
    vutils: &ViewUtils,
    post_id: u32
) -> VirtualNode {
  html! { <div> { format!("Editing post {}", post_id)} </div> }
}
```
