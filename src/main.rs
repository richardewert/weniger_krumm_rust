// Für den eigentlchen Algorithmus irrelevanter Code,
// zum schreiben und lesen von Beispielen und Lösungen
mod input_output_mod;

// Enthält den Node Struct und dessen Funktionen
mod node_mod;

// Funktionen zum lesen und schreiben
use input_output_mod::{read_nodes, render};
// Logging crate 
use log::{debug, info};
use node_mod::Node;
use std::cmp::Ordering;
use std::f32::MAX;
use std::num::NonZeroUsize;
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

// Gibt alle Kombinationen aus 3 unterschiedlichen Nodes nach länge sortiert zurück
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
    debug!("Start paths: {:?}", paths);
    info!("Generated {} start paths", paths.len());
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
    best_solution: &mut Arc<Mutex<Vec<usize>>>, 
    best_solution_length: &mut Arc<Mutex<f32>>,
    input_file_name: &String, 
    iterations: &mut u64,
    max_iterations: &u64,
) {
    // Jeder Aufruf der Funktion erhöht den iterationszähler um 1 
    *iterations += 1;
    // ========== ANFANG DER ABBRUCHBEDINGUNGEN ============================
    // Die Länge der besten bekannten Lösung ist über alle Threads geteilt.
    // Deshalb wird die Variable zuerst "gelocked", um andere Threads am Verändern zu hindern.
    let mut sol_len = best_solution_length.lock().unwrap();
    // Abgebrochen wird, wenn die maximale Menge an Iterationen erreicht oder
    // die Länge des eigenen Pfades größer als die der kürzesten bekannten Lösung ist.
    if *iterations > *max_iterations || path_length >= *sol_len {return};
    // Wenn es so viele Einträge im Pfad, wie Nodes gibt, wurde eine Lösung gefunden,
    // wird sie als neue gespeichert und die Funktion abgebrochen.
    if path.len() == nodes.len() {
        // Auch die beste Lösung wird "gelocked"
        let mut best_solution_lock = best_solution.lock().unwrap();
        // Der aktuelle Pfad wird in die Stelle der besten Lösung geklont
        path.clone_into(&mut best_solution_lock);
        // Die Länge der besten Lösung wird aktuallisiert 
        *sol_len = path_length;
        // Die Lösung wird als txt und svg gespeichert
        render(
            nodes, 
            &indices_to_nodes(nodes.clone(), &best_solution_lock), 
            path_length, input_file_name.clone()
        );
        return;
    }
    // Die Länge der besten Lösung wird nicht mehr benötigt
    // und fallen gelasssen, um anderen Threads den Zugriff zu gewähren
    drop(sol_len);
    // ========== ENDE DER ABBRUCHBEDINGUNGEN ============================

    // Alle Nodes, welche die Winkelbedingung erfüllen,
    // werden nach Distanz sortiert und in "options" gespeichert
    let mut options: Vec<usize> = 
    	angles[path[path.len() - 2]][path[path.len() - 1]].clone();
    // Es werden nur die behalten, welche nicht im Pfad enthalten sind
    options.retain(|x| !path.contains(x));

    // Jede dieser Nodes
    for i in options {
        // Wird zum Pfad hinzugefügt
        path.push(i);
        // Die zusätzliche Länge wird berechnet
        let add_length = distances[path[path.len() - 2]][path[path.len() - 1]];
        // Der veränderte Pfad wird mit den anderen Parametern weitergegeben
        solve_recursive(
            path, 
            path_length + add_length, 
            nodes, 
            angles, 
            distances, 
            best_solution, 
            best_solution_length, 
            input_file_name, 
            iterations, 
            max_iterations,
        ); 
        // Nachdem das finden der Lösungen dieses Teilbaumes abgeschlossen ist, 
        // werden die Veränderungen zum Pfad wieder rückgängig gemacht 
        path.pop().unwrap();
    }
}

fn main() {
    env_logger::init();
    
    // Ließt alle Nodes und die Suchlänge ein
    let (nodes, max_iterations, name) = read_nodes();

    // Winkel und Distanzen werden zum schnellen auslesen berechnet
    let (angles, distances) = calc_angles_distances(&nodes);

    // Alle Anfangspfade werden bestimmt
    let generated_paths = generate_start_paths(&angles, &distances);

    // Diese Variablen sind über alle Threads geteilt:
    // Vector mit Startpfaden die noch probiert werden müssen
    let start_paths: Arc<Mutex<Vec<Vec<usize>>>> = 
    	Arc::new(Mutex::new(generated_paths.clone()));
    // Die aktuell beste bekannte Lösung
    let best_solution: Arc<Mutex<Vec<usize>>> = Arc::new(Mutex::new(vec![]));
    // Die Länge der aktuell besten bekannten Lösung
    let best_solution_length: Arc<Mutex<f32>> = Arc::new(Mutex::new(MAX));
    let done_threads: Arc<Mutex<u32>> = Arc::new(Mutex::new(0));

    // Hier werden die handles für die Threads eingetragen werden
    let mut handles: Vec<JoinHandle<()>> = vec![];
    // Bestimmt, wie viele CPU Kerne zur Verfügung stehen
    let total_threads: usize = available_parallelism()
        .unwrap_or(NonZeroUsize::new(1).unwrap()).into();

    info!("Starting up {:?} threads", total_threads);
    for _i in 0..(total_threads - 1) {
        // Jeder Thread benötigt Zugriff auf die obigen Variablen.
        // Deshalb werden diese vom Hauptthread in neue mit "l_" 
        // notierte Variablen geklont. 
        // Jede der geteilten Variablen benötigt eine Referenz
        let mut l_best_solution = Arc::clone(&best_solution);
        let mut l_best_solution_length = Arc::clone(&best_solution_length);
        let l_start_paths = Arc::clone(&start_paths);
        let l_done_threads = Arc::clone(&done_threads);

        // Konstante Werte werden geklont
        let l_nodes = nodes.clone();
        let l_angles = angles.clone();
        let l_distances = distances.clone();
        let l_name = name.clone();
        let l_max_iterations = max_iterations.clone();
        let l_total_tasks = generated_paths.len();

        // Der neue Thread wird gestartet.
        // Die obigen "l_" Variablen ziehen in den Thread um, 
        // wenn sie Referenziert werden.
        let handle = thread::spawn(move || {
            info!("Started thread {:?}", thread::current().id());
            loop {
                let mut todo = l_start_paths.lock().unwrap();
                // Falls noch ein ungeprüfter Startpfad existiert
                if let Some(mut l_start_path) = todo.pop() {
                    let thread_number = todo.len();
                    drop(todo);
                    let l_path_length = path_len(&l_start_path, &l_distances);
                    let mut l_iterations = 0;
                    // Finde alle Lösungen
                    solve_recursive(
                        &mut l_start_path, 
                        l_path_length, &l_nodes, 
                        &l_angles, &l_distances, 
                        &mut l_best_solution, 
                        &mut l_best_solution_length, 
                        &l_name, 
                        &mut l_iterations, 
                        &l_max_iterations
                    );
                    // Debug und ausgabe (irrelevant)
                    let mut done = l_done_threads.lock().unwrap();
                    *done += 1;
                    debug!(
                        "Thread {:?} finished path with priority {:?}  {:?}/{:?}",
                        thread::current().id(), 
                        thread_number, 
                        done, 
                        l_total_tasks
                    );
                    drop(done);
                } else {
                    // Beende die Schleife und damit den Thread,
                    // wenn alle Startpfade probiert worden sind
                    break;
                }
            }
        });
        handles.push(handle);
    }
    // Iteriert über alle Threadhandles und wartet bis sie fertig sind
    while let Some(handle) = handles.pop() {
        handle.join().unwrap();
    }

    let final_solution = best_solution.lock().unwrap();
    // Stellt die gefundene Lösung dar
    if !final_solution.is_empty() {
        render(
            &nodes, 
            &indices_to_nodes(nodes.clone(), &final_solution), 
            path_len(&final_solution, &distances), 
            name
        );
    }
}
