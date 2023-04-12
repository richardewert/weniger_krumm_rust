use crate::Node;
use std::io::Write;
use clap::Parser;
use draw::*;
use log::{debug, info};
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::path::PathBuf;
use std::fs::OpenOptions;

#[derive(Parser, Debug)]
#[command(
    author = "Richard Ewert",
    version,
    about = None,
    long_about = "Lösung zur Aufgabe 1, der zweiten Runde des 41. Bundeswettbewerb Informatik `Weniger Krumme Touren` von Richard Ewert",
)]

struct Args {
    #[arg(short, long)]
    path: PathBuf,
    max_iterations: u64,
}

pub fn get_input() -> (String, u64, String) {
    let args = Args::parse();

    let path = Path::new(&args.path);
    let path_name: Vec<&str> = path.to_str().unwrap().split(&['.', '/']).collect();
    let name = path_name[path_name.len() - 2];
    let display = path.display().to_string();

    let mut file = match File::open(path) {
        Err(why) => panic!("Konnt Pfad nicht öffnen {}: {}", display, why),
        Ok(file) => file,
    };

    let mut s = String::new();
    match file.read_to_string(&mut s) {
        Err(why) => panic!("Konnte {} nicht lesen: {}", display, why),
        Ok(_) => (s, args.max_iterations, name.to_owned()),
    }
}

pub fn read_nodes() -> (Vec<Node>, u64, String) {
    let (input, duration, name) = get_input();

    let split_coords = input.split('\n');
    let unsplit_corrds: Vec<&str> = split_coords.collect();

    let mut nodes: Vec<Node> = vec![];
    for coord in unsplit_corrds.iter() {
        if !coord.is_empty() {
            let split = coord.split(' ');
            let vec: Vec<&str> = split.collect();
            nodes.push(Node {
                x: vec[0].parse().unwrap(),
                y: vec[1].parse().unwrap(),
            })
        }
    }
    info!("Loaded {} nodes", nodes.len());
    debug!("Loaded nodes:\n{:?}", nodes);
    (nodes, duration, name)
}

pub fn render(nodes: &Vec<Node>, solution: &Vec<Node>, length: f32, input_file_name: String) {
    let name = format!("output_{:?}_{:?}", input_file_name, length);

    let place = PathBuf::from("./outputs/txt/").join(format!("{}.txt", name));
    info!("Rendered solution to: {:?}", place);

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(place.clone())
        .unwrap_or_else(|_e| panic!("Couldn't save to: {:?}", place));
    let size_x = 1080;
    let size_y = 720;
    let mut canvas = Canvas::new(size_x, size_y);

    let center_x = (size_x as f32) / 2f32;
    let center_y = (size_y as f32) / 2f32;
    for node in nodes.iter() {
        if let Err(e) = writeln!(file, "{} {}", node.x, node.y) {
            eprintln!("Couldn't write to file: {}", e);
        }

        let circle = Drawing::new()
            .with_shape(Shape::Circle { radius: 5 })
            .with_xy(node.x + center_x, node.y + center_y)
            .with_style(Style::stroked(2, Color::black()));
        canvas.display_list.add(circle);
    }

    for i in 0..nodes.len() {
        if i < solution.len() - 1 {
            let matching = solution[i + 1];
            let color = 255 / solution.len() * i;
            let line = Drawing::new()
                .with_shape( 
                    LineBuilder::new(solution[i].x + center_x, solution[i].y + center_y)
                        .line_to(matching.x + center_x, matching.y + center_y)
                        .build(),
                )
                .with_style(Style::stroked(
                    2,
                    RGB {
                        r: color as u8,
                        g: 100,
                        b: color as u8,
                    },
                ));
            canvas.display_list.add(line);
        }
    }
    
    let binding = PathBuf::from("./outputs/svg/").join(format!("{}.svg", name));
    let place = binding.to_str().unwrap();
    render::save(&canvas, place.clone(), SvgRenderer::new()).expect("Failed to save");
    info!("Rendered image to: {:?}", place);
}
