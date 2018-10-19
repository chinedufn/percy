const path = require('path')

const mode =
  process.env.NODE_ENV === 'production' ? 'production' : 'development'

module.exports = {
  mode,
  entry: './src',
  output: {
    path: path.resolve(__dirname, 'build'),
    filename: 'bundle.js'
  },
  devServer: {
    proxy: { '/': { target: 'http://localhost:7878' } }
  }
}
