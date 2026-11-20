// Copyright (c) 2026 Polyglotic. All rights reserved.
//
// M0's pointer ink: a stroke's ribbon of triangles, drawn straight to the
// window with premultiplied alpha. No simulation (the plan's section 5:
// pointer ink is deposit and look only).
//
// Vertices arrive in window pixels; `view.size` is the window's size.

struct View {
  size: vec2<f32>,
  _pad: vec2<f32>,
  colour: vec4<f32>,
}
@group(0) @binding(0) var<uniform> view: View;

struct Out {
  @builtin(position) pos: vec4<f32>,
  @location(0) ink: f32,
}

@vertex
fn vs(@location(0) at: vec3<f32>) -> Out {
  var o: Out;
  let ndc = vec2(at.x / view.size.x * 2.0 - 1.0, 1.0 - at.y / view.size.y * 2.0);
  o.pos = vec4(ndc, 0.0, 1.0);
  o.ink = at.z;
  return o;
}

@fragment
fn fs(i: Out) -> @location(0) vec4<f32> {
  let a = view.colour.a * i.ink;
  return vec4(view.colour.rgb * a, a);
}
