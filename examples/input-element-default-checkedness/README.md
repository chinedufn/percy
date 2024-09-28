# input-element-default-checkedness

This example demonstrates an input element that maintains a **explicitly configured default checkedness** (i.e. does not change the `checked` HTML attribute) despite changes to `HTMLInputElement.checked`.

`percy` does not support explicitly setting the default checkedness ([why?](#why-does-percy-override-default-checkedness)), so this also serves as an example of **appending independently-managed DOM elements to the DOM elements controlled by `percy-dom`'s virtual DOM**. Elements that are outside `percy-dom`'s virtual DOM model and will not be affected by it (unless, for example, the parent element is removed).

---

### When Should I Configure Default Checkedness Independently of Checkedness?

This is irrelevant to server-side rendering, where there is no server-side DOM and only the HTML attributes are sent.

This is not generally useful for client side rendering, as the default only matters if you want to configure what happens when checkedness isn't specified and `HTMLFormElement.reset()` is called or similarly, a reset element is pressed (which changes the element's checkedness to the default checkedness).
- Most `percy` applications are expected to have a client-side application state, with GUI reflecting that app state. This is in contrast to UI-driven apps (which often scale poorly as complexity increases). In this case, the checkedness of an input is always specified according to app state.
- Calling external functions that independently affect DOM elements such as `HTMLFormElement.reset()` may cause the DOM state may desync from the virtual DOM and app state, and hence is typically best avoided.

---

### Why Does `percy` Override Default Checkedness?

- Server-side rendering sends HTML and HTML attributes only, no DOM properties. Therefore `percy`'s `virtual-node` needs to set the `checked` attribute (default checkedness) for server side rendering, using `VirtualNode`'s `Display` implementation i.e. `virtual_node.to_string()`
- `percy-dom` is intended for client-side rendering, but it overrides the default checkedness to match `VirtualNode`'s `Display` implementation.

---

## Running this example

```sh
git clone git@github.com:chinedufn/percy.git
cd percy

# Use one of the following to run the example in a headless web browser
CHROMEDRIVER=chromedriver ./examples/input-element-default-checkedness/start.sh
GECKODRIVER=geckodriver ./examples/input-element-default-checkedness/start.sh
SAFARIDRIVER=safaridriver ./examples/input-element-default-checkedness/start.sh
```

You may need to install the appropriate driver and (optionally) add it to your `PATH`.
