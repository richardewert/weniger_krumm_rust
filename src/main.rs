mod input_output_mod;
mod node_mod;

use input_output_mod::{read_nodes, render};
use log::{debug, info};
use node_mod::Node;
use std::cmp::Ordering;
use std::f32::MAX;
use std::sync::{Arc, Mutex};
use std::time::Instant;
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

fn calc_angles_distances(nodes: &[Node]) -> 
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
        }
    }
    info!("Cache entries count: {}", cache_entries);
    debug!("Cached entries: {:?}", angles);
    debug!("Cached distances: {:?}", distances);
    (angles, distances)
}

fn sort_paths(tasks: &mut Vec<Vec<usize>>, distances: &Vec<Vec<f32>>, sort_by_last: bool) {
    tasks.sort_by(|a, b| {
        let val_a;
        let val_b;
        if sort_by_last {
            val_a = distances[a[a.len() - 2]][a[a.len() - 1]];
            val_b = distances[b[b.len() - 2]][b[b.len() - 1]];
        } else {
            val_a = path_len(&a, distances);
            val_b = path_len(&b, distances);
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
    sort_paths(&mut paths, distances, false);
    paths.reverse();
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
    nodes: &Vec<Node>, 
    angles: &Vec<Vec<Vec<usize>>>, 
    distances: &Vec<Vec<f32>>, 
    solution: &mut Vec<usize>, 
    input_file_name: &String, 
    iterations: &mut u64,
    max_iterations: &u64,
) {
    *iterations += 1;
    if *iterations > *max_iterations {return};
    if path.len() == distances.len() {
        let length = path_len(path, distances);
        if length < path_len(solution, distances) || solution.len() == 0 {
            path.clone_into(solution);
            render(nodes, &indices_to_nodes(nodes.clone(), solution), length, input_file_name.clone())
        }
    }

    let mut options: Vec<usize> = angles[path[path.len() - 2]][path[path.len() - 1]].clone();
    options.retain(|x| !path.contains(x));
    let last_path_element = path.last().unwrap();
    options.sort_by(|b, a| {
        let val_a = distances[*a][*last_path_element];
        let val_b = distances[*b][*last_path_element];
        if val_a > val_b {
            Ordering::Less
        } else if val_a == val_b {
            Ordering::Equal
        } else {
            Ordering::Greater
        }
    });

    for i in options {
        path.push(i);
        solve_recursive(path, nodes, angles, distances, solution, input_file_name, iterations, max_iterations); 
        if *iterations % 1000000 == 0 {
            println!("{iterations}");
        }
        path.pop().unwrap();
    }
}

fn main() {
    env_logger::init();
    // Ließt alle Nodes und die Suchlänge ein
    let (nodes, max_iterations, name) = read_nodes();

    let (angles, distances) = calc_angles_distances(&nodes);

    let start_paths: Vec<Vec<usize>> = generate_start_paths(&angles, &distances);

    let solution: Vec<usize> = vec![];
    //let solution: Arc<Mutex<Vec<usize>>> = Arc::new(Mutex::new(vec![]));

    let mut handles: Vec<JoinHandle<Vec<usize>>> = vec![];
    for i in 0..start_paths.len() {
        let thread_number = i.clone(); 
        let mut l_start_path = start_paths[i].clone();
        let l_nodes = nodes.clone();
        let l_angles = angles.clone();
        let l_distances = distances.clone();
        let mut l_solution = solution.clone();
        let l_name = name.clone();
        let mut l_iterations = 0;
        let l_max_iterations = max_iterations.clone();

        let handle = thread::spawn(move || {
            solve_recursive(&mut l_start_path, &l_nodes, &l_angles, &l_distances, &mut l_solution, &l_name, &mut l_iterations, &l_max_iterations);
            info!("ran! {thread_number}");
            l_solution
        });

        handles.push(handle);
    }

    let mut solutions: Vec<Vec<usize>> = vec![];
    while let Some(handle) = handles.pop() {
        solutions.push(handle.join().unwrap());
    }

    // Stellt die gefundene Lösung dar
    if !solution.is_empty() {
        render(&nodes, &indices_to_nodes(nodes.clone(), &solution), path_len(&solution, &distances), name);
    }
}
