
use eframe::egui; // This re-exports `egui`, making it available for use
use egui_plot::{Line, Plot, PlotPoints};


fn main() {
    let options = eframe::NativeOptions::default();
    
    
    eframe::run_native(
        "Sick fuckin app bro",
        options,
        Box::new(|_cc| Box::new(MyApp::default())),
    ).expect("failed to make window");
}

#[derive(Default)]
struct MyApp {
    slider_val: f64,
    // declare graph at the beginning
    graph: Vec<[f64; 2]>,
    lines: Vec<LineSeg>,
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
        let mut sorted_graph = self.graph.clone();
        sorted_graph.sort_by(|a, b| a[0].partial_cmp(&b[0]).unwrap());
        for i in 0..sorted_graph.len()-1 {
            let point1 = sorted_graph[i];
            let point2 = sorted_graph[i+1];
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
            // slider here
            ui.add(egui::Slider::new(&mut self.slider_val, 0.0..=5.0).text("Slider"));
            ui.label(format!("Slider value: {}", self.slider_val));
            
            
            self.graph = vec![[0.0, 0.0], [0.1, 1.0], [2.0, 0.5], [4.0, 0.5], [2.0*3.1428, 0.0]];
            self.update_lines();
            
            let plot = Plot::new("my_plot")
                .view_aspect(2.0)
                .allow_drag(false)
                .allow_scroll(false)
                .include_x(0.0)
                .include_x(2.0 * 3.1428)
                .include_y(-1.0)
                .include_y(1.0);
            
            plot.show(ui, |plot_ui| { 
                
                let showgraph = self.graph.clone();
                plot_ui.line(Line::new(PlotPoints::from(showgraph)).name("env"));

                // sine wave
                let sin: PlotPoints = (0..1000).map(|i| {
                    let x = i as f64 * 0.01;
                    // let gain = self.slider_val as f64;
                    
                    [x, { (x*(1.0 + self.slider_val* 2.0)).sin() * self.get_y(x) }]
                }).collect();
                let sin_line = Line::new(sin);
                plot_ui.line(sin_line);

                // cos wave
                let cos: PlotPoints = (0..1000).map(|i| {
                    let x = i as f64 * 0.01;
                    let gain = self.slider_val as f64;
                    [x, {x.cos() * gain*0.5 }.tanh()]
                }).collect();
                let cos_line = Line::new(cos);
                plot_ui.line(cos_line);




                });
        });
    }
}
