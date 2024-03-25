
use eframe::egui; // This re-exports `egui`, making it available for use
//use egui_plot::{Line, Plot, PlotImage, PlotPoint, PlotPoints, PlotResponse, Points};
use egui_plot::{Line, Plot, PlotPoints};


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
            
            let plot = Plot::new("my_plot")
                .view_aspect(2.0)
                .allow_drag(false)
                .allow_scroll(false)
                .include_x(self.env_domain[0])
                .include_x(self.env_domain[1])
                .include_y(self.env_range[0])
                .include_y(self.env_range[1]);
            
            let plot_response = plot.show(ui, |plot_ui| { 
                
                // let envelope = self.envelope.clone(); // cant remember why but you can't use it directly
                let mut sorted_envelope = self.envelope.clone();
                sorted_envelope.sort_by(|a, b| a[0].partial_cmp(&b[0]).unwrap());
                plot_ui.line(Line::new(PlotPoints::from(sorted_envelope)).name("env"));

                // sine wave
                let sin: PlotPoints = (0..1000).map(|i| {
                    let x = i as f64 * 0.001 * (self.env_domain[1] - self.env_domain[0]) + self.env_domain[0];
                    [x, { (x*(self.slider_val)).sin() * self.get_y(x) }]

                }).collect();
                let sin_line = Line::new(sin);
                plot_ui.line(sin_line);


                // hovering ///
                if let Some(pointer_pos) = plot_ui.pointer_coordinate() {
                    self.plot_pointer_pos = Some([pointer_pos.x, pointer_pos.y]);
                    self.plot_selected_point = None;

                    let threshold = 0.1; // Define a suitable threshold for closeness
                    for (i, point) in self.envelope.iter().enumerate() {
                        let distance = ((pointer_pos.x - point[0]).powi(2) + (pointer_pos.y - point[1]).powi(2)).sqrt();
                        if distance < threshold {
                            self.plot_selected_point = Some(i);
                        }
                    }
                }
                else {
                    self.plot_pointer_pos = None;
                    
                }

            });


            // Check if the plot was clicked
            if plot_response.response.drag_started() {
                if let Some(plot_pointer_pos) = self.plot_pointer_pos {
                    // Convert hover position to plot coordinates
                    // Now, you can check if this click was close to any of your points
                    let threshold = 0.1; // Define a suitable threshold for closeness
                    for (i, point) in self.envelope.iter().enumerate() {
                        let distance = ((plot_pointer_pos[0] - point[0]).powi(2) + (plot_pointer_pos[1] - point[1]).powi(2)).sqrt();
                        if distance < threshold {
                            
                            self.print_label = format!("Clicked near point {} at plot coordinates: {:?}", i, point);
                            println!("Clicked near point {} at plot coordinates: {:?}", i, point);
                            break; // Found a close point, exit the loop
                        }
                    }
                }
            }
            else if plot_response.response.dragged() {
                if let Some(plot_pointer_pos) = self.plot_pointer_pos {
                    if let Some(selected_point) = self.plot_selected_point {
                        //println!("dragging point {} to {} {}", selected_point, plot_pointer_pos[0], plot_pointer_pos[1]);
                        self.envelope[selected_point] = plot_pointer_pos.clone();
                    }
                }    
            }
            else if plot_response.response.drag_released() {
                self.plot_pointer_pos = None;
            }
            

            if plot_response.response.clicked_by( egui::PointerButton::Secondary){
                
                if let Some(selected_point) = self.plot_selected_point {
                    println!("attempted delete point {}", selected_point);
                    self.envelope.remove(selected_point);
                }
            }

            // if plot_response.response.drag_released() {
            //     self.plot_selected_point = None;
            // }
            
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
