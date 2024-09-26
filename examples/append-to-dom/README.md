# append-to-dom

An example of manually managing and adding an HTML element into the `percy-dom` DOM tree.

Such elements are outside `percy-dom`'s virtual DOM model and will not be affected by it (unless, for example, the parent element is removed).

In this example, a checkbox that maintains a configured default checkedness (i.e. does not change the `checked` HTML attribute) despite changes to `HTMLInputElement.checked`.

Note: This is a poor choice for server-side rendering and not useful for most client-side rendered `percy` applications.

_Technical note: `percy` does not currently facilitate configuring a checkbox's default checkedness independently, instead setting both the `checked` attribute and property as it's seen to be the best default for typical users of `percy`, doing client-side or server-side rendering._


---

## Running this example

```
git clone git@github.com:chinedufn/percy.git
cd percy

cargo install wasm-pack
./examples/append-to-dom/start.sh
```
