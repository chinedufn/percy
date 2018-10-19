const path = require('path')

const mode =
  process.env.NODE_ENV === 'production' ? 'production' : 'development'

const outputDir = mode === 'production' ? 'dist' : 'build'

module.exports = {
  mode,
  entry: './src',
  output: {
    path: path.resolve(__dirname, outputDir),
    filename: 'bundle.js'
  },
  devServer: {
    proxy: { '/': { target: 'http://localhost:7878' } }
  }
}
