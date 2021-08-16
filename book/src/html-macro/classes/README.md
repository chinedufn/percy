# Classes

```rust,no_run,ignore
let _node = html! {
    <div class="some classes here">
        <span class=["array", "works", "too"]></span>

        <strong class=vec!["vec", "works", "as", "well"]></strong>

        <em class=vec!["vec", "works", "as", "well"]></em>

        <label class=["as_ref", "str", "works", CssClass::BigButton]></label>
    </div>
};

enum CssClass {
    BigButton
}

impl AsRef<str> for CssClass {
	fn as_ref (&self) -> &str {
		match self {
		    Self::BigButton => "big-button"
		}
	}
}
```
