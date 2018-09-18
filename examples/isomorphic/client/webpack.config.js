const mode =
  process.env.NODE_ENV === 'production' ? 'production' : 'development'
module.exports = {
  mode,
  devServer: {
    proxy: { '/': { target: 'http://localhost:7878' } }
  }
}
