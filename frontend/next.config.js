const path = require('node:path');

/** @type {import('next').NextConfig} */
const nextConfig = {
  images: {
    unoptimized: true
  },
  output: 'export',
  turbopack: {
    root: path.resolve(__dirname)
  },
  trailingSlash: true
};

module.exports = nextConfig;
