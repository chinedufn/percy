const fs = require('fs')
const path = require('path')
const test = require('tape')

const testUtils = require('./test-utils')
testUtils.initDOM()

const rust = require('../jsdom_tests.js')

test('Creating DOM elements from virtual nodes', t => {
  t.test('Nested divs', testNestedDivs)
  t.test('Sets element properties', testElementProps)
})

const testNestedDivs = t => {
  const div = rust.nested_divs()

  const nestedDivs = rust.nested_divs()

  t.equal(nestedDivs.innerHTML, '<div><div></div></div>')

  t.end()
}

const testElementProps = t => {
  t.end()
}
