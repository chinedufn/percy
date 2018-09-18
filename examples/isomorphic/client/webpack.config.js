const path = require('path')

const mode =
  process.env.NODE_ENV === 'production' ? 'production' : 'development'
module.exports = {
  mode,
  entry: './src', // webpack default
  output: {
    path: path.resolve(__dirname, 'dist'), // webpack default
    filename: 'bundle.js' // webpack default
  },
  devServer: {
    proxy: { '/': { target: 'http://localhost:7878' } }
  }
}
