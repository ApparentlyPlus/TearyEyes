#![windows_subsystem = "windows"]

use eframe::egui;

mod window; // Win95 window layout

#[allow(non_snake_case)]
mod AngryMan;

use AngryMan::{ExportData};

struct W95Playback {
    data: ExportData,
    current_frame: usize,
    last_update: std::time::Instant,
    lastud: std::time::Instant,
    image: egui::ColorImage,
    texture: egui::TextureHandle,
    flip_h: bool,
    flip_v: bool,
    shake_frames: usize,
    targpos: Option<egui::Pos2>,
    dv: egui::Vec2,
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
        let texture = cc.egui_ctx.load_texture("video_frame", image.clone(), egui::TextureOptions::NEAREST);
        Self {
            data,
            current_frame: 0,
            last_update: std::time::Instant::now(),
            lastud: std::time::Instant::now(),
            image,
            texture,
            flip_h: false,
            flip_v: false,
            shake_frames: 0,
            targpos: None,
            dv: egui::vec2(0.0, 0.0),
        }
    }
}

impl eframe::App for W95Playback {

    // this is the main rendering loop
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let now = std::time::Instant::now();
        
        // Video Frame Update (30 FPS)
        let dt = now.duration_since(self.last_update).as_secs_f32();
        if dt >= 0.0333333 {
            if dt > 0.1 {
                self.last_update = now;
            } else {
                self.last_update += std::time::Duration::from_secs_f32(0.0333333);
            }
            
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
            if frame_data.shake {
                self.shake_frames = 5;
                let time = ctx.input(|i| i.time);
                let angle = time as f32 * 100.0;
                self.dv += egui::vec2(angle.cos() * 2.0, angle.sin() * 2.0);
            }

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
            
            if self.shake_frames > 0 {
                self.shake_frames -= 1;
            }
            
            // Upload to GPU in place using TextureHandle::set
            self.texture.set(self.image.clone(), egui::TextureOptions::NEAREST);
        }

        // Window Position Update
        let mut shake_offset = egui::vec2(0.0, 0.0);
        if self.shake_frames > 0 {
            let time = ctx.input(|i| i.time);
            shake_offset.x = (time * 60.0).sin() as f32 * 3.0;
            shake_offset.y = (time * 80.0).cos() as f32 * 3.0;
        }

        let dt_phys = now.duration_since(self.lastud).as_secs_f32();
        if dt_phys >= 0.0166666 {
            if dt_phys > 0.1 {
                self.lastud = now;
            } else {
                self.lastud += std::time::Duration::from_secs_f32(0.0166666);
            }

            if self.targpos.is_none() {
                self.targpos = ctx.input(|i| i.viewport().inner_rect).map(|r| r.min);
            }

            let maximized = ctx.input(|i| i.viewport().maximized.unwrap_or(false));

            if let Some(mut tpos) = self.targpos {
                if let Some(rp) = ctx.input(|i| i.viewport().inner_rect).map(|r| r.min) {
                    if rp.distance(tpos) > 50.0 && self.shake_frames == 0 {
                        tpos = rp;
                    }
                }

                if !maximized {
                    tpos += self.dv;
                    self.dv *= 0.85; // friction
                    
                    // Keep the window inside the monitor
                    if let Some(monitor_size) = ctx.input(|i| i.viewport().monitor_size) {
                        if let Some(window_size) = ctx.input(|i| i.viewport().inner_rect).map(|r| r.size()) {
                            tpos.x = tpos.x.clamp(0.0, (monitor_size.x - window_size.x).max(0.0));
                            tpos.y = tpos.y.clamp(0.0, (monitor_size.y - window_size.y).max(0.0));
                        }
                    }
                    
                    self.targpos = Some(tpos);

                    if self.shake_frames > 0 || self.dv.length_sq() > 0.01 {
                        ctx.send_viewport_cmd(egui::ViewportCommand::OuterPosition(tpos + shake_offset));
                    }
                } else {
                    self.dv = egui::vec2(0.0, 0.0);
                    self.targpos = Some(tpos);
                }
            }
        }

        // Scheduling the next Repaint to avoid busy looping while staying blazing fast
        ctx.request_repaint_after(std::time::Duration::from_millis(8));

        // Delegate rendering to the layout file
        window::dw(ctx, &self.texture, self.data.width as f32, self.data.height as f32, &mut self.flip_h, &mut self.flip_v, shake_offset);
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
            .with_transparent(false)
            .with_icon(generate_icon()),
        ..Default::default()
    };

    eframe::run_native(
        "TearyEyes",
        options,
        Box::new(|cc| Box::new(W95Playback::new(cc, data))),
    )
}
