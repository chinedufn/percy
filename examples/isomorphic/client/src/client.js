const { Client } = window.wasm_bindgen

window.global_js = new GlobalJS();

window.wasm_bindgen(`/isomorphic_client_bg.wasm`).then(main)

let client

function main () {
  client = new Client(window.initialState)

  let rootNode = document.getElementById('isomorphic-rust-web-app').children[0]
  rootNode.parentElement.replaceChild(client.render(), rootNode)
  rootNode = document.getElementById('isomorphic-rust-web-app').children[0]

  client.set_root_node(rootNode)
  client.update_dom()

}

let updateScheduled = false

// TODO:
// https://rustwasm.github.io/wasm-bindgen/api/web_sys/struct.Window.html#method.request_animation_frame
window.update = function() {
  if (!updateScheduled) {
    requestAnimationFrame(() => {
      let rootNode = document.getElementById('isomorphic-rust-web-app')
        .children[0]

      client.set_root_node(rootNode)

      client.update_dom()

      updateScheduled = false
    })
  }

  updateScheduled = true
}
