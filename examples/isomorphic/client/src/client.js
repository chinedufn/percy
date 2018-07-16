import { Client } from '../isomorphic_client'

console.log('ok')
const rootNode = document.getElementById('isomorphic-rust-web-app').children[0]
console.log(rootNode)

const client = new Client(window.initialState, rootNode);

let updateScheduled = false

export function update() {
  console.log('UPDATE called!')
  if (!updateScheduled) {
    requestAnimationFrame(() => {
      client.update_dom()
      updateScheduled = false
    })
  }

  updateScheduled = true
}

console.log('start')
  let element = client.render()
  document.body.appendChild(element)

