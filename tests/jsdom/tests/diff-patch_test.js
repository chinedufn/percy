const test = require('tape')

const testUtils = require('./test-utils')
testUtils.initDOM()

const rust = require('../jsdom_tests.js')

test('Diffing and patching virtual nodes', t => {
  t.test('Correctly applies patches', testPatchRootNode)
})

const testPatchRootNode = t => {
  const patchTest = new rust.PatchTest()

  patchTest.run_tests();

  t.pass("If we made it here then none of our Rust tests panicked!")

  t.end()
}
