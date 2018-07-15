const test = require('tape')

const testUtils = require('./test-utils')
testUtils.initDOM()

const rust = require('../jsdom_tests.js')

test('Creating DOM elements from virtual nodes', t => {
  t.test('Patch root node', testPatchRootNode)
})

const testPatchRootNode = t => {
  const patchTest = new rust.PatchTest()

  patchTest.patch_element();


  let patchedElem = document.getElementById('patched');
  t.equal(patchedElem.innerHTML, 'Patched element')

  t.end()
}
