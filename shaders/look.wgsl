// Copyright 2026 Polyglotic
// SPDX-License-Identifier: Apache-2.0
//
// The look pass: the picture of the ink on the paper, lit from a height field
// (Hertzmann 2002; Baxter et al., "IMPaSTo", 2004).
//
// Height = the paper's grain + fixed pigment + the wet film. The normal comes
// from the height's central differences; the light is fixed. Diffuse on
// everything, a specular glint only where the film is wet. M0 mixes ink over
// paper by Beer-Lambert darkening; the spectral mixing (spectral.js, ported)
// arrives with colour.

@group(0) @binding(0) var pig: texture_2d<f32>;
@group(0) @binding(1) var paper: texture_2d<f32>;
@group(0) @binding(2) var s2: texture_2d<f32>;
@group(0) @binding(3) var outp: texture_storage_2d<rgba8unorm, write>;

fn h(p: vec2<i32>) -> f32 {
  return textureLoad(paper, p, 0).g * 0.5 + textureLoad(pig, p, 0).b * 2.0
    + textureLoad(s2, p, 0).y * 0.2;
}

@compute @workgroup_size(8, 8)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
  let dim = vec2<i32>(textureDimensions(pig));
  let p = vec2<i32>(gid.xy);
  if (p.x >= dim.x || p.y >= dim.y) { return; }
  let l = clamp(p - vec2(1, 0), vec2(0), dim - vec2(1));
  let r = clamp(p + vec2(1, 0), vec2(0), dim - vec2(1));
  let d = clamp(p - vec2(0, 1), vec2(0), dim - vec2(1));
  let t = clamp(p + vec2(0, 1), vec2(0), dim - vec2(1));
  let n = normalize(vec3(h(l) - h(r), h(d) - h(t), 0.05));
  let light = normalize(vec3(-0.4, 0.5, 0.75));
  let diff = max(dot(n, light), 0.0);
  let wet = textureLoad(s2, p, 0).y;
  let spec = pow(max(dot(reflect(-light, n), vec3(0.0, 0.0, 1.0)), 0.0), 40.0) * wet;
  let c = textureLoad(pig, p, 0);
  let ink = 1.0 - exp(-3.0 * (c.r + c.b));
  let col = mix(vec3(0.96, 0.94, 0.88), vec3(0.05, 0.05, 0.07), ink) * (0.35 + 0.65 * diff) + spec;
  textureStore(outp, p, vec4(col, 1.0));
}
