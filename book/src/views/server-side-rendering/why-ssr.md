# Why use Server Side Rendering

In recent years it has become popular for just about all of a web application to be rendered on the client.

Applications will often serve almost nothing but a `<script>` tag that loads up some front-end code (`JavaScript` and/or `WebAssembly`)
and that front-end code is responsible for rendering the application's `HTML` and interactions.

Here's an example of what many of today's web application boil down to:

```html
<!DOCTYPE html>
<html lang="en">
<body>
  <div id='app'>
    <!-- application will render HTML here when it begins -->
  </div>
  <!--
    One this applications loads it will
    inject some HTML into the body
  -->
  <script src="/my-application.js"></script>
</body>
</html>

```

---

One downside to this approach is that a user must wait until the script begins rendering before seeing anything.

Let's illustrate:

```
Client side rendering
without server side rendering:

┌─────────────────────────────────────┐
│   1) Client requests for web page   │
└─────────────────────────────────────┘
                   │
                   ▼
┌─────────────────────────────────────┐
│2) Server responds with <script> tag │
└─────────────────────────────────────┘
                   │
                   ▼
┌─────────────────────────────────────┐
│     3) Client downloads script      │
└─────────────────────────────────────┘
                   │
                   ▼
┌─────────────────────────────────────┐
│4) Client parses the returned script │
└─────────────────────────────────────┘
                   │
                   ▼
┌─────────────────────────────────────┐
│ 5) Client executes returned script  │
└─────────────────────────────────────┘
                   │
                   ▼
┌─────────────────────────────────────┐  User first
│ 6) Script starts rendering content  │◀─   sees
└─────────────────────────────────────┘   content
```

Contrast this with server side rendering, where the initial page load might look something like this:

```html
<!DOCTYPE html>
<html lang="en">
<body>
  <div id='app'>
  <!--
    This content was sent down from the server so
    that the user sees something immediately!
  -->
  </div>

  <script src="/my-application.js"></script>
</body>
</html>

```

And the flow:

```
Server side rendering then client
takes over rendering:

┌─────────────────────────────────────┐
│   1) Client requests for web page   │
└─────────────────────────────────────┘
                   │
                   ▼
┌─────────────────────────────────────┐
│   2) Server responds with server    │  User first
│ side rendered content along with a  │◀─   sees
│            <script> tag             │   content
└─────────────────────────────────────┘
                   │
                   ▼
┌─────────────────────────────────────┐
│     3) Client downloads script      │
└─────────────────────────────────────┘
                   │
                   ▼
┌─────────────────────────────────────┐
│4) Client parses the returned script │
└─────────────────────────────────────┘
                   │
                   ▼
┌─────────────────────────────────────┐
│ 5) Client executes returned script  │
└─────────────────────────────────────┘
                   │
                   ▼
┌─────────────────────────────────────┐
│ 6) Script starts rendering content  │
└─────────────────────────────────────┘
```

Server side rendering allows you to some *something* to your users more quickly,
especially so for users with slower machines and/or bandwidth.
