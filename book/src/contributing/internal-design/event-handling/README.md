# Event Handling

There are two categories of events, delegated and per-node events.

## Delegated Events

For delegated events, we attach a single event listener to the application's mount point.

So, say we have a delegated event `onclick`. If you create 50 DOM nodes with `onclick` handlers,
only one `onclick` handler will be in the DOM.

This event listener handles all events and handles bubbling and `.stop_propagation()`.

## Per-node Events

For per-node events we attach the event to the DOM node.

So, say `onfoo` is a per-node event. If you create 50 DOM nodes with 50 `onfoo` handlers,
there will be 50 `onfoo` callbacks in the DOM (one per node).
