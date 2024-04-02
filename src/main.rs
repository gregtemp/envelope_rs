use eframe::egui; // This re-exports `egui`, making it available for use

//use egui_plot::{Line, Plot, PlotImage, PlotPoint, PlotPoints, PlotResponse, Points};
use egui_plot::{Line, Plot, PlotPoints};
use std::f64::consts::{E, PI, TAU};

fn main() {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Sick fuckin app bro",
        options,
        Box::new(|_cc| {
            let mut app = MyApp::default();
            Box::new(app)
        }),
    )
    .expect("failed to make window");
}

#[derive(Default, Debug)]
pub enum DragState {
    Dragging(usize),
    #[default]
    Released,
}

#[derive(Default, Debug)]
pub enum MouseState {
    /// Near to a point
    Near([f64; 2], usize),
    /// Far from any point
    Far([f64; 2]),
    #[default]
    /// Not even on the canvas
    Gone,
}

impl MouseState {
    fn pos(&self) -> Option<&[f64; 2]> {
        match self {
            MouseState::Near(pos, _) => Some(pos),
            MouseState::Far(pos) => Some(pos),
            MouseState::Gone => None,
        }
    }

    fn near_point(&self) -> Option<usize> {
        match self {
            MouseState::Near(_, point) => Some(*point),
            MouseState::Far(_) => None,
            MouseState::Gone => None,
        }
    }
}

struct MyApp {
    slider_val: f64,
    // declare envelope at the beginning
    envelope: Vec<[f64; 2]>,
    env_domain: [f64; 2],
    env_range: [f64; 2],
    lines: Vec<LineSeg>,
    print_label: String,
    drag_state: DragState,
    mouse_state: MouseState,
}

impl Default for MyApp {
    fn default() -> Self {
        let mut zelf = Self {
            slider_val: 40.0,
            envelope: vec![[0.0, 0.0], [0.1, 1.0], [0.3, 0.5], [0.6, 0.5], [1.0, 0.0]],
            env_domain: [0.0, 1.0],
            env_range: [-1.0, 1.0],
            lines: Default::default(),
            print_label: format!("Nothing to print!"),
            drag_state: Default::default(),
            mouse_state: Default::default(),
        };

        zelf.update_lines();

        zelf
    }
}

struct LineSeg {
    point1: [f64; 2],
    point2: [f64; 2],
    slope: f64,
    offset: f64,
}

impl MyApp {
    fn update_lines(&mut self) {
        self.lines.clear();
        let mut sorted_envelope = self.envelope.clone();
        sorted_envelope.sort_by(|a, b| a[0].partial_cmp(&b[0]).unwrap());
        for i in 0..sorted_envelope.len() - 1 {
            let point1 = sorted_envelope[i];
            let point2 = sorted_envelope[i + 1];
            let slope = (point2[1] - point1[1]) / (point2[0] - point1[0]);
            let offset = point1[1] - slope * point1[0];
            self.lines.push(LineSeg {
                point1,
                point2,
                slope,
                offset,
            });
        }
    }

    fn get_y(&self, x: f64) -> f64 {
        for lineseg in &self.lines {
            if lineseg.point1[0] <= x && lineseg.point2[0] >= x {
                return lineseg.slope * x + lineseg.offset;
            }
        }
        0.0
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label("Hello, this is my app bro!");
            ui.add(egui::Slider::new(&mut self.slider_val, 1.0..=140.0).text("Frequency"));
            ui.label(format!("Slider value: {}", self.slider_val));

            self.update_lines();

            self.print_label = format!("hover: {:?} drag: {:?}", self.mouse_state, self.drag_state);

            let plot = Plot::new("my_plot")
                .view_aspect(2.0)
                .allow_drag(false)
                .allow_scroll(false)
                .include_x(self.env_domain[0])
                .include_x(self.env_domain[1])
                .include_y(self.env_range[0])
                .include_y(self.env_range[1]);

            let plot_response = plot.show(ui, |plot_ui| {
                // Envelope display
                let mut sorted_envelope = self.envelope.clone();
                sorted_envelope.sort_by(|a, b| a[0].partial_cmp(&b[0]).unwrap());
                plot_ui.line(Line::new(PlotPoints::from(sorted_envelope)).name("env"));

                // Sine wave display (and mult by env)
                let sin: PlotPoints = (0..1000)
                    .map(|i| {
                        let x = i as f64 * 0.001 * (self.env_domain[1] - self.env_domain[0])
                            + self.env_domain[0];
                        [x, { (x * (self.slider_val)).sin() * self.get_y(x) }]
                    })
                    .collect();
                let sin_line = Line::new(sin);
                plot_ui.line(sin_line);

                if let Some(sel_pt_idx) = self.mouse_state.near_point() {
                    let sel_point_display: PlotPoints = (0..=4)
                        .map(|i| {
                            let t = ((i as f64) / 4.0) * TAU;
                            [
                                (t.cos() * 0.01) + self.envelope[sel_pt_idx][0],
                                (t.sin() * 0.04) + self.envelope[sel_pt_idx][1],
                            ]
                        })
                        .collect();
                    plot_ui.line(Line::new(sel_point_display));
                }

                // Hovering
                // Also handles gk distance to env points
                if let Some(pointer_pos) = plot_ui.pointer_coordinate() {
                    self.mouse_state = MouseState::Far([pointer_pos.x, pointer_pos.y]);
                    //self.plot_selected_point = None;

                    let threshold = 0.05; // Define a suitable threshold for closeness
                    for (i, point) in self.envelope.iter().enumerate() {
                        // If not dragging but near a point,
                        if let DragState::Released = self.drag_state {
                            let distance = ((pointer_pos.x - point[0]).powi(2)
                                + (pointer_pos.y - point[1]).powi(2))
                            .sqrt();
                            if distance < threshold {
                                self.mouse_state =
                                    MouseState::Near([pointer_pos.x, pointer_pos.y], i);
                                break;
                            }
                        }
                    }
                } else {
                    self.mouse_state = MouseState::Gone;
                }
            });

            if plot_response.response.drag_started() {
                // drag started
                // TODO(emily): Handle if we are not near a point properly
                if let Some(mouse_state) = self.mouse_state.near_point() {
                    self.drag_state = DragState::Dragging(mouse_state);
                }
            } else if plot_response.response.dragged() {
                if let DragState::Dragging(point) = &self.drag_state {
                    if let Some(hover_point) = self.mouse_state.pos() {
                        self.envelope[*point] = *hover_point;
                    }
                }
            } else if plot_response.response.drag_released() {
                // drag ended
                self.drag_state = DragState::Released;
            }

            if plot_response
                .response
                .clicked_by(egui::PointerButton::Secondary)
            {
                if let Some(point) = self.mouse_state.near_point() {
                    self.envelope.remove(point);
                    self.mouse_state = MouseState::Far(*self.mouse_state.pos().unwrap());
                }
            }

            if plot_response.response.clicked() {
                if let MouseState::Far(point) = self.mouse_state {
                    self.envelope.push(point);
                }
            }

            ui.label(&self.print_label);
        });
    }
}
