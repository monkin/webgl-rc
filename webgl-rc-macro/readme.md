# GLSL Loader for `webgl-rc`

## Shader file

```glsl
/* include absolute path 'project_dir/glsl/lib/color.glsl'  */
#include <lib/color.glsl>

/* include relative path './common/bezier.glsl' */
#include "./common/bezier.glsl"

void main() {
    ...
}
```

## Rust file

```rust
use webgl_rc::load_glsl;

const fragment_source: &str = load_glsl!("fragment.glsl");

```