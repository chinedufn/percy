const test = require('tape')

const testUtils = require('./test-utils')
testUtils.initDOM()

const rust = require('../jsdom_tests.js')

test('Creating DOM elements from virtual nodes', t => {
  t.test('Nested divs', testNestedDivs)
  t.test('Sets element properties', testElementProps)
  t.test('Click event', testClickEvent)
})

const testNestedDivs = t => {
  const nestedDivs = rust.nested_divs()

  t.equal(nestedDivs.innerHTML, '<div><div></div></div>')

  t.end()
}

const testElementProps = t => {
  const divsWithProps = rust.div_with_properties()

  t.equal(divsWithProps.id, "id-here")
  t.deepEqual(divsWithProps.classList, ['two', 'classes']);

  t.end()
}

const testClickEvent = t => {
  const clickTest = new rust.ClickTest()

  const div = clickTest.div_with_click_event();

  const clickEvent = new window.Event("click")

  t.equal(clickTest.get_clicked(), false);

  div.dispatchEvent(clickEvent)

  t.equal(clickTest.get_clicked(), true);

  t.end()
}
