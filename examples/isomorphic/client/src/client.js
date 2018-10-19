import { Client } from '../build/isomorphic_client'

console.log('ok')

const client = new Client(window.initialState)

let rootNode = document.getElementById('isomorphic-rust-web-app').children[0]
rootNode.parentElement.replaceChild(client.render(), rootNode)
rootNode = document.getElementById('isomorphic-rust-web-app').children[0]

client.set_root_node(rootNode)
client.update_dom()

let updateScheduled = false

export function update() {
  console.log('UPDATE called!')
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
