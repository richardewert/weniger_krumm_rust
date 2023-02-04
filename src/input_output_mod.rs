use crate::Node;
use std::time::Instant;
use draw::*;
use std::path::Path;
use std::io::Read;
use std::path::PathBuf;
use clap::Parser;
use std::fs::File;
use log::{debug, info};

#[derive(Parser, Debug)]
#[command(
    author = "Richard Ewert",
    version,
    about = None, 
    long_about = 
        "Lösung zur Aufgabe 1, der zweiten Runde des 41. Bundeswettbewerb Informatik `Weniger Krumme Touren` von Richard Ewert"
)]

struct Args {
    #[arg(short, long)]
    path: PathBuf
}

fn get_input() -> String {
    let args = Args::parse();

    let path = Path::new(&args.path);
    let display = path.display();     
    
    let mut file = match File::open(&path) {
        Err(why) => panic!("Konnt Pfad nicht öffnen {}: {}", display, why),
        Ok(file) => file,
    };

    let mut s = String::new();
    match file.read_to_string(&mut s) {
        Err(why) => panic!("Konnte {} nicht lesen: {}", display, why),
        Ok(_) => return s,
    }
}

pub fn read_nodes() -> Vec<Node> {
    let input = get_input();
    
    let split_coords = input.split("\n");
    let unsplit_corrds: Vec<&str> = split_coords.collect();

    let mut nodes: Vec<Node> = vec![];
    for coord in unsplit_corrds.iter() {
        if !coord.is_empty() {
            let split = coord.split(" ");
            let vec: Vec<&str> = split.collect();
            nodes.push(Node{
                x: vec[0].parse().unwrap(),
                y: vec[1].parse().unwrap(),
            })
        }
    }
    info!("Loaded {} nodes", nodes.len());
    debug!("Loaded nodes:\n{:?}", nodes);
    nodes
}

pub fn render(nodes: &Vec<Node>, solution: &Vec<Node>) {
    let size_x = 1080;
    let size_y = 720;
    let mut canvas = Canvas::new(size_x, size_y);

    let center_x = (size_x as f32)/2f32;
    let center_y = (size_y as f32)/2f32;
    for node in nodes.iter() {
        let circle = Drawing::new()
            .with_shape(Shape::Circle {
                radius: 5,
            })
            .with_xy(node.x + center_x, node.y + center_y)
            .with_style(Style::stroked(2, Color::black()));
        canvas.display_list.add(circle);
    }

    for i in 0..nodes.len() {
        if i < solution.len() - 1 {
            let matching = solution[i + 1];
            let color = 255/solution.len()*i;
            let line = Drawing::new()
                .with_shape(
                    LineBuilder::new(solution[i].x + center_x, solution[i].y + center_y)
                    .line_to(matching.x + center_x, matching.y + center_y)
                    .build())
                .with_style(Style::stroked(2, RGB { r: color as u8, g: 100, b: color as u8 }));
            canvas.display_list.add(line);
        }
    }
    
    render::save(
        &canvas,
        "output.svg",
        SvgRenderer::new(),
    ).expect("Failed to save");
    info!("Rendered image");
}

pub fn give_status_info(iteration: i64, timer: Instant, mut last_time: f32) -> f32 {
    let update_frequency = 1000000;
    if iteration % update_frequency == 0 {
        let average_iterations = iteration as f32 / timer.elapsed().as_secs_f32();
        let update_time = timer.elapsed().as_secs_f32() - last_time;
        let iterations = update_frequency as f64 / update_time as f64 ;
        info!("
            Average iterations per second:  {:?}
            Iterations per second           {:?}    
            Time since last update:         {:?}",
                average_iterations.round() as u32,
            iterations.round() as u32,
            update_time);
        last_time = timer.elapsed().as_secs_f32(); 
    }
    last_time
}

