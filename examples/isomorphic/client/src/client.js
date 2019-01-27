// TODO: Remove webpack and just include this stuff in `index.html`

const { Client } = window.wasm_bindgen

window.global_js = new GlobalJS()

window.wasm_bindgen(`/isomorphic_client_bg.wasm`).then(main)

let client

function main () {
  client = new Client(window.initialState)
}

let updateScheduled = false

// TODO:
// https://rustwasm.github.io/wasm-bindgen/api/web_sys/struct.Window.html#method.request_animation_frame
window.update = function () {
  if (!updateScheduled) {
    requestAnimationFrame(() => {
      client.render()

      updateScheduled = false
    })
  }

  updateScheduled = true
}

