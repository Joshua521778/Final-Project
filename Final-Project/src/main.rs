use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{BufRead, BufReader};


pub struct Graph {
    adjacency_list: HashMap<String, HashSet<String>>,
}

impl Graph {
    pub fn new() -> Self {
        Graph {
            adjacency_list: HashMap::new(),
        }
    }


    pub fn add_edge(&mut self, node1: String, node2: String) {
        self.adjacency_list.entry(node1.clone()).or_insert_with(HashSet::new).insert(node2.clone());
        self.adjacency_list.entry(node2).or_insert_with(HashSet::new).insert(node1);
    }

  
    pub fn degree_distribution(&self) -> HashMap<usize, usize> {
        let mut distribution = HashMap::new();

        for neighbors in self.adjacency_list.values() {
            let degree = neighbors.len();
            *distribution.entry(degree).or_insert(0) += 1;
        }

        distribution
    }


    pub fn neighbors_at_distance_two(&self, node: &String) -> usize {
        if let Some(neighbors) = self.adjacency_list.get(node) {
            let mut distance_two_neighbors = HashSet::new();

            for neighbor in neighbors {
                if let Some(second_neighbors) = self.adjacency_list.get(neighbor) {
                    for second_neighbor in second_neighbors {
                        if second_neighbor != node {
                            distance_two_neighbors.insert(second_neighbor.clone());
                        }
                    }
                }
            }

            return distance_two_neighbors.len();
        }

        0
    }
}


pub fn build_graph_from_csv(file_path: &str) -> Graph {
    let file = File::open(file_path).expect("Unable to open file");
    let reader = BufReader::new(file);

    let mut graph = Graph::new();

    for line in reader.lines() {
        if let Ok(record) = line {
            let parts: Vec<&str> = record.split(',').collect();
            if parts.len() >= 2 {
                let node1 = parts[0].trim().to_string();
                let node2 = parts[1].trim().to_string();
                graph.add_edge(node1, node2);
            }
        }
    }

    graph
}


pub fn evaluate_power_law(distribution: &HashMap<usize, usize>) -> f64 {
    let total_nodes: usize = distribution.values().sum();
    let mut observed: Vec<(usize, f64)> = distribution
        .iter()
        .map(|(&degree, &count)| (degree, count as f64 / total_nodes as f64))
        .collect();
    observed.sort_by(|a, b| a.0.cmp(&b.0)); 


    let mut theoretical: Vec<f64> = Vec::new();
    let alpha = 2.5; 
    let normalization: f64 = observed.iter().map(|(degree, _)| 1.0 / (*degree as f64).powf(alpha)).sum();
    for (degree, _) in &observed {
        theoretical.push(1.0 / (*degree as f64).powf(alpha) / normalization);
    }


    let mse: f64 = observed
        .iter()
        .zip(theoretical.iter())
        .map(|((_, obs_prob), theo_prob)| (obs_prob - theo_prob).powi(2))
        .sum();

    1.0 / (1.0 + mse) 
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_graph_construction() {
        let mut graph = Graph::new();
        graph.add_edge("A".to_string(), "B".to_string());
        graph.add_edge("A".to_string(), "C".to_string());

        assert_eq!(graph.adjacency_list["A"].len(), 2);
        assert_eq!(graph.adjacency_list["B"].len(), 1);
    }

    #[test]
    fn test_degree_distribution() {
        let mut graph = Graph::new();
        graph.add_edge("A".to_string(), "B".to_string());
        graph.add_edge("A".to_string(), "C".to_string());

        let distribution = graph.degree_distribution();
        assert_eq!(distribution[&2], 1);
        assert_eq!(distribution[&1], 2);
    }

    #[test]
    fn test_neighbors_at_distance_two() {
        let mut graph = Graph::new();
        graph.add_edge("A".to_string(), "B".to_string());
        graph.add_edge("B".to_string(), "C".to_string());
        graph.add_edge("C".to_string(), "D".to_string());

        assert_eq!(graph.neighbors_at_distance_two(&"A".to_string()), 1);
        assert_eq!(graph.neighbors_at_distance_two(&"B".to_string()), 2);
    }
}

fn main() {
    let file_path = "./dataset.csv";
    let graph = build_graph_from_csv(file_path);

    let degree_dist = graph.degree_distribution();
    println!("Degree Distribution: The graph has the following degree distribution, where the key represents the degree and the value represents the number of nodes with that degree: {:?}", degree_dist);
    for (degree, count) in &degree_dist {
        println!("{} nodes have a degree of {}. This means {} accounts participated in {} transactions.", count, degree, count, degree);
    }

    let power_law_fit = evaluate_power_law(&degree_dist);
    if power_law_fit > 0.8 {
        println!("Power-Law Fit: {:.2}. This indicates a strong fit to a power-law distribution. The network likely has a few highly connected nodes and many nodes with fewer connections, forming a hierarchical structure.", power_law_fit);
    } else {
        println!("Power-Law Fit: {:.2}. This indicates a weak fit to a power-law distribution. The network may not exhibit a centralized structure typically seen in social or transactional networks, indicating a more evenly distributed connectivity.", power_law_fit);
    }
}
