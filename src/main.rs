mod input_output_mod;
mod node_mod;

use input_output_mod::{read_nodes, render};
use log::{debug, info};
use node_mod::Node;
use std::cmp::Ordering;
use std::f32::MAX;
use std::time::Instant;

#[derive(Debug, Clone)]
struct Task {
    path: Vec<usize>,
    free: Vec<usize>,
}

fn path_len(path: &Vec<usize>, distances: &[Vec<f32>]) -> f32 {
    let mut distance: f32 = 0f32;
    for (i, _node_index) in path.iter().enumerate() {
        if i < path.len() - 1 {
            distance += distances[path[i]][path[i + 1]];
        }
    }
    distance
}

fn calc_angles_distances(nodes: &[Node]) -> 
        (Vec<Vec<Vec<usize>>>, Vec<Vec<f32>>) {
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
    (angles, distances)
}

fn sort_tasks(tasks: &mut [Task], distances: &[Vec<f32>], sort_by_last: bool) {
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
            Ordering::Less
        } else if val_a == val_b {
            Ordering::Equal
        } else {
            Ordering::Greater
        }
    });
}

//Choose shortest valid path of 3 nodes to begin from.
fn generate_start_tasks(
    nodes: &Vec<Node>,
    angles: &[Vec<Vec<usize>>],
    distances: &[Vec<f32>],
) -> Vec<Task> {
    let mut tasks: Vec<Task> = vec![];
    for (first_node_index, second_node_indices) in 
            angles.iter().enumerate() {
        for (second_node_index, third_node_indices) in 
                second_node_indices.iter().enumerate() {
            for valid_third_node_index in 
                    third_node_indices.iter() {
                if first_node_index != second_node_index
                    && second_node_index != *valid_third_node_index
                    && first_node_index != *valid_third_node_index
                {
                    let path = vec![
                        first_node_index, 
                        second_node_index, 
                        *valid_third_node_index
                    ];
                    let mut free: Vec<usize> = (0..nodes.len()).collect();
                    free.retain(|x| !path.contains(x));

                    tasks.push(Task { path, free });
                }
            }
        }
    }
    sort_tasks(&mut tasks, distances, false);
    info!("Generated {} start tasks", tasks.len());
    debug!("Start tasks: {:?}", tasks);
    tasks
}

fn get_tasks(
    path: Vec<usize>,
    free: Vec<usize>,
    angles: &[Vec<Vec<usize>>],
    distances: &[Vec<f32>],
) -> Vec<Task> {
    let mut potential_options: Vec<usize> =
        angles[path[path.len() - 2]][path[path.len() - 1]].clone();

    potential_options.retain(
        |potential_option| free.contains(potential_option)
    );
    let mut next_tasks: Vec<Task> = vec![];
    for node_i in potential_options.iter() {
        let mut new_free = free.clone();
        let mut new_path = path.clone();
        new_free.retain(|x| x != node_i);
        new_path.push(*node_i);
        next_tasks.push(Task {
            path: new_path,
            free: new_free,
        });
    }
    sort_tasks(&mut next_tasks, distances, true);
    next_tasks
}

fn indices_to_nodes(
        nodes: Vec<Node>, 
        indices_path: &Vec<usize>) -> Vec<Node> {
    let mut node_path: Vec<Node> = vec![];
    for i in indices_path {
        node_path.push(nodes[*i]);
    }
    node_path
}

fn solve(nodes: Vec<Node>) -> Option<Vec<Node>> {
    let (angles, distances) = 
        calc_angles_distances(&nodes);

    let mut task_queue: Vec<Task> = 
        generate_start_tasks(&nodes, &angles, &distances);

    let mut solution_paths: Vec<Vec<usize>> = vec![];

    let timer = Instant::now();
    let mut iteration = 0i64;
    let mut last_time = timer.elapsed().as_secs_f32();
    let update_frequency = 1000000;
    let mut shortest: Vec<usize> = vec![];
    let mut shortest_length: f32 = MAX;

    while !task_queue.is_empty() {
        //debug!("task_queue: {:?}", task_queue);
        if iteration % update_frequency == 0 {
            let average_iterations = 
                iteration as f32 / timer.elapsed().as_secs_f32();

            let update_time = 
                timer.elapsed().as_secs_f32() - last_time;

            let iterations = 
                update_frequency as f64 / update_time as f64;

            info!(
                "
                Average iterations per second:  {:?}
                Iterations per second           {:?}    
                Time since last update:         {:?}",
                average_iterations.round() as u32,
                iterations.round() as u32,
                update_time
            );
            last_time = timer.elapsed().as_secs_f32();
        }

        let task = task_queue.pop().unwrap();
        if task.path.len() == nodes.len() {
            solution_paths.push(task.path.clone());
            let new = solution_paths.last().unwrap();
            let new_len = path_len(new, &distances);
            if shortest_length > new_len {
                shortest_length = new_len;
                shortest = new.clone();

                info!(
                    "\nSolution Nr. {:?}:\n    Solution length: {:?}\n    {:?}",
                    solution_paths.len(),
                    new_len,
                    new
                );

                let node_path = indices_to_nodes(nodes.clone(), &shortest);
                render(&nodes, &node_path);
            } else {
                debug!(
                    "Solution Nr. {:?}: \nSolution length: {:?} \n{:?} ",
                    solution_paths.len(),
                    new_len,
                    new
                );
            }
        }
        let mut tasks = get_tasks(task.path, task.free, &angles, &distances);
        task_queue.append(&mut tasks);
        iteration += 1;
    }
    if !solution_paths.is_empty() {
        Some(indices_to_nodes(nodes, &shortest))
    } else {
        None
    }
}

fn main() {
    env_logger::init();
    let total_time = Instant::now();
    let nodes = read_nodes();

    let compute_render_time = Instant::now();

    let solution: Vec<Node> = match solve(nodes.clone()) {
        Some(x) => x,
        _ => {
            println!("No solution found.");
            return;
        }
    };

    let render_time = Instant::now();
    render(&nodes, &solution);
    let total = total_time.elapsed().as_micros();

    info!("
=============Time=============
read: ca. {:?}%
compute: ca. {:?}%
render: ca. {:?}%
---------------
total: {:?} mikro seconds",
        (total - compute_render_time.elapsed().as_micros()) 
            * 100 / total,
        (compute_render_time.elapsed().as_micros() - render_time.elapsed().as_micros()) 
            * 100 / total,
        render_time.elapsed().as_micros() 
            * 100 / total,
        total
    );
}
