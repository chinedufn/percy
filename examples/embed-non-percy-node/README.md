# embed-non-percy-node

This example demonstrates embedding non-`percy`-controlled DOM elements within the DOM elements controlled by `percy-dom`'s virtual DOM.

Elements that are outside `percy-dom`'s virtual DOM model and will not be affected by it (unless, for example, the parent element is deleted).

## Running this example

```sh
git clone git@github.com:chinedufn/percy.git
cd percy

# Use one of the following to run the example in a headless web browser
CHROMEDRIVER=chromedriver ./examples/embed-non-percy-node/start.sh
GECKODRIVER=geckodriver ./examples/embed-non-percy-node/start.sh
SAFARIDRIVER=safaridriver ./examples/embed-non-percy-node/start.sh
```

You may need to install the appropriate driver and (optionally) add it to your `PATH`.
