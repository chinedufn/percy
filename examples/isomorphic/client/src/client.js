import { Client } from '../isomorphic_client'

const rootNode = document.getElementById('isomorphic-rust-web-app').children[0]
console.log(rootNode)

const client = new Client(window.initialState, rootNode);

let updateScheduled = false

export function update() {
  if (!updateScheduled) {
    requestAnimationFrame(() => {
      client.update_dom()
      updateScheduled = false
    })
  }

  updatedScheduled = true
}

console.log('start')
  let element = client.render()
  document.body.appendChild(element)

