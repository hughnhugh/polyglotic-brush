// Copyright (c) 2026 Polyglotic. All rights reserved.
//
// The flow pass: water and pigment in the paper, one fixed step, over the
// WHOLE canvas. Never a region that follows the brush (docs/decisions/0004).
//
// The maths (Chu and Tai, "MoXi", 2005; Curtis et al. 1997):
// - D2Q9 lattice Boltzmann, pull streaming: f_i(x) takes f_i(x - e_i).
// - Partial bounce-back: the paper's fibres (paper.r, 0 to 1) send a share of
//   each incoming population back the way it came, so dense paper slows flow.
// - BGK collision towards the equilibrium
//   feq_i = w_i rho (1 + 3 e_i.u + 4.5 (e_i.u)^2 - 1.5 u.u), rate omega.
// - A little evaporation each step.
// - Pigment: rg is carried by the water (moved back along u, semi-Lagrangian),
//   ba is fixed in the paper; a share fixes each step, more where the paper is
//   rough (paper.g).
//
// State: nine populations in s0 (0-3), s1 (4-7), s2.x (8); s2.yzw keeps
// density and velocity for the look. Half-float textures throughout.

@group(0) @binding(0) var s0: texture_2d<f32>;
@group(0) @binding(1) var s1: texture_2d<f32>;
@group(0) @binding(2) var s2: texture_2d<f32>;
@group(0) @binding(3) var pig: texture_2d<f32>;
@group(0) @binding(4) var paper: texture_2d<f32>;
@group(0) @binding(5) var d0: texture_storage_2d<rgba16float, write>;
@group(0) @binding(6) var d1: texture_storage_2d<rgba16float, write>;
@group(0) @binding(7) var d2: texture_storage_2d<rgba16float, write>;
@group(0) @binding(8) var dpig: texture_storage_2d<rgba16float, write>;
@group(0) @binding(9) var lin: sampler;

const E = array<vec2<i32>, 9>(
  vec2(0, 0), vec2(1, 0), vec2(0, 1), vec2(-1, 0), vec2(0, -1),
  vec2(1, 1), vec2(-1, 1), vec2(-1, -1), vec2(1, -1));
const W = array<f32, 9>(4.0 / 9.0, 1.0 / 9.0, 1.0 / 9.0, 1.0 / 9.0, 1.0 / 9.0,
  1.0 / 36.0, 1.0 / 36.0, 1.0 / 36.0, 1.0 / 36.0);
const OPP = array<i32, 9>(0, 3, 4, 1, 2, 7, 8, 5, 6);

fn load(i: i32, p: vec2<i32>) -> f32 {
  if (i < 4) { return textureLoad(s0, p, 0)[i]; }
  if (i < 8) { return textureLoad(s1, p, 0)[i - 4]; }
  return textureLoad(s2, p, 0).x;
}

@compute @workgroup_size(8, 8)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
  let dim = vec2<i32>(textureDimensions(s0));
  let p = vec2<i32>(gid.xy);
  if (p.x >= dim.x || p.y >= dim.y) { return; }
  let here = textureLoad(paper, p, 0);
  var e = E;
  var w = W;
  var opp = OPP;
  var f: array<f32, 9>;
  var rho = 0.0;
  var u = vec2<f32>(0.0);
  for (var i = 0; i < 9; i++) {
    let q = clamp(p - e[i], vec2(0), dim - vec2(1));
    let block = 0.5 * (here.r + textureLoad(paper, q, 0).r);
    let v = mix(load(i, q), load(opp[i], p), block);
    f[i] = v;
    rho += v;
    u += v * vec2<f32>(e[i]);
  }
  u = u / max(rho, 1e-4);
  let usq = dot(u, u);
  let omega = 1.2;
  var o: array<f32, 9>;
  for (var i = 0; i < 9; i++) {
    let eu = dot(vec2<f32>(e[i]), u);
    let feq = w[i] * rho * (1.0 + 3.0 * eu + 4.5 * eu * eu - 1.5 * usq);
    o[i] = mix(f[i], feq, omega) * 0.9995;
  }
  textureStore(d0, p, vec4(o[0], o[1], o[2], o[3]));
  textureStore(d1, p, vec4(o[4], o[5], o[6], o[7]));
  textureStore(d2, p, vec4(o[8], rho, u.x, u.y));
  let uv = (vec2<f32>(p) + 0.5 - u) / vec2<f32>(dim);
  let c = textureSampleLevel(pig, lin, uv, 0.0);
  let fix = c.rg * 0.01 * (1.0 + 4.0 * here.g);
  textureStore(dpig, p, vec4(c.rg - fix, c.ba + fix));
}
