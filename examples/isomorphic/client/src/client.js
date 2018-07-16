import { Client } from '../isomorphic_client'

const rootNode = document.getElementById('isomorphic-rust-web-app').children[0]
console.log(rootNode)
const client = new Client(window.initialState, rootNode);

console.log('start')
  let element = client.render()
  document.body.appendChild(element)

