# Getting Started

1. Rust Nightly.

    ```sh
    rustup default nightly
    rustup target add wasm32-unknown-unknown
    ```
2. [Install Geckodriver](https://github.com/mozilla/geckodriver/releases) since some of our tests are meant to run in a browser.
Put it somewhere in your path, i.e. you might move it to `/usr/local/bin/geckdriver`.

3. Download the project and make sure that you can run the test suite

    ```sh
    git clone https://github.com/chinedufn/percy
    cd percy
    ./test.sh
    ```
