use std::io::prelude::*;
use std::path::Path;
use std::fs::File;
use std::path::PathBuf;
use std::time::Instant;
use clap::Parser;
use draw::*;
use node_mod::Node;
use log::{debug, info, warn, error};

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

pub mod node_mod {
    use libm::acosf;
    use std::f64::consts::PI;

    #[derive(Copy, Clone, Debug)]
    pub struct Node {
        pub x: f32,
        pub y: f32,
    }

    impl Node {
        pub fn eq(&self, other: &Node) -> bool {
            return self.x == other.x && self.y == other.y;
        }

        fn pow(&self, pow: i32) -> Node {
            return Node {
                x: self.x.powi(pow),
                y: self.y.powi(pow),
            };
        }

        fn sub(&self, other: &Node) -> Node{
            return Node {
                x: self.x - other.x,
                y: self.y - other.y,
            }
        }

        fn added(&self) -> f32 {
            return self.x + self.y;
        }

        pub fn distance(&self, other: &Node) -> f32 {
            let mut val = self.sub(other);
            val = val.pow(2);
            let distance = val.added();
            return distance.sqrt();
        }

        pub fn angle(&self, one: &Node, other: &Node) -> f64 {
            let gegenkathete = one.distance(other);
            let ankathete = self.distance(one);
            let hypothenuse = self.distance(other);

            let cos_angle = (ankathete.powi(2) + hypothenuse.powi(2) - gegenkathete.powi(2)) / (2f32 * ankathete * hypothenuse);

            let angle = acosf(cos_angle);

            let angle_degrees: f64 = <f32 as Into<f64>>::into(angle) * 180f64 / PI;

            return (angle_degrees * 1000f64).round() / 1000f64;
        }
    }
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

fn read_nodes() -> Vec<Node> {
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
    nodes
}

fn render(nodes: &Vec<Node>, solution: &Vec<Node>) {
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
}

fn main() {
    env_logger::init();
    let total_time = Instant::now();
    let nodes = read_nodes();
    debug!("The following nodes were loaded:\n{:?}", nodes);

    let compute_render_time = Instant::now();
    solve();

    let render_time = Instant::now();
    render(&nodes, &nodes);
    let total = total_time.elapsed().as_micros();
    info!( "
=============Time=============
read: ca. {:?}%
compute: ca. {:?}%
render: ca. {:?}%
---------------
total: {:?} mikro seconds",
            (total - compute_render_time.elapsed().as_micros()) * 100 / total,
            (compute_render_time.elapsed().as_micros() -
            render_time.elapsed().as_micros()) * 100 / total,
            render_time.elapsed().as_micros() * 100 / total,
            total
    );
}
