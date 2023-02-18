use std::f32::MAX;
use std::time::Instant;
use std::cmp::Ordering;
mod input_output_mod;
mod node_mod;
use node_mod::Node;
use input_output_mod::{render, read_nodes, give_status_info};
use log::{debug, info};

#[derive(Debug, Clone)]
struct Task {
    path: Vec<usize>,
}

fn indices_to_nodes(nodes: Vec<Node>, 
                    indices_path: &Vec<usize>) -> Vec<Node> {
    let mut node_path: Vec<Node> = vec![];
    for i in indices_path {
        node_path.push(nodes[*i]);
    }
    node_path
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

fn sort_tasks(  tasks: &mut Vec<Task>, 
                distances: &Vec<Vec<f32>>, 
                sort_by_last: bool) {
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
fn generate_start_tasks(nodes: &Vec<Node>, 
                        angles: &Vec<Vec<Vec<usize>>>, 
                        distances: &Vec<Vec<f32>>) -> Vec<Task> {
    let mut tasks: Vec<Task> = vec![];
    for (first_node_index, second_node_indices) 
            in angles.iter().enumerate() {
        for (second_node_index, third_node_indices) 
                in second_node_indices.iter().enumerate() {
            for valid_third_node_index in third_node_indices.iter() {
                if first_node_index != second_node_index &&
                    second_node_index != *valid_third_node_index &&
                    first_node_index != *valid_third_node_index 
                {
                    let path = vec![first_node_index, 
                                    second_node_index, 
                                    *valid_third_node_index];
                    let mut free: Vec<usize> = (0..nodes.len()).collect();
                    free.retain(|x| !path.contains(x));

                    tasks.push(Task { path });
                }
            }
        }
    }
    sort_tasks(&mut tasks, &distances, false);
    info!("Generated {} start tasks", tasks.len());
    debug!("Start tasks: {:?}", tasks);
    tasks
}

fn calc_angles_distances(nodes: &Vec<Node>) -> (Vec<Vec<Vec<usize>>>, 
                                                Vec<Vec<f32>>) { 
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

fn get_tasks(   path: Vec<usize>, 
                angles: &Vec<Vec<Vec<usize>>>, 
                distances: &Vec<Vec<f32>>) -> Vec<Task> {
    let mut potential_options: Vec<usize> = 
        angles[path[path.len() - 2]][path[path.len() - 1]].clone();
    potential_options.retain(|potential_option| {
        return !path.contains(potential_option)
    });
    let mut next_tasks: Vec<Task> = vec![]; 
    for node_i in potential_options.iter() {
        let mut new_path = path.clone();
        new_path.push(*node_i);
        next_tasks.push(Task { path: new_path});
    }
    sort_tasks(&mut next_tasks, &distances, true);
    next_tasks
}

fn solve(   nodes: Vec<Node>, 
            angles: &Vec<Vec<Vec<usize>>>, 
            distances: &Vec<Vec<f32>>) -> Option<Vec<Node>> {
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
                info!("\nSolution Nr. {:?}:\n    Solution length: {:?}\n    {:?} ", 
                    solution_paths.len(), new_len, new);
                let node_path = indices_to_nodes(nodes.clone(), &shortest);
                render(&nodes, &node_path);
            } else {
                debug!("Solution Nr. {:?}: \nSolution length: {:?} \n{:?} ", 
                    solution_paths.len(), new_len, new);
            }
        }
        let mut tasks = get_tasks(task.path, &angles, &distances);
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
    println!("started");

    let (angles, distances) = calc_angles_distances(&nodes);
    let solution: Vec<Node> = match solve(nodes.clone(), &angles, &distances) {
        Some(x) => x,
        _ => {println!("No solution found."); return;},
    };

    render(&nodes, &solution);
}
