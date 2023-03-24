#![windows_subsystem = "windows"]

fn main() -> Result<(), eframe::Error> {
    eframe::run_native(
        "Screen painter",
        eframe::NativeOptions {
            fullscreen: true,
            transparent: true,
            resizable: false,
            // always_on_top: true, // This makes the background black for some reason (only with fullscreen (&transparent iirc))
            default_theme: eframe::Theme::Dark,
            ..Default::default()
        },
        Box::new(|cc| Box::<ScreenPainter>::new(ScreenPainter::new(cc))),
    )
}
struct ScreenPainter {
    canvas: Vec<Line>,
    recently_deleted: Vec<Line>,
    current_stroke: eframe::egui::Stroke,
    currently_drawing: bool,
}

struct Line {
    stroke: eframe::egui::Stroke,
    points: Vec<eframe::egui::Pos2>,
}

impl ScreenPainter {
    fn new(cc: &eframe::CreationContext) -> Self {
        use eframe::egui::{
            FontFamily::{Monospace, Proportional},
            FontId, TextStyle,
        };

        let mut style = (*cc.egui_ctx.style()).clone();
        style.text_styles = [
            (TextStyle::Heading, FontId::new(25.0, Proportional)),
            (TextStyle::Body, FontId::new(16.0, Proportional)),
            (TextStyle::Monospace, FontId::new(12.0, Monospace)),
            (TextStyle::Button, FontId::new(16.0, Proportional)),
            (TextStyle::Small, FontId::new(8.0, Proportional)),
        ]
        .into();
        cc.egui_ctx.set_style(style);

        Self::default()
    }

    fn remove_last_line(&mut self) {
        if let Some(removed_line) = self.canvas.pop() {
            self.recently_deleted.push(removed_line);
        } else {
            // println!("Tried to remove a line from an empty canvas")
        }
    }
    fn restore_last_line(&mut self) {
        if let Some(last_removed_line) = self.recently_deleted.pop() {
            self.canvas.push(last_removed_line);
        } else {
            // println!("The recently deleted stack is empty")
        }
    }
    fn clear_canvas(&mut self) {
        while !self.canvas.is_empty() {
            self.remove_last_line()
        }
    }

    fn render_paint_canvas(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        let (mut response, painter) =
            ui.allocate_painter(ui.available_size_before_wrap(), eframe::egui::Sense::drag());

        if let Some(pos) = response.interact_pointer_pos() {
            if !self.currently_drawing {
                // Since the last frame we were not drawing:
                // -    The cursor might have moved a lot, so it would look like a weird jump
                // -    The stroke might have changed
                //
                // So we need to create a new line in our canvas
                self.canvas.push(Line::new(self.current_stroke));
            }

            let current_line = self.canvas.last_mut().unwrap();

            // Is the current position of the cursor the last one registered ?
            if current_line.points.last() != Some(&pos) {
                current_line.points.push(pos);
                response.mark_changed();
            }
            self.currently_drawing = true
        } else {
            // click was not held this frame

            // clean any empty lines
            self.canvas
                .retain(|line| line.points.len() > 1 && line.stroke.width != 0.0);

            self.currently_drawing = false;
        }

        let shapes = self
            .canvas
            .iter()
            .map(|line| eframe::egui::Shape::line(line.points.clone(), line.stroke));

        painter.extend(shapes);
        response
    }
    fn render_tooltip(
        &mut self,
        ctx: &eframe::egui::Context,
        ui: &mut eframe::egui::Ui,
        frame: &mut eframe::Frame,
    ) {
        if ui
            .button(format!("Clear all ({})", self.canvas.len()))
            .clicked()
        {
            self.clear_canvas();
        }

        // Add a button and a CTRL+Z shortcut to remove the last drawn line
        if ui.button("Remove last").clicked()
            || ctx.input_mut(|i| {
                i.consume_shortcut(&eframe::egui::KeyboardShortcut::new(
                    eframe::egui::Modifiers::CTRL,
                    eframe::egui::Key::Z,
                ))
            })
        {
            self.remove_last_line();
        }

        // Add a button and a CTRL+Y shortcut to restore the last deleted line
        if ui.button("Restore last").clicked()
            || ctx.input_mut(|i| {
                i.consume_shortcut(&eframe::egui::KeyboardShortcut::new(
                    eframe::egui::Modifiers::CTRL,
                    eframe::egui::Key::Y,
                ))
            })
        {
            self.restore_last_line()
        }

        // Add some controls to modify the current stroke
        let eframe::egui::Stroke { width, color } = &mut self.current_stroke;
        ui.horizontal(|ui| {
            // width control
            ui.add(eframe::egui::Slider::new(width, 0.1..=30.0));
            // color control
            eframe::egui::widgets::color_picker::color_edit_button_srgba(
                ui,
                color,
                eframe::egui::widgets::color_picker::Alpha::Opaque,
            );
        });

        // {Documentation} :)
        if ui.button("Exit").clicked() {
            frame.close();
        }
    }
}

impl eframe::App for ScreenPainter {
    fn update(&mut self, ctx: &eframe::egui::Context, frame: &mut eframe::Frame) {
        eframe::egui::CentralPanel::default()
            .frame(eframe::egui::Frame::none().fill(eframe::egui::Color32::from_white_alpha(0)))
            .show(ctx, |ui| self.render_paint_canvas(ui));

        eframe::egui::Window::new("Control panel")
            .resizable(false)
            .show(ctx, |ui| self.render_tooltip(ctx, ui, frame));
    }
    fn clear_color(&self, _visuals: &eframe::egui::style::Visuals) -> [f32; 4] {
        [0.0, 0.0, 0.0, 0.]
    }
}

impl Line {
    fn new(stroke: eframe::egui::Stroke) -> Self {
        Self {
            stroke,
            points: Vec::new(),
        }
    }
}

impl Default for ScreenPainter {
    fn default() -> Self {
        Self {
            canvas: Default::default(),
            recently_deleted: Vec::new(),
            current_stroke: eframe::egui::Stroke::new(
                3.0,
                eframe::egui::Color32::from_rgb(25, 200, 100),
            ),
            currently_drawing: false,
        }
    }
}
