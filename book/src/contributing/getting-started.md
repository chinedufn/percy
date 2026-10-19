# Getting Started

1. Rust stable, with the `wasm32-unknown-unknown` target.

    ```sh
    rustup default stable
    rustup target add wasm32-unknown-unknown
    ```
2. [Install chromedriver](https://developer.chrome.com/docs/chromedriver/downloads) since some of our tests are meant to run in a browser.
Put it somewhere in your path, i.e. you might move it to `/usr/local/bin/chromedriver`.

    To run the browser tests in Firefox or Safari instead, install geckodriver or safaridriver and
    set `GECKODRIVER=geckodriver` or `SAFARIDRIVER=safaridriver` when running `./test.sh`.

3. Download the project, install `wasm-bindgen-cli`, and make sure that you can run the test suite

    ```sh
    git clone https://github.com/chinedufn/percy
    cd percy
    cargo install wasm-bindgen-cli
    ./test.sh
    ```
