# css-in-rust example

This directory is an example of using the `css!` macro to write your CSS next to your Rust views.

A procedural macro generates classes for your CSS so that you can assign your CSS to your views by
applying the class.

It then writes your CSS to an `app.css` file by providing an `OUTPUT_CSS` environment variable in the
`start.sh` script.

```
./start.sh
```
