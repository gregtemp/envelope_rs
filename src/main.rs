
use eframe::egui; // This re-exports `egui`, making it available for use
//use egui_plot::{Line, Plot, PlotImage, PlotPoint, PlotPoints, PlotResponse, Points};
use egui_plot::{Line, Plot, PlotPoints};
use std::f64::consts::{PI, TAU, E};



fn main() {
    let options = eframe::NativeOptions::default();
    
    
    eframe::run_native(
        "Sick fuckin app bro",
        options,
        Box::new(|_cc| {
            let mut app = MyApp::default();
            app.setup();
            Box::new(app)            
        }),
    ).expect("failed to make window");
}

#[derive(Default)]
struct MyApp {
    slider_val: f64,
    // declare envelope at the beginning
    envelope: Vec<[f64; 2]>,
    env_domain: [f64; 2],
    env_range: [f64; 2],
    lines: Vec<LineSeg>,
    print_label: String,
    plot_pointer_pos: Option<[f64; 2]>,
    plot_selected_point: Option<usize>,
    point_dragging_lock: bool,
}


struct LineSeg {
    point1: [f64; 2],
    point2: [f64; 2],
    slope: f64,
    offset: f64,
}

impl MyApp {
    fn setup(&mut self) {
        self.slider_val = 40.0;
        self.envelope = vec![[0.0, 0.0], [0.1, 1.0], [0.3, 0.5], [0.6, 0.5], [1.0, 0.0]];
        self.env_domain = [0.0, 1.0];
        self.env_range = [-1.0, 1.0];
        self.print_label = format!("Nothing to print!");
        self.plot_selected_point = None;
        self.point_dragging_lock = false;
        self.update_lines();
        // Set up other fields or perform initial calculations...
    }

    fn update_lines(&mut self) {
        self.lines.clear();
        let mut sorted_envelope = self.envelope.clone();
        sorted_envelope.sort_by(|a, b| a[0].partial_cmp(&b[0]).unwrap());
        for i in 0..sorted_envelope.len()-1 {
            let point1 = sorted_envelope[i];
            let point2 = sorted_envelope[i+1];
            let slope = (point2[1] - point1[1]) / (point2[0] - point1[0]);
            let offset = point1[1] - slope * point1[0];
            self.lines.push(LineSeg { point1, point2, slope, offset });
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

            self.print_label = format!("Selected point {:?} , drag_lock status {}", self.plot_selected_point, self.point_dragging_lock);
            
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
                let sin: PlotPoints = (0..1000).map(|i| {
                    let x = i as f64 * 0.001 * (self.env_domain[1] - self.env_domain[0]) + self.env_domain[0];
                    [x, { (x*(self.slider_val)).sin() * self.get_y(x) }]

                }).collect();
                let sin_line = Line::new(sin);
                plot_ui.line(sin_line);

                if let Some(sel_pt_idx) = self.plot_selected_point {
                    let sel_point_display: PlotPoints = (0..=4).map(|i| {
                        let t = ((i as f64)/4.0) * TAU;
                        [(t.cos() * 0.01) + self.envelope[sel_pt_idx][0], (t.sin() * 0.04) + self.envelope[sel_pt_idx][1]]
                    }).collect();
                    plot_ui.line(Line::new(sel_point_display));
                }
                

                

                // Hovering
                // Also handles check distance to env points
                if let Some(pointer_pos) = plot_ui.pointer_coordinate() {
                    self.plot_pointer_pos = Some([pointer_pos.x, pointer_pos.y]);
                    //self.plot_selected_point = None;
                    
                    let threshold = 0.05; // Define a suitable threshold for closeness
                    for (i, point) in self.envelope.iter().enumerate() {
                        if self.point_dragging_lock == false {
                            self.plot_selected_point = None;
                            let distance = ((pointer_pos.x - point[0]).powi(2) + (pointer_pos.y - point[1]).powi(2)).sqrt();
                            if distance < threshold {
                                self.plot_selected_point = Some(i);
                                break;
                            }
                        }
                    }
                }
            });


            
            if plot_response.response.drag_started() {                                      // drag started
                self.point_dragging_lock = true;
            }
            else if plot_response.response.dragged() {                                      // dragging
                if let Some(plot_pointer_pos) = self.plot_pointer_pos {
                    if let Some(selected_point) = self.plot_selected_point {
                        //println!("dragging point {} to {} {}", selected_point, plot_pointer_pos[0], plot_pointer_pos[1]);
                        self.envelope[selected_point] = plot_pointer_pos.clone();
                    }
                }    
            }
            else if plot_response.response.drag_released() {                                // drag ended
                self.point_dragging_lock = false;
                self.plot_pointer_pos = None;
            }
            

            if plot_response.response.clicked_by( egui::PointerButton::Secondary){
                
                if let Some(selected_point) = self.plot_selected_point {
                    println!("attempted delete point {}", selected_point);
                    self.envelope.remove(selected_point);
                    self.plot_selected_point = None;
                }
            }

            if plot_response.response.clicked() {
                if self.plot_selected_point == None {
                    if let Some(plot_pointer_pos) = self.plot_pointer_pos {
                        self.envelope.push(plot_pointer_pos);
                    }
                }
            }
            
            
            

            ui.label(&self.print_label);
            
        });
    }
}
