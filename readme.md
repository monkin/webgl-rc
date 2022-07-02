# Rust WebGL Wrapper

Every resource in `webgl-rc` contains a reference counter.
So, cloning texture or buffer will create a new reference to the same resource.

## Example

Source: https://github.com/monkin/webgl-rc/tree/master/webgl-rc-example

Live: https://monkin.github.io/webgl-rc/webgl-rc-example/dist/index.html