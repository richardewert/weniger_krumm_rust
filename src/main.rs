use std::{path::Path, f32::MAX};
use std::io::Read;
use std::cmp::Ordering;
use std::fs::File;
use std::path::PathBuf;
use std::time::Instant;
use clap::Parser;
use draw::*;
use node_mod::Node;
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

pub mod node_mod {
    use libm::acosf;
    use std::f32::consts::PI;

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

        pub fn angle(&self, one: &Node, other: &Node) -> f32 {
            let gegenkathete = one.distance(other);
            let ankathete = self.distance(one);
            let hypothenuse = self.distance(other);

            let cos_angle = (ankathete.powi(2) + hypothenuse.powi(2) - gegenkathete.powi(2)) / (2f32 * ankathete * hypothenuse);

            let angle = acosf(cos_angle);

            let angle_degrees: f32 = angle * 180f32 / PI;

            angle_degrees
        }

        pub fn make_key(&self) -> (i32, i32) {
            return (self.x as i32, self.y as i32);
        }
    }
}

#[derive(Debug, Clone)]
struct Task {
    path: Vec<usize>,
    free: Vec<usize>,
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
    info!("Loaded {} nodes", nodes.len());
    debug!("Loaded nodes:\n{:?}", nodes);
    nodes
}

fn indices_to_nodes(nodes: Vec<Node>, indices_path: &Vec<usize>) -> Vec<Node> {
    let mut node_path: Vec<Node> = vec![];
    for i in indices_path {
        node_path.push(nodes[*i]);
    }
    node_path
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
    info!("Rendered image");
}

fn path_len(path: &Vec<usize>, distances: &Vec<Vec<f32>>) -> f32 {
    let mut distance: f32 = 0f32;
    for (i, _node_index) in path.iter().enumerate() {
        if i < path.len() - 1 {
            distance += distances[path[i]][path[i + 1]];
        }
    }
    distance
}

fn sort_tasks(tasks: &mut Vec<Task>, distances: &Vec<Vec<f32>>, sort_by_last: bool) {
    tasks.sort_by(|a, b| {
        let val_a;
        let val_b;
        if sort_by_last {
            val_a = distances[a.path[a.path.len() - 2]][a.path[a.path.len() - 1]];
            val_b = distances[b.path[b.path.len() - 2]][b.path[b.path.len() - 1]];
        } else {
            val_a = path_len(&a.path, distances);
            val_b = path_len(&b.path, distances);
        }
        if val_a > val_b {
            return Ordering::Less;
        } else if val_a == val_b {
            return Ordering::Equal;
        } else {
            return Ordering::Greater;
        }
    });
}

//Choose shortest valid path of 3 nodes to begin from.
fn generate_start_tasks(nodes: &Vec<Node>, angles: &Vec<Vec<Vec<usize>>>, distances: &Vec<Vec<f32>>) -> Vec<Task> {
    let mut tasks: Vec<Task> = vec![];
    for (first_node_index, second_node_indices) in angles.iter().enumerate() {
        for (second_node_index, third_node_indices) in second_node_indices.iter().enumerate() {
            for valid_third_node_index in third_node_indices.iter() {
                if first_node_index != second_node_index &&
                    second_node_index != *valid_third_node_index &&
                    first_node_index != *valid_third_node_index 
                {
                    let path = vec![first_node_index, second_node_index, *valid_third_node_index];
                    let mut free: Vec<usize> = (0..nodes.len()).collect();
                    free.retain(|x| !path.contains(x));

                    tasks.push(Task { path, free });
                }
            }
        }
    }
    sort_tasks(&mut tasks, &distances, false);
    info!("Generated {} start tasks", tasks.len());
    debug!("Start tasks: {:?}", tasks);
    tasks
}

fn get_tasks(path: Vec<usize>, free: Vec<usize>, angles: &Vec<Vec<Vec<usize>>>, distances: &Vec<Vec<f32>>) -> Vec<Task> {
    let mut potential_options: Vec<usize> = angles[path[path.len() - 2]][path[path.len() - 1]].clone();
    potential_options.retain(|potential_option| {return free.contains(potential_option)});
    let mut next_tasks: Vec<Task> = vec![]; 
    for node_i in potential_options.iter() {
        let mut new_free = free.clone();
        let mut new_path = path.clone();
        new_free.retain(|x| {return x != node_i});
        new_path.push(*node_i);
        next_tasks.push(Task { path: new_path, free: new_free });
    }
    sort_tasks(&mut next_tasks, &distances, true);
    next_tasks
}

fn give_status_info(iteration: i64, timer: Instant, mut last_time: f32) -> f32 {
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

fn calc_angles_distances(nodes: &Vec<Node>) -> (Vec<Vec<Vec<usize>>>, Vec<Vec<f32>>) { 
    let mut distances: Vec<Vec<f32>> = vec![];
    let mut angles: Vec<Vec<Vec<usize>>> = vec![];
    let mut cache_entries = 0;
    for (start, start_node) in nodes.iter().enumerate() {
        distances.push(vec![]);
        angles.push(vec![]);
        for (main, main_node) in nodes.iter().enumerate() {
            distances[start].push(start_node.distance(main_node));
            angles[start].push(vec![]);
            for (end, end_node) in nodes.iter().enumerate() {
                let angle = main_node.angle(start_node, end_node);
                debug!("Angle between {start}, {main}, {end} : {angle}");
                if 90f32 <= angle {
                    angles[start][main].push(end);
                    cache_entries += 1;
                }
            }
        }
    }
    info!("Cache entries count: {}", cache_entries);
    debug!("Cached entries: {:?}", angles);
    debug!("Cached distances: {:?}", distances);
    return (angles, distances);
}

fn solve(nodes: Vec<Node>, angles: &Vec<Vec<Vec<usize>>>, distances: &Vec<Vec<f32>>) -> Option<Vec<Node>> {
    let mut task_queue: Vec<Task> = generate_start_tasks(&nodes, angles, distances);

    let timer = Instant::now();
    let mut iteration = 0i64;
    let mut last_time = timer.elapsed().as_secs_f32();

    let mut solution_paths: Vec<Vec<usize>> = vec![];
    let mut shortest: Vec<usize> = vec![];
    let mut shortest_length: f32 = MAX;
    while !task_queue.is_empty() {
        last_time = give_status_info(iteration, timer, last_time);
        let task = task_queue.pop().unwrap();
        if task.path.len() == nodes.len() {
            solution_paths.push(task.path.clone());
            let new = solution_paths.last().unwrap();
            let new_len = path_len(new, distances);
            if shortest_length > new_len {
                shortest_length = new_len.clone();
                shortest = new.clone();
                info!("\nSolution Nr. {:?}: \n    Solution length: {:?} \n    {:?} ", solution_paths.len(), new_len, new);
                let node_path = indices_to_nodes(nodes.clone(), &shortest);
                render(&nodes, &node_path);
            } else {
                debug!("Solution Nr. {:?}: \nSolution length: {:?} \n{:?} ", solution_paths.len(), new_len, new);
            }
        }
        let mut tasks = get_tasks(task.path, task.free, &angles, &distances);
        task_queue.append(&mut tasks);
        iteration += 1;
    }
    if !solution_paths.is_empty() {
        return Some(indices_to_nodes(nodes, &shortest)); 
    }
    return None;
}

fn main() {
    env_logger::init();
    let nodes = read_nodes(); 

    let (angles, distances) = calc_angles_distances(&nodes);
    let solution: Vec<Node> = match solve(nodes.clone(), &angles, &distances) {
        Some(x) => x,
        _ => {println!("No solution found."); return;},
    };

    render(&nodes, &solution);
}
