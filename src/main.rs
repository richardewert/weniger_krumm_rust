mod input_output_mod;
mod node_mod;

use input_output_mod::{read_nodes, render};
use log::{debug, info};
use node_mod::Node;
use std::cmp::Ordering;
use std::f32::MAX;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use std::thread::available_parallelism;
use std::thread::{self, JoinHandle};

fn path_len(path: &Vec<usize>, distances: &[Vec<f32>]) -> f32 {
    let mut distance: f32 = 0f32;
    for (i, _node_index) in path.iter().enumerate() {
        if i < path.len() - 1 {
            distance += distances[path[i]][path[i + 1]];
        }
    }
    distance
}

fn calc_angles_distances(nodes: &Vec<Node>) -> 
        (Vec<Vec<Vec<usize>>>, Vec<Vec<f32>>) {
    // 2d Vector, um alle Distanzen zwischen 2 Nodes zu speichern
    let mut distances: Vec<Vec<f32>> = vec![];
    // 3d Vector, um alle Winkel zwischen 3 Nodes zu speichern
    let mut angles: Vec<Vec<Vec<usize>>> = vec![];
    // Debug Variable, um Menge von einträgen zu zählen
    let mut cache_entries = 0;
    for (start_node_index, start_node) in nodes.iter().enumerate() {
        distances.push(vec![]);
        angles.push(vec![]);
        for (main_node_index, main_node) in nodes.iter().enumerate() {
            distances[start_node_index].push(start_node.distance(main_node));
            angles[start_node_index].push(vec![]);
            for (end_node_index, end_node) in nodes.iter().enumerate() {
                let angle = main_node.angle(start_node, end_node);
                debug!(
                    "Angle between {:?}, {:?}, {:?} : {:?}", 
                    start_node_index, 
                    main_node_index, 
                    end_node_index, 
                    angle);
                if 90f32 <= angle {
                    angles[start_node_index][main_node_index].push(end_node_index);
                    cache_entries += 1;
                }
            }
            angles[start_node_index][main_node_index].sort_by(|a, b| {
                let val_a = main_node.distance(&nodes[*a]);
                let val_b = main_node.distance(&nodes[*b]);
                if val_a < val_b {
                    Ordering::Less
                } else if val_a == val_b {
                    Ordering::Equal
                } else {
                    Ordering::Greater
                }
            });
        }
    }
    info!("Cache entries count: {}", cache_entries);
    debug!("Cached entries: {:?}", angles);
    debug!("Cached distances: {:?}", distances);
    (angles, distances)
}

fn sort_paths(tasks: &mut Vec<Vec<usize>>, distances: &Vec<Vec<f32>>) {
    tasks.sort_by(|a, b| {
        let val_a = path_len(&a, distances);
        let val_b = path_len(&b, distances);
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
fn generate_start_paths(
    angles: &Vec<Vec<Vec<usize>>>,
    distances: &Vec<Vec<f32>>,
) -> Vec<Vec<usize>> {
    let mut paths: Vec<Vec<usize>> = vec![];
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
                    paths.push(path);
                }
            }
        }
    }
    sort_paths(&mut paths, distances);
    info!("Generated {} start paths", paths.len());
    debug!("Start paths: {:?}", paths);
    paths
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

fn solve_recursive(
    path: &mut Vec<usize>, 
    path_length: f32,
    nodes: &Vec<Node>, 
    angles: &Vec<Vec<Vec<usize>>>, 
    distances: &Vec<Vec<f32>>, 
    solution: &mut Arc<Mutex<Vec<usize>>>, 
    input_file_name: &String, 
    iterations: &mut u64,
    max_iterations: &u64,
    solution_length: &mut Arc<Mutex<f32>>,
) {
    *iterations += 1;
    let mut sol_len = solution_length.lock().unwrap();
    if *iterations > *max_iterations || path_length >= *sol_len {return};
    if path.len() == distances.len() {
        let mut global_best = solution.lock().unwrap();
        if path_length < path_len(&global_best, distances) || global_best.len() == 0 {
            path.clone_into(&mut global_best);
            render(nodes, &indices_to_nodes(nodes.clone(), &global_best), path_length, input_file_name.clone());
            *sol_len = path_length;
            return;
        }
    }
    drop(sol_len);

    let mut options: Vec<usize> = angles[path[path.len() - 2]][path[path.len() - 1]].clone();
    options.retain(|x| !path.contains(x));

    for i in options {
        path.push(i);
        let add_length = distances[path[path.len() - 2]][path[path.len() - 1]];
        solve_recursive(path, path_length + add_length, nodes, angles, distances, solution, input_file_name, iterations, max_iterations, solution_length); 
        path.pop().unwrap();
    }
}

fn main() {
    env_logger::init();
    // Ließt alle Nodes und die Suchlänge ein
    let (nodes, max_iterations, name) = read_nodes();

    let (angles, distances) = calc_angles_distances(&nodes);

    let generated_paths = generate_start_paths(&angles, &distances);
    let start_paths: Arc<Mutex<Vec<Vec<usize>>>> = Arc::new(Mutex::new(generated_paths.clone()));

    let solution: Arc<Mutex<Vec<usize>>> = Arc::new(Mutex::new(vec![]));
    let solution_length: Arc<Mutex<f32>> = Arc::new(Mutex::new(MAX));
    let done_threads: Arc<Mutex<u32>> = Arc::new(Mutex::new(0));

    let mut handles: Vec<JoinHandle<()>> = vec![];
    let total_threads: usize = available_parallelism().unwrap().into();
    info!("Starting up {:?} threads", total_threads);
    for _i in 0..(total_threads - 1) {
        let total_tasks = generated_paths.len();
        let mut l_solution = Arc::clone(&solution);
        let mut l_solution_length = Arc::clone(&solution_length);
        let l_start_paths = Arc::clone(&start_paths);
        let l_done_threads = Arc::clone(&done_threads);

        let mut l_iterations = 0;

        let l_nodes = nodes.clone();
        let l_angles = angles.clone();
        let l_distances = distances.clone();

        let l_name = name.clone();
        let l_max_iterations = max_iterations.clone();

        let handle = thread::spawn(move || {
            info!("Started thread {:?}", thread::current().id());
            loop {
                let mut todo = l_start_paths.lock().unwrap();
                if let Some(mut l_start_path) = todo.pop() {
                    let thread_number = todo.len();
                    drop(todo);
                    let l_path_length = path_len(&l_start_path, &l_distances);
                    solve_recursive(&mut l_start_path, l_path_length, &l_nodes, &l_angles, &l_distances, &mut l_solution, &l_name, &mut l_iterations, &l_max_iterations, &mut l_solution_length);
                    let mut done = l_done_threads.lock().unwrap();
                    *done += 1;
                    debug!("Thread {:?} finished work on start path with priority {:?}  {:?}/{:?}", thread::current().id(), thread_number, done, total_tasks);
                    drop(done);
                } else {
                    break;
                }
            }
        });

        handles.push(handle);
    }

    while let Some(handle) = handles.pop() {
        handle.join().unwrap();
    }
    info!("First phase done. Starting to swap");


    let final_solution = solution.lock().unwrap();
    // Stellt die gefundene Lösung dar
    if !final_solution.is_empty() {
        render(&nodes, &indices_to_nodes(nodes.clone(), &final_solution), path_len(&final_solution, &distances), name);
    }
}
