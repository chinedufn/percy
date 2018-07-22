const test = require('tape')

const testUtils = require('./test-utils')
testUtils.initDOM()

const rust = require('../jsdom_tests.js')

test('Creating DOM elements from virtual nodes', t => {
  t.test('Patch root node', testPatchRootNode)
})

const testPatchRootNode = t => {
  const patchTest = new rust.PatchTest()

  patchTest.run_tests();

  t.pass("If we made it here then none of our Rust tests panicked!")

  t.end()
}
