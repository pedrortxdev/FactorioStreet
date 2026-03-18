use crate::constants::Terrain;
use macroquad::prelude::*;

pub const TILE_SIZE: usize = 128;
pub const P: usize = 4; // Pixel size for the "Atomic" look

#[derive(Clone, Copy)]
pub struct Generator {
    pub seed: f32,
}

impl Generator {
    pub fn new(seed: f32) -> Self {
        Self { seed }
    }

    fn hex(h: &str) -> Color {
        let h = h.trim_start_matches('#');
        let r = u8::from_str_radix(&h[0..2], 16).unwrap_or(0);
        let g = u8::from_str_radix(&h[2..4], 16).unwrap_or(0);
        let b = u8::from_str_radix(&h[4..6], 16).unwrap_or(0);
        Color::from_rgba(r, g, b, 255)
    }

    fn get_palette(t: Terrain) -> Vec<Color> {
        match t {
            Terrain::Empty => vec![Self::hex("#14290d"), Self::hex("#2d5e1e"), Self::hex("#3f822a"), Self::hex("#5ba339"), Self::hex("#82d152")],
            Terrain::Water => vec![Self::hex("#0a1430"), Self::hex("#124ca1"), Self::hex("#2091e3"), Self::hex("#42d4ff"), Self::hex("#ffffff")],
            Terrain::Sand => vec![Self::hex("#634b35"), Self::hex("#ad683e"), Self::hex("#d29a58"), Self::hex("#f1c27d"), Self::hex("#ffedaf")],
            Terrain::Uranium => vec![Self::hex("#050a05"), Self::hex("#0d2b0d"), Self::hex("#2bff00"), Self::hex("#9dff00"), Self::hex("#ffffff")],
            Terrain::Oil => vec![Self::hex("#050505"), Self::hex("#120d1a"), Self::hex("#2d1b33"), Self::hex("#4a2b52"), Self::hex("#a357b3"), Self::hex("#52e3e8")],
            Terrain::Gold => vec![Self::hex("#5e3314"), Self::hex("#a66014"), Self::hex("#f2a61a"), Self::hex("#ffd541"), Self::hex("#ffffff")],
            Terrain::Iron => vec![Self::hex("#2e2e2e"), Self::hex("#4a4a4a"), Self::hex("#7a7a7a"), Self::hex("#b0b0b0"), Self::hex("#ffffff")],
            Terrain::Coal => vec![Self::hex("#000000"), Self::hex("#0a0a0a"), Self::hex("#1a1a1a"), Self::hex("#333333"), Self::hex("#4a4a4a")],
            _ => vec![Self::hex("#1a1c2c"), Self::hex("#333c57"), Self::hex("#566c86"), Self::hex("#94b0c2"), Self::hex("#f4f4f4")], // Pedra/Default
        }
    }

    fn seeded_random(&self, s: f32) -> f32 {
        let x = (s + self.seed).sin() * 10000.0;
        x - x.floor()
    }

    pub fn generate_texture(&self, terrain: Terrain, time: f32) -> Texture2D {
        let mut img = Image::gen_image_color(TILE_SIZE as u16, TILE_SIZE as u16, BLANK);
        let pal = Self::get_palette(terrain);

        match terrain {
            Terrain::Water => self.draw_water(&mut img, &pal, time),
            Terrain::Uranium => self.draw_uranium(&mut img, &pal, time),
            Terrain::Oil => self.draw_oil(&mut img, &pal, time),
            Terrain::Tree => self.draw_tree(&mut img, &pal),
            _ => self.draw_base_terrain(&mut img, &pal, terrain as usize as f32),
        }

        Texture2D::from_image(&img)
    }

    fn draw_base_terrain(&self, img: &mut Image, pal: &[Color], id: f32) {
        // Fill base
        for y in 0..TILE_SIZE {
            for x in 0..TILE_SIZE {
                img.set_pixel(x as u32, y as u32, pal[1]);
            }
        }

        // Noise dots (Atomic Dots logic)
        for d in 0..180 {
            let dx = self.seeded_random(self.seed + id + d as f32 * 0.5) * TILE_SIZE as f32;
            let dy = self.seeded_random(self.seed + id + d as f32 * 0.8) * TILE_SIZE as f32;
            let val = self.seeded_random(self.seed + id + d as f32 * 0.13);

            let color = if val > 0.88 { pal[0] } else if val < 0.12 { pal[2] } else { continue };
            
            // Draw PxP pixel block (4x4)
            let px = (dx as i32 / P as i32) * P as i32;
            let py = (dy as i32 / P as i32) * P as i32;
            for iy in 0..P as i32 {
                for ix in 0..P as i32 {
                    let rx = px + ix;
                    let ry = py + iy;
                    if rx >= 0 && rx < TILE_SIZE as i32 && ry >= 0 && ry < TILE_SIZE as i32 {
                        img.set_pixel(rx as u32, ry as u32, color);
                    }
                }
            }
        }
    }

    fn draw_water(&self, img: &mut Image, pal: &[Color], t: f32) {
        // Deep blue base
        for y in 0..TILE_SIZE {
            for x in 0..TILE_SIZE {
                img.set_pixel(x as u32, y as u32, pal[0]);
            }
        }

        // Waves (spatial noise)
        for y in (0..TILE_SIZE).step_by(P * 2) {
            for x in (0..TILE_SIZE).step_by(P * 2) {
                let noise = (x as f32 * 0.04 + t * 0.003).sin() * (y as f32 * 0.04 + t * 0.002).cos();
                let color = if noise > 0.8 { pal[3] } else if noise > 0.6 { pal[2] } else { continue };
                
                // Draw 8x8 blocks as in JS P*2
                for iy in 0..(P*2) {
                    for ix in 0..(P*2) {
                        let rx = x + ix;
                        let ry = y + iy;
                        if rx < TILE_SIZE && ry < TILE_SIZE {
                            img.set_pixel(rx as u32, ry as u32, color);
                        }
                    }
                }
            }
        }

        // Specular highlight line (scrolling)
        let line_y = (t * 0.015) as u32 % TILE_SIZE as u32;
        let mut highlight = pal[3];
        highlight.a = 0.2;
        for x in 0..TILE_SIZE {
            for iy in 0..P as u32 {
                let ry = line_y + iy;
                if ry < TILE_SIZE as u32 {
                    // Manual blend alpha since img.set_pixel overwrites
                    let old = img.get_pixel(x as u32, ry);
                    let blended = Color::new(
                        old.r * (1.0 - 0.2) + highlight.r * 0.2,
                        old.g * (1.0 - 0.2) + highlight.g * 0.2,
                        old.b * (1.0 - 0.2) + highlight.b * 0.2,
                        1.0
                    );
                    img.set_pixel(x as u32, ry, blended);
                }
            }
        }
    }

    fn draw_uranium(&self, img: &mut Image, pal: &[Color], t: f32) {
        // Pedra base
        self.draw_base_terrain(img, &Self::get_palette(Terrain::Iron), 42.0);
        
        // Glow pulse
        let pulso = ((t * 0.006).sin() + 1.0) / 2.0;
        let centers = [(48.0, 50.0, 34.0), (85.0, 75.0, 22.0)];

        for (cx, cy, r) in centers {
            // Radioactive cores
            self.draw_circle_img(img, cx, cy, r, pal[0]);

            // Rotating crystals
            for i in 0..6 {
                let ang = (i as f32 * std::f32::consts::PI / 3.0) + (t * 0.0008);
                let kx = cx + ang.cos() * (r * 0.7);
                let ky = cy + ang.sin() * (r * 0.7);
                self.draw_rect_img(img, kx as i32, ky as i32, P * 2, P * 2, pal[2]);
                self.draw_rect_img(img, (kx + P as f32 / 2.0) as i32, (ky + P as f32 / 2.0) as i32, 2, 2, pal[4]);
            }

            // Ascending Gamma Particles
            for p in 0..4 {
                let pt = (t * 0.2 + p as f32 * 150.0) % 100.0;
                let px = cx + (pt * 0.1).sin() * 20.0;
                let py = cy - pt;
                self.draw_rect_img(img, px as i32, py as i32, P, P, pal[3]);
            }

            // Pulse Glow
            let mut glow = pal[2];
            glow.a = 0.15 * pulso;
            self.draw_circle_img_blended(img, cx, cy, r + 20.0 * pulso, glow);
        }
    }

    fn draw_oil(&self, img: &mut Image, pal: &[Color], t: f32) {
        self.draw_base_terrain(img, &Self::get_palette(Terrain::Iron), 42.0); // Stone base
        
        let ox = TILE_SIZE as f32 / 2.0;
        let oy = TILE_SIZE as f32 / 2.0;

        // Dark puddle base (ellipse-like)
        self.draw_circle_img(img, ox, oy, 50.0, Color::new(0.0, 0.0, 0.0, 1.0));

        // Iridescent bubbles
        for i in 0..12 {
            let ang = (i as f32 * std::f32::consts::PI / 6.0) + t * 0.002;
            let bx = ox + ang.cos() * 32.0;
            let by = oy + ang.sin() * 22.0;
            
            let mut bubble_col = if i % 2 == 0 { pal[4] } else { pal[5] };
            bubble_col.a = 0.5;
            self.draw_circle_img_blended(img, bx, by, 10.0, bubble_col);
        }
    }

    fn draw_tree(&self, img: &mut Image, pal: &[Color]) {
        self.draw_base_terrain(img, pal, 5.0); // Grass base
        
        let tx = TILE_SIZE as f32 / 2.0;
        let ty = TILE_SIZE as f32 / 2.0;

        // Shadow
        self.draw_circle_img_blended(img, tx + 12.0, ty + 12.0, 50.0, Color::new(0.0, 0.0, 0.0, 0.5));

        // Foliage Layers
        let radii = [54.0, 42.0, 28.0, 15.0, 6.0];
        for (i, &r) in radii.iter().enumerate() {
            let offset = i as f32 * 3.0;
            self.draw_circle_img(img, tx - offset, ty - offset, r, pal[i]);
        }
    }

    // Helper: Rectangle with scale P alignment
    fn draw_rect_img(&self, img: &mut Image, x: i32, y: i32, w: usize, h: usize, color: Color) {
        let align_x = (x / P as i32) * P as i32;
        let align_y = (y / P as i32) * P as i32;
        for iy in 0..h as i32 {
            for ix in 0..w as i32 {
                let rx = align_x + ix;
                let ry = align_y + iy;
                if rx >= 0 && rx < TILE_SIZE as i32 && ry >= 0 && ry < TILE_SIZE as i32 {
                    img.set_pixel(rx as u32, ry as u32, color);
                }
            }
        }
    }

    fn draw_circle_img(&self, img: &mut Image, cx: f32, cy: f32, r: f32, color: Color) {
        for y in 0..TILE_SIZE {
            for x in 0..TILE_SIZE {
                let dx = x as f32 - cx;
                let dy = y as f32 - cy;
                if dx*dx + dy*dy < r*r {
                    img.set_pixel(x as u32, y as u32, color);
                }
            }
        }
    }

    fn draw_circle_img_blended(&self, img: &mut Image, cx: f32, cy: f32, r: f32, color: Color) {
        let alpha = color.a;
        if alpha >= 1.0 { self.draw_circle_img(img, cx, cy, r, color); return; }
        for y in 0..TILE_SIZE {
            for x in 0..TILE_SIZE {
                let dx = x as f32 - cx;
                let dy = y as f32 - cy;
                if dx*dx + dy*dy < r*r {
                    let old = img.get_pixel(x as u32, y as u32);
                    let blended = Color::new(
                        old.r * (1.0 - alpha) + color.r * alpha,
                        old.g * (1.0 - alpha) + color.g * alpha,
                        old.b * (1.0 - alpha) + color.b * alpha,
                        1.0
                    );
                    img.set_pixel(x as u32, y as u32, blended);
                }
            }
        }
    }
}
