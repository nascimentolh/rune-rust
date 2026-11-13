//! A tile's triangles, as `Class329` cuts them from its floors. With ground blending off
//! (`method5845`) the overlay and underlay split the tile by its shape; with it on (`method5846`,
//! the default) blending overlays also reach into the tiles around them, sides where overlays
//! meet get triangles of their own, and underlay points take the blended colours at the tile's
//! corners. Textures and the second colours (`secondaryRGB`, `VarNPCMap.method2617`) aren't used
//! yet: the ground is drawn in the primary colours.

use super::ground::SCENE;
use crate::cache::floors::{Overlay, Underlay};

/// `OVERLAY_FACE_COUNT`: overlay triangles per shape, no edges.
pub const OVERLAY_FACES: [i32; 15] = [2, 1, 1, 1, 2, 2, 2, 1, 3, 3, 3, 2, 0, 4, 0];
/// `UNDERLAY_FACE_COUNT`: underlay triangles per shape, no edges.
pub const UNDERLAY_FACES: [i32; 15] = [0, 1, 2, 2, 1, 1, 2, 3, 1, 3, 3, 4, 2, 0, 4];
/// `anIntArray3826`: overlay triangles per shape with edges.
pub const EDGE_OVERLAY_FACES: [i32; 13] = [4, 2, 1, 1, 2, 2, 3, 1, 3, 3, 3, 2, 0];
/// `anIntArray3847`: underlay triangles per shape with edges.
pub const EDGE_UNDERLAY_FACES: [i32; 13] = [0, 2, 2, 2, 1, 1, 3, 3, 1, 3, 3, 4, 4];
/// `anIntArray3778`: overlay triangles per shape over a blending overlay.
pub const BLEND_OVERLAY_FACES: [i32; 13] = [4, 2, 1, 1, 2, 2, 3, 1, 3, 3, 3, 2, 0];
/// `anIntArray3819`: underlay triangles per shape over a blending overlay.
pub const BLEND_UNDERLAY_FACES: [i32; 13] = [0, 4, 3, 3, 1, 1, 3, 5, 1, 5, 3, 6, 4];
/// `aBoolArrayArray3822`: which of the 13 points each shape's overlay covers.
pub const COVERS: [[bool; 13]; 15] = [
    [true, true, true, true, true, true, true, true, true, true, true, true, true],
    [true, true, true, false, false, false, true, true, false, false, false, false, true],
    [true, false, false, false, false, true, true, true, false, false, false, false, false],
    [false, false, true, true, true, true, false, false, false, false, false, false, false],
    [true, true, true, true, true, true, false, false, false, false, false, false, false],
    [true, true, true, false, false, true, true, true, false, false, false, false, false],
    [true, true, false, false, false, true, true, true, false, false, false, false, true],
    [true, true, false, false, false, false, false, true, false, false, false, false, false],
    [false, true, true, true, true, true, true, true, false, false, false, false, false],
    [true, false, false, false, true, true, true, true, true, true, false, false, false],
    [true, true, true, true, true, false, false, false, true, true, false, false, false],
    [true, true, true, false, false, false, false, false, false, false, true, true, false],
    [false, false, false, false, false, false, false, false, false, false, false, false, false],
    [true, true, true, true, true, true, true, true, true, true, true, true, true],
    [false, false, false, false, false, false, false, false, false, false, false, false, false],
];
/// `anIntArrayArray3824`: first point of each triangle, no edges.
pub const FACE_A: [&[i32]; 15] = [
    &[0, 2],
    &[0, 2],
    &[0, 0, 2],
    &[2, 0, 0],
    &[0, 2, 0],
    &[0, 0, 2],
    &[0, 5, 1, 4],
    &[0, 4, 4, 4],
    &[4, 4, 4, 0],
    &[6, 6, 6, 2, 2, 2],
    &[2, 2, 2, 6, 6, 6],
    &[0, 11, 6, 6, 6, 4],
    &[0, 2],
    &[0, 4, 4, 4],
    &[0, 4, 4, 4],
];
/// `anIntArrayArray3860`: second point, no edges.
pub const FACE_B: [&[i32]; 15] = [
    &[2, 4],
    &[2, 4],
    &[5, 2, 4],
    &[4, 5, 2],
    &[2, 4, 5],
    &[5, 2, 4],
    &[1, 6, 2, 5],
    &[1, 6, 7, 1],
    &[6, 7, 1, 1],
    &[0, 8, 9, 8, 9, 4],
    &[8, 9, 4, 0, 8, 9],
    &[2, 10, 0, 10, 11, 11],
    &[2, 4],
    &[1, 6, 7, 1],
    &[1, 6, 7, 1],
];
/// `anIntArrayArray3815`: third point, no edges.
pub const FACE_C: [&[i32]; 15] = [
    &[6, 6],
    &[6, 6],
    &[6, 5, 5],
    &[5, 6, 5],
    &[5, 5, 6],
    &[6, 5, 5],
    &[5, 0, 4, 1],
    &[7, 7, 1, 2],
    &[7, 1, 2, 7],
    &[8, 9, 4, 0, 8, 9],
    &[0, 8, 9, 8, 9, 4],
    &[11, 0, 10, 11, 4, 2],
    &[6, 6],
    &[7, 7, 1, 2],
    &[7, 7, 1, 2],
];
/// `aBoolArrayArray3816`: which sides each shape's overlay fills.
pub const SIDES: [[bool; 4]; 13] = [
    [false, false, false, false],
    [false, false, false, false],
    [false, false, true, false],
    [false, false, true, false],
    [false, false, true, false],
    [false, false, true, false],
    [true, false, true, false],
    [true, false, false, true],
    [true, false, false, true],
    [false, false, false, false],
    [false, false, false, false],
    [false, false, false, false],
    [false, false, false, false],
];
/// `anIntArrayArray3828`: the triangle split at each side (-1 none), with edges.
pub const EDGE_SPLIT: [&[i32]; 13] = [
    &[0, 1, 2, 3],
    &[1, 2, 3, 0],
    &[1, 2, -1, 0],
    &[2, 0, -1, 1],
    &[0, 1, -1, 2],
    &[1, 2, -1, 0],
    &[-1, 4, -1, 1],
    &[-1, 1, 3, -1],
    &[-1, 0, 2, -1],
    &[3, 5, 2, 0],
    &[0, 2, 5, 3],
    &[0, 2, 3, 5],
    &[0, 1, 2, 3],
];
/// `anIntArrayArray3774`: first point of each triangle, with edges.
pub const EDGE_A: [&[i32]; 13] = [
    &[0, 2, 4, 6],
    &[6, 0, 2, 4],
    &[6, 0, 2],
    &[2, 6, 0],
    &[0, 2, 6],
    &[6, 0, 2],
    &[5, 6, 0, 1, 2, 4],
    &[7, 2, 4, 4],
    &[2, 4, 4, 7],
    &[6, 6, 4, 0, 2, 2],
    &[0, 2, 2, 6, 6, 4],
    &[0, 2, 2, 4, 6, 6],
    &[0, 2, 4, 6],
];
/// `anIntArrayArray3830`: second point, with edges.
pub const EDGE_B: [&[i32]; 13] = [
    &[2, 4, 6, 0],
    &[0, 2, 4, 6],
    &[0, 2, 4],
    &[4, 0, 2],
    &[2, 4, 0],
    &[0, 2, 4],
    &[6, 0, 1, 2, 4, 5],
    &[0, 4, 7, 6],
    &[4, 7, 6, 0],
    &[0, 8, 6, 2, 9, 4],
    &[2, 9, 4, 0, 8, 6],
    &[2, 11, 4, 6, 10, 0],
    &[2, 4, 6, 0],
];
/// `anIntArrayArray3831`: third point, with edges.
pub const EDGE_C: [&[i32]; 13] = [
    &[12, 12, 12, 12],
    &[12, 12, 12, 12],
    &[5, 5, 5],
    &[5, 5, 5],
    &[5, 5, 5],
    &[5, 5, 5],
    &[12, 12, 12, 12, 12, 12],
    &[1, 1, 1, 7],
    &[1, 1, 7, 1],
    &[8, 9, 9, 8, 8, 9],
    &[8, 8, 9, 8, 9, 9],
    &[10, 10, 11, 11, 11, 10],
    &[12, 12, 12, 12],
];
/// `aBoolArrayArray3793`: which sides each shape's blending overlay fills.
pub const BLEND_SIDES: [[bool; 4]; 13] = [
    [false, false, false, false],
    [false, true, true, false],
    [true, false, true, false],
    [true, false, true, false],
    [false, false, true, false],
    [false, false, true, false],
    [true, false, true, false],
    [true, false, false, true],
    [true, false, false, true],
    [true, true, false, false],
    [false, false, false, false],
    [false, true, false, true],
    [false, false, false, false],
];
/// `anIntArrayArray3833`: the triangle split at each side, over a blending overlay.
pub const BLEND_SPLIT: [&[i32]; 13] = [
    &[0, 1, 2, 3],
    &[1, -1, -1, 0],
    &[-1, 2, -1, 0],
    &[-1, 0, -1, 2],
    &[0, 1, -1, 2],
    &[1, 2, -1, 0],
    &[-1, 4, -1, 1],
    &[-1, 3, 4, -1],
    &[-1, 0, 2, -1],
    &[-1, -1, 2, 0],
    &[0, 2, 5, 3],
    &[0, -1, 6, -1],
    &[0, 1, 2, 3],
];
/// `anIntArrayArray3775`: first point, over a blending overlay.
pub const BLEND_A: [&[i32]; 13] = [
    &[0, 2, 4, 6],
    &[6, 0, 2, 3, 5, 3],
    &[6, 0, 2, 4],
    &[2, 5, 6, 1],
    &[0, 2, 6],
    &[6, 0, 2],
    &[5, 6, 0, 1, 2, 4],
    &[7, 7, 1, 2, 4, 6],
    &[2, 4, 4, 7],
    &[6, 6, 4, 0, 1, 1, 3, 3],
    &[0, 2, 2, 6, 6, 4],
    &[0, 2, 2, 3, 7, 0, 4, 3],
    &[0, 2, 4, 6],
];
/// `anIntArrayArray3821`: second point, over a blending overlay.
pub const BLEND_B: [&[i32]; 13] = [
    &[2, 4, 6, 0],
    &[0, 2, 3, 5, 6, 4],
    &[0, 1, 4, 5],
    &[4, 6, 0, 2],
    &[2, 4, 0],
    &[0, 2, 4],
    &[6, 0, 1, 2, 4, 5],
    &[0, 1, 2, 4, 6, 7],
    &[4, 7, 6, 0],
    &[0, 8, 6, 1, 9, 2, 9, 4],
    &[2, 9, 4, 0, 8, 6],
    &[2, 11, 3, 7, 10, 10, 6, 6],
    &[2, 4, 6, 0],
];
/// `anIntArrayArray3836`: third point, over a blending overlay.
pub const BLEND_C: [&[i32]; 13] = [
    &[12, 12, 12, 12],
    &[12, 12, 12, 12, 12, 5],
    &[5, 5, 1, 1],
    &[5, 1, 1, 5],
    &[5, 5, 5],
    &[5, 5, 5],
    &[12, 12, 12, 12, 12, 12],
    &[1, 12, 12, 12, 12, 12],
    &[1, 1, 7, 1],
    &[8, 9, 9, 8, 8, 3, 1, 9],
    &[8, 8, 9, 8, 9, 9],
    &[10, 10, 11, 11, 11, 7, 3, 7],
    &[12, 12, 12, 12],
];

/// `anIntArray3811` / `anIntArray3800`: the 13 points a tile's shapes are cut from (x, y inside
/// the tile, 512 across): corners 0, 2, 4, 6, side middles 1, 3, 5, 7, inner points 8..=12.
const POINT_X: [i32; 13] = [0, 256, 512, 512, 512, 256, 0, 0, 128, 256, 128, 384, 256];
const POINT_Y: [i32; 13] = [0, 0, 0, 256, 512, 512, 512, 256, 256, 384, 128, 128, 256];

/// A corner of a triangle: x, y inside the tile (0..=512) and its packed HSL colour (-1: none).
pub type Point = (i32, i32, i32);
/// A triangle of a tile.
pub type Face = [Point; 3];

/// Point `p` of the 13 turned `rotation` quarters (1: (y, 512 − x), 2: (512 − x, 512 − y),
/// 3: (512 − y, x)).
fn turn(p: usize, rotation: i32) -> (i32, i32) {
    let (x, y) = (POINT_X[p], POINT_Y[p]);
    match rotation {
        1 => (y, 512 - x),
        2 => (512 - x, 512 - y),
        3 => (512 - y, x),
        _ => (x, y),
    }
}

/// Groups points into triangles.
fn triangles(points: Vec<Point>) -> Vec<Face> {
    points.as_chunks::<3>().0.to_vec()
}

/// `Class329.method5845` (ground blending off) for one tile of shape `shape` (12: all underlay)
/// turned `rotation` quarters: the overlay's triangles in its colour, then the underlay's in the
/// blended one.
pub fn plain(shape: u8, rotation: u8, overlay: Option<i32>, underlay: Option<i32>) -> Vec<Face> {
    let s = shape as usize;
    let face = |k: usize, colour: i32| {
        [FACE_A[s][k], FACE_B[s][k], FACE_C[s][k]].map(|p| {
            let (x, y) = turn(p as usize, rotation as i32);
            (x, y, colour)
        })
    };
    let over = OVERLAY_FACES[s] as usize;
    let mut out = Vec::new();
    if let Some(colour) = overlay {
        out.extend((0..over).map(|k| face(k, colour)));
    }
    if let Some(colour) = underlay {
        out.extend((over..over + UNDERLAY_FACES[s] as usize).map(|k| face(k, colour)));
    }
    out
}

/// The scene's floors, `[x × SCENE + y]`, for `method5846`.
pub struct Floors<'a> {
    /// (underlay id, overlay id, shape, rotation) per tile, ids 0 for none.
    pub tiles: &'a [(u8, u8, u8, u8)],
    /// Each tile's overlay definition.
    pub overlays: &'a [Option<Overlay>],
    /// Each tile's underlay definition.
    pub underlays: &'a [Option<Underlay>],
    /// `ints_4`: each tile's blended underlay colour (0 where the blend wrote nothing).
    pub blended: &'a [i32],
    /// The ground's corner heights, (SCENE + 1)².
    pub heights: &'a [i32],
}

/// `method5849`'s pick of a tile's cut: each triangle's three points, the triangle split at each
/// side with an edge (-1 none), and how many triangles the overlay and underlay get.
struct Cut {
    a: &'static [i32],
    b: &'static [i32],
    c: &'static [i32],
    split: &'static [i32],
    overs: i32,
    unders: i32,
}

/// What the overlays around a tile leave at its 8 edge points (`anIntArray3838` colour,
/// `anIntArray3842` priority, `anIntArray3843` which overlays reached it).
struct Around {
    colour: [i32; 8],
    priority: [i32; 8],
    mask: [i32; 8],
}

impl Around {
    /// Overlay `o` takes edge point `slot` unless one ranked higher has it; `bit` marks it, kept
    /// alongside the bits of overlays of the same priority.
    fn take(&mut self, slot: usize, o: &Overlay, bit: i32) {
        if self.priority[slot] > o.priority {
            return;
        }
        self.colour[slot] = o.colour;
        self.mask[slot] = if o.priority == self.priority[slot] { self.mask[slot] | bit } else { bit };
        self.priority[slot] = o.priority;
    }
}

impl Floors<'_> {
    fn at(x: usize, y: usize) -> usize {
        x * SCENE + y
    }

    /// `Class329.method5846` (ground blending on) for tile (`x`, `y`): blending overlays reach
    /// into the tiles around them, sides where the overlay changes get their own triangles, and
    /// underlay points take the blended colours of the tiles at each corner.
    pub fn blended(&self, x: usize, y: usize) -> Vec<Face> {
        let n = SCENE;
        let (x1, y1) = (if x < n - 1 { x + 1 } else { x }, if y < n - 1 { y + 1 } else { y });
        let (under_id, over_id, raw_shape, raw_rotation) = self.tiles[Self::at(x, y)];
        if over_id == 0 && under_id == 0 {
            return Vec::new();
        }
        let mut overlay = self.overlays[Self::at(x, y)].as_ref();
        let underlay = self.underlays[Self::at(x, y)].as_ref();
        let mut shape = raw_shape as usize;
        if shape == 0 && overlay.is_none() {
            shape = 12;
        }
        // `aBool3810`: a blending overlay over an underlay.
        let mut over_blend = false;
        if let Some(o) = overlay {
            if o.colour == -1 && o.secondary == -1 {
                overlay = None;
            } else if underlay.is_some() && shape != 0 {
                over_blend = o.blends;
            }
        }
        let mut rotation = self.diagonal(under_id, x, y, shape, raw_rotation as i32);
        let mut around = Around { colour: [0; 8], priority: [-1; 8], mask: [1; 8] };
        let mut edges = [false; 4];
        // `method5859`.
        let sides = if overlay.is_some_and(|o| o.blends) { BLEND_SIDES[shape] } else { SIDES[shape] };
        self.neighbours(overlay, x, y, shape, rotation, &sides, &mut around, &mut edges);
        let open = |k: i32| !sides[((rotation + k) & 3) as usize];
        if open(1) {
            edges[1] |= around.mask[2] & around.mask[4] == 0;
        }
        if open(3) {
            edges[3] |= around.mask[6] & around.mask[0] == 0;
        }
        if open(0) {
            edges[0] |= around.mask[0] & around.mask[2] == 0;
        }
        if open(2) {
            edges[2] |= around.mask[4] & around.mask[6] == 0;
        }
        if !over_blend && (shape == 0 || shape == 12) {
            let turned = match edges {
                [true, false, false, true] => Some(0),
                [true, true, false, false] => Some(3),
                [false, true, true, false] => Some(2),
                [false, false, true, true] => Some(1),
                _ => None,
            };
            if let Some(r) = turned {
                edges = [false; 4];
                shape = if shape == 0 { 13 } else { 14 };
                rotation = r;
            }
        }
        // `method5849`: which cut of the tile.
        let plain = !over_blend && edges == [false; 4];
        let cut = if plain {
            Cut {
                a: FACE_A[shape],
                b: FACE_B[shape],
                c: FACE_C[shape],
                split: &[],
                overs: OVERLAY_FACES[shape],
                unders: UNDERLAY_FACES[shape],
            }
        } else if over_blend {
            Cut {
                a: BLEND_A[shape],
                b: BLEND_B[shape],
                c: BLEND_C[shape],
                split: BLEND_SPLIT[shape],
                overs: BLEND_OVERLAY_FACES[shape],
                unders: BLEND_UNDERLAY_FACES[shape],
            }
        } else {
            Cut {
                a: EDGE_A[shape],
                b: EDGE_B[shape],
                c: EDGE_C[shape],
                split: EDGE_SPLIT[shape],
                overs: EDGE_OVERLAY_FACES[shape],
                unders: EDGE_UNDERLAY_FACES[shape],
            }
        };
        let Cut { a, b, c, split, overs, unders } = cut;
        if overlay.map_or(0, |_| overs) + underlay.map_or(0, |_| unders) <= 0 {
            return Vec::new();
        }
        // The points of triangle `k`: split in two at a side with an edge.
        let corners = |k: usize| -> Vec<usize> {
            let at_side = |side: i32, nth: usize| edges[((side - rotation) & 3) as usize] && split.get(nth) == Some(&(k as i32));
            let middle = [(0, 0, 1), (2, 2, 5), (1, 1, 3), (3, 3, 7)].into_iter().find(|&(side, nth, _)| at_side(side, nth));
            let (pa, pb, pc) = (a[k] as usize, b[k] as usize, c[k] as usize);
            match middle {
                Some((_, _, m)) => vec![pa, m, pc, m, pb, pc],
                None => vec![pa, pb, pc],
            }
        };
        let mut points = Vec::new();
        let mut k = 0;
        // `method5850`: the overlay's triangles; edge points another overlay ranks above take
        // its colour.
        let over_colour = overlay.map_or(-1, |o| o.colour);
        match overlay {
            Some(o) => {
                for _ in 0..overs {
                    for p in corners(k) {
                        let slot = ((p as i32 - rotation * 2) & 7) as usize;
                        let colour = if p < 8 && around.priority[slot] > o.priority { around.colour[slot] } else { o.colour };
                        let (px, py) = turn(p, rotation);
                        points.push((px, py, colour));
                    }
                    k += 1;
                }
            }
            None => k += overs as usize,
        }
        // `method5851`: the underlay's triangles, coloured from the overlays around, the
        // blending overlay, or the blended colours at the tile's corners.
        if underlay.is_some() {
            let bl = |tx: usize, ty: usize| self.blended[Self::at(tx, ty)];
            for _ in 0..unders {
                for p in corners(k) {
                    let slot = ((p as i32 - rotation * 2) & 7) as usize;
                    let (px, py) = turn(p, rotation);
                    let colour = if p < 8 && around.priority[slot] >= 0 {
                        around.colour[slot]
                    } else if over_blend && COVERS[shape][p] {
                        over_colour
                    } else {
                        match (px, py) {
                            (0, 0) => bl(x, y),
                            (0, 512) => bl(x, y1),
                            (512, 512) => bl(x1, y1),
                            (512, 0) => bl(x1, y),
                            _ => {
                                let south = mix(bl(x, y), bl(x1, y), (px << 7) >> 9);
                                let north = mix(bl(x, y1), bl(x1, y1), (px << 7) >> 9);
                                mix(south, north, (py << 7) >> 9)
                            }
                        }
                    };
                    points.push((px, py, colour));
                }
                k += 1;
            }
        }
        triangles(points)
    }

    /// `Class329.method5870`: a whole-underlay tile (shape 0 or 12) inside the scene picks the
    /// diagonal its two triangles split along, by which corners have the most of the same
    /// underlay around them (then by the flatter diagonal); others keep `rotation`.
    fn diagonal(&self, id: u8, x: usize, y: usize, shape: usize, rotation: i32) -> i32 {
        if !((shape == 0 || shape == 12) && x > 0 && y > 0 && x < SCENE && y < SCENE) {
            return rotation;
        }
        let (x1, y1) = (if x < SCENE - 1 { x + 1 } else { x }, if y < SCENE - 1 { y + 1 } else { y });
        // The map keeps ids as signed bytes, so ids above 127 never match here.
        let same = |tx: usize, ty: usize| -> i32 {
            if (self.tiles[Self::at(tx, ty)].0 as i8) as i32 == id as i32 {
                1
            } else {
                -1
            }
        };
        let mut sw = same(x - 1, y - 1);
        let mut se = same(x1, y - 1);
        let mut ne = same(x1, y1);
        let mut nw = same(x - 1, y1);
        let s = same(x, y - 1);
        (sw, se) = (sw + s, se + s);
        let e = same(x1, y);
        (se, ne) = (se + e, ne + e);
        let nn = same(x, y1);
        (ne, nw) = (ne + nn, nw + nn);
        let w = same(x - 1, y);
        (nw, sw) = (nw + w, sw + w);
        let mut one = (sw - ne).abs();
        let mut two = (se - nw).abs();
        if one == two {
            let h = |cx: usize, cy: usize| self.heights[cx * (SCENE + 1) + cy];
            one = (h(x, y) - h(x1, y1)).abs();
            two = (h(x1, y) - h(x, y1)).abs();
        }
        if one < two {
            1
        } else {
            0
        }
    }

    /// `Class329.method5848`: the overlays on the 8 tiles around (and this tile's own, if it
    /// blends) claim this tile's edge points, and mark the sides where a different overlay meets.
    #[allow(clippy::too_many_arguments)]
    fn neighbours(
        &self,
        overlay: Option<&Overlay>,
        x: usize,
        y: usize,
        shape: usize,
        rotation: i32,
        sides: &[bool; 4],
        around: &mut Around,
        edges: &mut [bool; 4],
    ) {
        let n = SCENE;
        // A tile's overlay with a colour, its shape and rotation.
        let tile = |tx: usize, ty: usize| {
            let (_, id, s, r) = self.tiles[Self::at(tx, ty)];
            if id == 0 {
                return None;
            }
            self.overlays[Self::at(tx, ty)].as_ref().filter(|o| o.colour != -1).map(|o| (o, s as usize, r as i32))
        };
        // Diagonal neighbours: a blending overlay covering the touching corner takes it.
        let mut diagonal = |tx: usize, ty: usize, add: i32, slot: usize, mask: i32| {
            if let Some((o, s, r)) = tile(tx, ty).filter(|(o, _, _)| o.blends) {
                if COVERS[s][((r * 2 + add) & 7) as usize] {
                    around.colour[slot] = o.colour;
                    around.priority[slot] = o.priority;
                    around.mask[slot] = mask;
                }
            }
        };
        if y > 0 {
            if x > 0 {
                diagonal(x - 1, y - 1, 4, 0, 256);
            }
            if x < n - 1 {
                diagonal(x + 1, y - 1, 6, 2, 512);
            }
        }
        if y < n - 1 {
            if x > 0 {
                diagonal(x - 1, y + 1, 2, 6, 64);
            }
            if x < n - 1 {
                diagonal(x + 1, y + 1, 0, 4, 128);
            }
        }
        // Side neighbours: a blending overlay takes the 3 points of the shared side it covers
        // and outranks; any overlay marks the side if this tile's shape leaves it open.
        let mut side =
            |tx: usize, ty: usize, slot: i32, slot_step: i32, add: i32, add_step: i32, bit: i32, edge: usize, open: i32, turn: i32| {
                let Some((o, s, r)) = tile(tx, ty) else { return };
                let (mut slot, mut point) = (slot, r * 2 + add);
                if o.blends {
                    for _ in 0..3 {
                        (slot, point) = (slot & 7, point & 7);
                        if COVERS[s][point as usize] {
                            around.take(slot as usize, o, bit);
                        }
                        (slot, point) = (slot + slot_step, point + add_step);
                    }
                }
                if !sides[((rotation + open) & 3) as usize] {
                    let table = if o.blends { &BLEND_SIDES } else { &SIDES };
                    edges[edge] = table[s][((r + turn) & 3) as usize];
                }
            };
        if y > 0 {
            side(x, y - 1, 2, -1, 4, 1, 0x20, 0, 0, 2);
        }
        if y < n - 1 {
            side(x, y + 1, 4, 1, 2, -1, 0x10, 2, 2, 0);
        }
        if x > 0 {
            side(x - 1, y, 6, 1, 4, -1, 0x8, 3, 3, 1);
        }
        if x < n - 1 {
            side(x + 1, y, 4, -1, 6, 1, 0x4, 1, 1, 3);
        }
        if let Some(o) = overlay.filter(|o| o.blends) {
            for (p, &covered) in COVERS[shape].iter().enumerate().take(8) {
                if covered {
                    around.take(((p as i32 - rotation * 2) & 7) as usize, o, 2);
                }
            }
        }
    }
}

/// `Class371.method6347`: packed HSL `a` blended towards `b` by `t` of 128, each part apart.
fn mix(a: i32, b: i32, t: i32) -> i32 {
    if a == b {
        return a;
    }
    let u = 128 - t;
    let light = (u * (a & 0x7f) + (b & 0x7f) * t) >> 7;
    let sat = (u * (a & 0x380) + t * (b & 0x380)) >> 7;
    let hue = ((b & 0xfc00) * t + u * (a & 0xfc00)) >> 7;
    (hue & 0xfc00) | (sat & 0x380) | (light & 0x7f)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn overlay(colour: i32, priority: i32, blends: bool) -> Overlay {
        Overlay { colour, texture: -1, secondary: -1, priority, blends }
    }

    #[test]
    fn plain_shapes_split_a_tile_between_overlay_and_underlay() {
        // Shape 12 (no overlay): two underlay triangles over the whole tile.
        let all = plain(12, 0, None, Some(7));
        assert_eq!(all, vec![[(0, 0, 7), (512, 0, 7), (0, 512, 7)], [(512, 0, 7), (512, 512, 7), (0, 512, 7)]]);
        // Shape 1 turned once, (x, y) becomes (y, 512 − x): one overlay triangle, one underlay.
        let half = plain(1, 1, Some(3), Some(7));
        assert_eq!(half, vec![[(0, 512, 3), (0, 0, 3), (512, 512, 3)], [(0, 0, 7), (512, 0, 7), (512, 512, 7)]]);
        assert!(plain(0, 0, None, None).is_empty());
    }

    #[test]
    fn colours_mix_part_by_part() {
        assert_eq!(mix(5, 5, 64), 5);
        // Halfway between lightness 0 and 100, hue 0 and 2.
        assert_eq!(mix(0, (2 << 10) | 100, 64), (1 << 10) | 50);
    }

    /// The underlays of `tiles`, blended colour `c` everywhere, flat heights.
    fn scene(tiles: &[(u8, u8, u8, u8)], c: i32) -> (Vec<Option<Underlay>>, Vec<i32>, Vec<i32>) {
        let under = tiles.iter().map(|t| (t.0 != 0).then(Underlay::default)).collect();
        (under, vec![c; SCENE * SCENE], vec![0; (SCENE + 1) * (SCENE + 1)])
    }

    #[test]
    fn a_lone_underlay_tile_takes_the_corner_colours() {
        let tiles = vec![(1, 0, 0, 0); SCENE * SCENE];
        let overlays = vec![None; SCENE * SCENE];
        let (underlays, blended, heights) = scene(&tiles, 9);
        let f = Floors { tiles: &tiles, overlays: &overlays, underlays: &underlays, blended: &blended, heights: &heights };
        let faces = f.blended(10, 10);
        // Shape 12 split along a diagonal: two triangles, every point the blended colour.
        assert_eq!(faces.len(), 2);
        assert!(faces.iter().flatten().all(|p| p.2 == 9));
    }

    #[test]
    fn a_blending_overlay_reaches_into_the_tile_beside_it() {
        // Tile (10, 9) is a full blending overlay; (10, 10) above it is plain underlay.
        let mut tiles = vec![(1, 0, 0, 0); SCENE * SCENE];
        let mut overlays = vec![None; SCENE * SCENE];
        tiles[10 * SCENE + 9] = (1, 1, 0, 0);
        overlays[10 * SCENE + 9] = Some(overlay(500, (8 << 8) | 1, true));
        let (underlays, blended, heights) = scene(&tiles, 9);
        let f = Floors { tiles: &tiles, overlays: &overlays, underlays: &underlays, blended: &blended, heights: &heights };
        let faces = f.blended(10, 10);
        // The south side's points (y 0) take the overlay's colour; the north ones stay underlay.
        let points: Vec<Point> = faces.iter().flatten().copied().collect();
        assert!(points.iter().filter(|p| p.1 == 0).all(|p| p.2 == 500));
        assert!(points.iter().filter(|p| p.1 == 512).all(|p| p.2 == 9));
    }
}
