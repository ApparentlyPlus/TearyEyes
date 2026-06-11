use eframe::egui;

// we use these throughout
const GRAY: egui::Color32 = egui::Color32::from_rgb(192, 192, 192);
const NAVY: egui::Color32 = egui::Color32::from_rgb(0, 0, 128);
const DARK_GRAY: egui::Color32 = egui::Color32::from_rgb(128, 128, 128);
const LIGHT_GRAY: egui::Color32 = egui::Color32::from_rgb(223, 223, 223);
const WHITE: egui::Color32 = egui::Color32::WHITE;
const BLACK: egui::Color32 = egui::Color32::BLACK;

// Helper to draw the classic 3D beveled borders with a single line
fn d_bevel(painter: &egui::Painter, r: egui::Rect, tl: egui::Color32, br: egui::Color32) {

    let t = egui::Stroke::new(1.0, tl);
    let b = egui::Stroke::new(1.0, br);

    // Draw top and left
    painter.line_segment([r.left_top(), r.right_top()], t);
    painter.line_segment([r.left_top(), r.left_bottom()], t);

    // Draw bottom and right
    painter.line_segment([r.left_bottom(), r.right_bottom()], b);
    painter.line_segment([r.right_top(), r.right_bottom()], b);
}

// Helper for drawing Win95 caption buttons (16x14)
fn d_caption_btn(ui: &mut egui::Ui, rect: egui::Rect, id: &str, icon: impl FnOnce(&egui::Painter, egui::Rect)) -> bool {
    let response = ui.interact(rect, egui::Id::new(id), egui::Sense::click());
    
    ui.painter().rect_filled(rect, 0.0, GRAY);
    
    let pressed = response.is_pointer_button_down_on();
    if pressed {
        // Pressed is a 1px dark border
        ui.painter().rect_stroke(rect, 0.0, egui::Stroke::new(1.0, DARK_GRAY));
    } else {
        // Raised is 2px
        d_bevel(ui.painter(), rect, WHITE, BLACK);
        d_bevel(ui.painter(), rect.shrink(1.0), LIGHT_GRAY, DARK_GRAY);
    }

    let icon_rect = if pressed { rect.translate(egui::vec2(1.0, 1.0)) } else { rect };
    icon(ui.painter(), icon_rect);

    response.clicked()
}

// Custom wrapper to perfectly draw dropdown menus
fn dd(ui: &mut egui::Ui, title: &str, add_contents: impl FnOnce(&mut egui::Ui)) {
    let response = ui.menu_button(title, |ui| {
        let frame = egui::Frame::none()
            .fill(GRAY)
            .inner_margin(egui::Margin::symmetric(4.0, 4.0));
            
        let res = frame.show(ui, |ui| {
            // Remove spacing between items
            ui.spacing_mut().item_spacing.y = 0.0;
            add_contents(ui);
        });
        
        // Draw AltRaised 3D border over the entire popup
        let r = res.response.rect;
        d_bevel(ui.painter(), r, LIGHT_GRAY, BLACK);
        d_bevel(ui.painter(), r.shrink(1.0), WHITE, DARK_GRAY);
    });
    
    // Draw ThinRaised border over the Top Level Button if hovered or open
    let btn_rect = response.response.rect;
    if response.response.hovered() || response.inner.is_some() {
        d_bevel(ui.painter(), btn_rect, WHITE, DARK_GRAY);
    }
}

// Custom wrapper to perfectly draw internal menu items
pub fn d_menu_item(ui: &mut egui::Ui, text: &str) -> egui::Response {
    let size = egui::vec2(ui.available_width(), 18.0);
    let (rect, response) = ui.allocate_exact_size(size, egui::Sense::click());
    
    if response.hovered() {
        ui.painter().rect_filled(rect, 0.0, NAVY);
        ui.painter().text(rect.left_center() + egui::vec2(4.0, 0.0), egui::Align2::LEFT_CENTER, text, egui::FontId::proportional(12.0), WHITE);
    } else {
        ui.painter().text(rect.left_center() + egui::vec2(4.0, 0.0), egui::Align2::LEFT_CENTER, text, egui::FontId::proportional(12.0), BLACK);
    }
    response
}

// Notepad-ish
fn d_icon(painter: &egui::Painter, rect: egui::Rect) {
    let r = rect.shrink(2.0);
    
    // Notepad background
    painter.rect_filled(r, 1.0, egui::Color32::from_rgb(240, 240, 240));
    
    // Notepad binding
    let binding_rect = egui::Rect::from_min_max(r.left_top(), egui::pos2(r.right(), r.top() + r.height() * 0.25));
    painter.rect_filled(binding_rect, 1.0, egui::Color32::from_rgb(100, 150, 255));
    
    // Horizontal lines
    let ls = egui::Stroke::new(1.0, egui::Color32::from_rgb(200, 200, 200));
    for i in 1..=3 {
        let y = binding_rect.bottom() + (r.height() * 0.75) * (i as f32 / 4.0);
        painter.line_segment(
            [egui::pos2(r.left() + 2.0, y), egui::pos2(r.right() - 2.0, y)],
            ls,
        );
    }
    
    // Outline
    painter.rect_stroke(r, 1.0, egui::Stroke::new(1.0, egui::Color32::from_rgb(100, 100, 100)));
}

// Invisible resize handles around the edges
fn enable_resizing(ctx: &egui::Context, ui: &mut egui::Ui) {

    // Don't show resize handles if maximized, since that doesn't make sense and can cause issues
    if ctx.input(|i| i.viewport().maximized.unwrap_or(false)) {
        return;
    }

    let r = ctx.screen_rect();
    let edge = 6.0; // slightly thicker corner grab area for ease
    
    // Define 4 corner handles (NW, NE, SW, SE)
    let handles = [
        (egui::Rect::from_min_max(r.left_top(), r.left_top() + egui::vec2(edge*2.0, edge*2.0)), egui::ResizeDirection::NorthWest),
        (egui::Rect::from_min_max(r.right_top() - egui::vec2(edge*2.0, 0.0), r.right_top() + egui::vec2(0.0, edge*2.0)), egui::ResizeDirection::NorthEast),
        (egui::Rect::from_min_max(r.left_bottom() - egui::vec2(0.0, edge*2.0), r.left_bottom() + egui::vec2(edge*2.0, 0.0)), egui::ResizeDirection::SouthWest),
        (egui::Rect::from_min_max(r.right_bottom() - egui::vec2(edge*2.0, edge*2.0), r.right_bottom()), egui::ResizeDirection::SouthEast),
    ];

    // Add invisible interactive areas for each handle
    for (rect, dir) in handles {
        let response = ui.interact(rect, egui::Id::new(format!("{:?}", dir)), egui::Sense::drag());
        
        // Add hover cursor
        let cursor = match dir {
            egui::ResizeDirection::NorthWest | egui::ResizeDirection::SouthEast => egui::CursorIcon::ResizeNwSe,
            egui::ResizeDirection::NorthEast | egui::ResizeDirection::SouthWest => egui::CursorIcon::ResizeNeSw,
            _ => egui::CursorIcon::Default,
        };
        if response.hovered() {
            ctx.set_cursor_icon(cursor);
        }
        
        if response.drag_started() {
            ctx.send_viewport_cmd(egui::ViewportCommand::BeginResize(dir));
        }
    }
}

pub fn dw(ctx: &egui::Context, texture: &egui::TextureHandle, width: f32, height: f32, flip_h: &mut bool, flip_v: &mut bool, shake_offset: egui::Vec2) {
    let frame = egui::Frame::none().fill(GRAY).inner_margin(4.0);

    egui::CentralPanel::default().frame(frame).show(ctx, |ui| {
        let inertia = |factor: f32| egui::vec2(-shake_offset.x * factor, -shake_offset.y * factor);

        // Title Bar
        let (mut title_rect, _) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 18.0), egui::Sense::hover());
        title_rect = title_rect.translate(inertia(0.8));
        
        if ui.interact(title_rect, egui::Id::new("drag"), egui::Sense::drag()).dragged() {
            ctx.send_viewport_cmd(egui::ViewportCommand::StartDrag);
        }

        ui.painter().rect_filled(title_rect, 0.0, NAVY);
        
        // Window Icon (16x16)
        let icon_rect = egui::Rect::from_min_size(title_rect.left_center() + egui::vec2(2.0, -8.0), egui::vec2(16.0, 16.0)).translate(inertia(1.2));
        d_icon(ui.painter(), icon_rect);

        // Bold text, offset by 20px to make space for the icon
        ui.painter().text(title_rect.left_center() + egui::vec2(22.0, 0.0), egui::Align2::LEFT_CENTER, "Notepad", egui::FontId::proportional(12.0), WHITE);
        
        // Caption Buttons
        // Margin 2 from right, spacing 0 between min/max, spacing 2 between max/close
        let close_rect = egui::Rect::from_min_size(title_rect.right_top() + egui::vec2(-16.0 - 2.0, 2.0), egui::vec2(16.0, 14.0)).translate(inertia(1.5));
        let max_rect = egui::Rect::from_min_size(close_rect.left_top() + egui::vec2(-16.0 - 2.0, 0.0), egui::vec2(16.0, 14.0)).translate(inertia(1.4));
        let min_rect = egui::Rect::from_min_size(max_rect.left_top() + egui::vec2(-16.0, 0.0), egui::vec2(16.0, 14.0)).translate(inertia(1.3));

        // Minimize
        if d_caption_btn(ui, min_rect, "min_btn", |painter, r| {
            painter.rect_filled(egui::Rect::from_min_max(r.min + egui::vec2(4.0, 9.0), r.min + egui::vec2(10.0, 11.0)), 0.0, BLACK);
        }) {
            ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(true));
        }

        // Maximize/Restore
        let maximized = ctx.input(|i| i.viewport().maximized.unwrap_or(false));
        if d_caption_btn(ui, max_rect, "max_btn", |painter, r| {
            if maximized {
                // Restore icon
                let b1 = egui::Rect::from_min_max(r.min + egui::vec2(5.0, 2.0), r.min + egui::vec2(11.0, 8.0));
                painter.rect_stroke(b1, 0.0, egui::Stroke::new(1.0, BLACK));
                painter.rect_filled(egui::Rect::from_min_max(b1.min, b1.min + egui::vec2(6.0, 2.0)), 0.0, BLACK);
                
                let b2 = egui::Rect::from_min_max(r.min + egui::vec2(2.0, 5.0), r.min + egui::vec2(8.0, 11.0));
                painter.rect_filled(b2, 0.0, GRAY);
                painter.rect_stroke(b2, 0.0, egui::Stroke::new(1.0, BLACK));
                painter.rect_filled(egui::Rect::from_min_max(b2.min, b2.min + egui::vec2(6.0, 2.0)), 0.0, BLACK);
            } else {
                // Maximize icon
                let br = egui::Rect::from_min_max(r.min + egui::vec2(3.0, 3.0), r.min + egui::vec2(12.0, 11.0));
                painter.rect_stroke(br, 0.0, egui::Stroke::new(1.0, BLACK));
                painter.rect_filled(egui::Rect::from_min_max(br.min, br.min + egui::vec2(9.0, 2.0)), 0.0, BLACK);
            }
        }) {
            ctx.send_viewport_cmd(egui::ViewportCommand::Maximized(!maximized));
        }

        // Close
        if d_caption_btn(ui, close_rect, "close_btn", |painter, r| {
            let p0 = r.left_top() + egui::vec2(4.0, 4.0);
            let p1 = r.right_bottom() + egui::vec2(-5.0, -4.0);
            let p2 = r.right_top() + egui::vec2(-5.0, 4.0);
            let p3 = r.left_bottom() + egui::vec2(4.0, -4.0);
            painter.line_segment([p0, p1], egui::Stroke::new(1.5, BLACK));
            painter.line_segment([p2, p3], egui::Stroke::new(1.5, BLACK));
        }) {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }

        // Menu Bar
        ui.horizontal(|ui| {
            // Remove margin below the menu bar
            ui.style_mut().spacing.item_spacing.y = 0.0;
            
            egui::menu::bar(ui, |ui| {
                dd(ui, "File", |ui| {
                    if d_menu_item(ui, "Exit").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
                dd(ui, "Edit", |ui| {
                    if d_menu_item(ui, "Flip horizontally").clicked() {
                        *flip_h = !*flip_h;
                        ui.close_menu();
                    }
                    if d_menu_item(ui, "Flip vertically").clicked() {
                        *flip_v = !*flip_v;
                        ui.close_menu();
                    }
                });
                dd(ui, "Search", |ui| {
                    if d_menu_item(ui, "Search text").clicked() {
                        ui.close_menu();
                    }
                });
                dd(ui, "Help", |ui| {
                    if d_menu_item(ui, "Get Help").clicked() {
                        ui.close_menu();
                    }
                });
            });
        });

        // 1px gap between Menu Bar and Content
        ui.add_space(1.0);
        
        // Canvas
        ui.vertical_centered(|ui| {

            // keep it stretched to fill 
            let padding = egui::vec2(8.0, 8.0); // 4px margin on each side for the sunken border
            let available = ui.available_size() - padding;
            
            // Stretch to fill (ignore aspect ratio)
            let mut new_w = available.x;
            let mut new_h = available.y;
            
            // Avoid crushing to 0
            if new_w < 10.0 || new_h < 10.0 {
                new_w = width.max(10.0);
                new_h = height.max(10.0);
            }

            let (mut canvas_rect, _) = ui.allocate_exact_size(egui::vec2(new_w, new_h), egui::Sense::hover());
            canvas_rect = canvas_rect.translate(inertia(0.3));
            
            let mut uv = egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0));
            if *flip_h {
                std::mem::swap(&mut uv.min.x, &mut uv.max.x);
            }
            if *flip_v {
                std::mem::swap(&mut uv.min.y, &mut uv.max.y);
            }
            ui.painter().image(texture.id(), canvas_rect, uv, WHITE);
            
            // True Sunken border for Canvas (2 pixels thick)
            let canvas_outer = canvas_rect.expand(2.0);
            d_bevel(ui.painter(), canvas_outer, DARK_GRAY, WHITE);
            d_bevel(ui.painter(), canvas_outer.shrink(1.0), BLACK, LIGHT_GRAY);
        });

        // Register edges for native resizing
        enable_resizing(ctx, ui);
    });

    // Outer 3D Borders
    let painter = ctx.layer_painter(egui::LayerId::new(egui::Order::Foreground, egui::Id::new("border")));
    let r = ctx.screen_rect();
    
    d_bevel(&painter, r, LIGHT_GRAY, BLACK);
    d_bevel(&painter, r.shrink(1.0), WHITE, DARK_GRAY);

    if shake_offset.length_sq() > 0.0 {
        let p = ctx.layer_painter(egui::LayerId::new(egui::Order::Foreground, egui::Id::new("smear")));
        let time = ctx.input(|i| i.time);
        
        // Horizontal glitch streaks
        for i in 0..15 {
            let prand = (time * 1000.0 + i as f64) as u64 * 12345;
            let x = ((prand >> 4) % r.width() as u64) as f32;
            let y = ((prand >> 8) % r.height() as u64) as f32;
            let w = ((prand >> 12) % 150) as f32 + 20.0;
            let h = ((prand >> 16) % 8) as f32 + 2.0;

            let color = match prand % 4 {
                0 => NAVY,
                1 => GRAY,
                2 => WHITE,
                _ => BLACK,
            };
            
            p.rect_filled(egui::Rect::from_min_size(egui::pos2(x, y), egui::vec2(w, h)), 0.0, color);
        }
        
        // Melting vertical drips
        for i in 0..60 {
            let prand = (time * 1000.0 + i as f64 * 7.0) as u64 * 54321;
            let x = ((prand >> 3) % r.width() as u64) as f32;
            let y = ((prand >> 7) % r.height() as u64) as f32;
            let len = ((prand >> 11) % 60) as f32 + 10.0;

            // Sample a color typical of the UI at that Y position
            let color = if y < 22.0 {
                NAVY
            } else if y < 45.0 {
                if prand % 2 == 0 { BLACK } else { GRAY }
            } else {
                if prand % 3 == 0 { WHITE } else if prand % 3 == 1 { DARK_GRAY } else { BLACK }
            };

            p.line_segment([egui::pos2(x, y), egui::pos2(x, y + len)], egui::Stroke::new(if prand % 2 == 0 { 2.0 } else { 4.0 }, color));
        }
    }
}
