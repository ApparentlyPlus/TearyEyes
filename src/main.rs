use eframe::egui;

mod window; // Win95 window layout

#[allow(non_snake_case)]
mod AngryMan;

use AngryMan::{ExportData};

struct W95Playback {
    data: ExportData,
    current_frame: usize,
    last_update: std::time::Instant,
    image: egui::ColorImage,
    texture: Option<egui::TextureHandle>,
    flip_h: bool,
    flip_v: bool,
}

impl W95Playback {
    fn new(cc: &eframe::CreationContext<'_>, data: ExportData) -> Self {

        // Enforce classic Win95 global styling
        let mut visuals = egui::Visuals::light();
        visuals.popup_shadow = egui::epaint::Shadow::NONE;
        visuals.window_shadow = egui::epaint::Shadow::NONE;
        visuals.window_rounding = egui::Rounding::ZERO;
        visuals.menu_rounding = egui::Rounding::ZERO;
        visuals.widgets.noninteractive.rounding = egui::Rounding::ZERO;
        visuals.widgets.inactive.rounding = egui::Rounding::ZERO;
        visuals.widgets.hovered.rounding = egui::Rounding::ZERO;
        visuals.widgets.active.rounding = egui::Rounding::ZERO;
        visuals.widgets.open.rounding = egui::Rounding::ZERO;
        visuals.widgets.hovered.expansion = 0.0;
        visuals.widgets.active.expansion = 0.0;
        visuals.widgets.open.expansion = 0.0;

        // Set background to gray and remove window borders
        visuals.window_fill = egui::Color32::from_rgb(192, 192, 192);
        visuals.window_stroke = egui::Stroke::NONE;

        // Reset hover styles
        visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(192, 192, 192); // this is the hover background color, set to gray to match Win95
        visuals.widgets.hovered.fg_stroke.color = egui::Color32::BLACK;

        // Same idea for active
        visuals.widgets.active.bg_fill = egui::Color32::from_rgb(192, 192, 192);
        visuals.widgets.active.fg_stroke.color = egui::Color32::BLACK;

        // You get it by now chief
        visuals.widgets.open.bg_fill = egui::Color32::from_rgb(192, 192, 192);
        visuals.widgets.open.fg_stroke.color = egui::Color32::BLACK;

        // Apply the modified style
        let mut style = (*cc.egui_ctx.style()).clone();
        style.visuals = visuals;
        style.spacing.menu_margin = egui::Margin::same(0.0);
        cc.egui_ctx.set_style(style);

        // Initialize our player with the first frame as a white canvas
        let image = egui::ColorImage::new([data.width, data.height], egui::Color32::WHITE);
        Self {
            data,
            current_frame: 0,
            last_update: std::time::Instant::now(),
            image,
            texture: None,
            flip_h: false,
            flip_v: false,
        }
    }
}

impl eframe::App for W95Playback {

    // this is the main rendering loop
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let now = std::time::Instant::now();
        
        // 30 FPS update (1.0 / 30.0 = 0.0333 seconds)
        if now.duration_since(self.last_update).as_secs_f32() > 0.0333 {
            self.last_update = now;
            
            // Loop frame counter
            if self.current_frame >= self.data.frames.len() {
                self.current_frame = 0;
                // Clear to white when looping
                for p in self.image.pixels.iter_mut() {
                    *p = egui::Color32::WHITE;
                }
            }

            // Draw current frame deltas
            let frame_data = &self.data.frames[self.current_frame];
            for &[x, y] in frame_data.black_pixels {
                if x < self.data.width && y < self.data.height {
                    let idx = y * self.data.width + x;
                    if idx < self.image.pixels.len() {
                        self.image.pixels[idx] = egui::Color32::BLACK;
                    }
                }
            }
            for &[x, y] in frame_data.white_pixels {
                if x < self.data.width && y < self.data.height {
                    let idx = y * self.data.width + x;
                    if idx < self.image.pixels.len() {
                        self.image.pixels[idx] = egui::Color32::WHITE;
                    }
                }
            }

            self.current_frame += 1;
            
            // Upload to GPU
            self.texture = Some(ctx.load_texture("video_frame", self.image.clone(), egui::TextureOptions::NEAREST));
        }

        ctx.request_repaint();

        let texture = self.texture.get_or_insert_with(|| {
            ctx.load_texture("video_frame", self.image.clone(), egui::TextureOptions::NEAREST)
        });

        // Delegate rendering to the layout file
        window::dw(ctx, texture, self.data.width as f32, self.data.height as f32, &mut self.flip_h, &mut self.flip_v);
    }
}

fn generate_icon() -> std::sync::Arc<egui::IconData> {
    let width = 32;
    let height = 32;
    let mut rgba = vec![0; (width * height * 4) as usize];

    for y in 0..height {
        for x in 0..width {
            let idx = ((y * width + x) * 4) as usize;
            
            // Background padding (transparent)
            if x < 2 || x >= width - 2 || y < 2 || y >= height - 2 {
                rgba[idx..idx + 4].copy_from_slice(&[0, 0, 0, 0]);
                continue;
            }
            
            // Outline
            if x == 2 || x == width - 3 || y == 2 || y == height - 3 {
                rgba[idx..idx + 4].copy_from_slice(&[100, 100, 100, 255]);
                continue;
            }

            // Notepad background
            rgba[idx..idx + 4].copy_from_slice(&[240, 240, 240, 255]);

            // Top binding
            if y > 2 && y < 10 {
                rgba[idx..idx + 4].copy_from_slice(&[100, 150, 255, 255]);
            }
            
            // Lines
            if x > 4 && x < width - 4 {
                if y == 14 || y == 20 || y == 26 {
                    rgba[idx..idx + 4].copy_from_slice(&[200, 200, 200, 255]);
                }
            }
        }
    }

    std::sync::Arc::new(egui::IconData {
        rgba,
        width,
        height,
    })
}

fn main() -> eframe::Result<()> {
    let data = AngryMan::ldata();

    // Exact size calculations:
    // 4px left margin + 2px canvas border + image + 2px canvas border + 4px right margin = 12
    // 4px top margin + 18 title bar + 24 menu bar + 1 gap + 2px canvas border + image + 2px canvas border + 4px bottom margin = 55

    let ww = data.width as f32 + 12.0;
    let wh = data.height as f32 + 55.0;

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([ww, wh])
            .with_decorations(false) // Borderless OS window!
            .with_transparent(true)
            .with_icon(generate_icon()),
        ..Default::default()
    };

    eframe::run_native(
        "TearyEyes",
        options,
        Box::new(|cc| Box::new(W95Playback::new(cc, data))),
    )
}
