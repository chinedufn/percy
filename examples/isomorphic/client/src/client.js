import { Client } from '../isomorphic_client'

console.log('ok')
console.log(window.initialState)
let client = new Client(window.initialState);
console.log(client.render())
console.log(client)
console.log('hi')

console.log('hi')
