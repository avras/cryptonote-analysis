use std::{time::Instant, collections::{HashMap, HashSet, BTreeMap}};
use clap::{Arg, Command};
use petgraph::{Graph, Undirected};
use petgraph::prelude::DiGraphMap;
use petgraph::algo::tarjan_scc;
use petgraph::algo::maximum_matching;

use xmrtrace::{read_rings, write_ring_sets, remove_closed_set_pks};

fn main() {
    let cmd = Command::new("Clustering Algorithm for Closed Set Attack")
    .bin_name("cluster")
    .arg(
        Arg::new("post_cascade_attack_rings")
            .value_name("Post Cascade Attack Rings Filename")
            .required(true)
            .long_help("The name of the file containing a list of rings after the cascade attack")
    )
    .arg(
        Arg::new("post_clustering_algorithm_rings")
            .value_name("Post Clustering Algorithm Rings Filename")
            .required(true)
            .long_help("The name of the file containing a list of rings after the clustering algorithm")
    )
    .after_help("The cluster command executes the clustering algorithm to implement the closed set attack of Yu et al (FC 2019).\
    The input file has a list of rings that have already been subjected to the cascade attack. The first line of the file has the \
    number of rings. Each subsequent line begins with the index of the key image, followed by the indices of the public keys in the \
    rings. The indices are separated by a single space. The key image indices range from 0 to one less than the number of rings. \
    The public key indices range from 0 to one less than the number of distinct public keys.");


    let m = cmd.get_matches();
    let input_fname = m.get_one::<String>("post_cascade_attack_rings").unwrap();
    let output_fname = m.get_one::<String>("post_clustering_algorithm_rings").unwrap();

    let mut start_instant = Instant::now();
    let (tx_rings, pk_to_ki_map) = read_rings(input_fname);
    let mut end_instant = Instant::now();
    println!("Rings file read in {:?}", end_instant.duration_since(start_instant));

    let mut tx_ring_sets: Vec<HashSet<u32>> = vec![];
    start_instant = Instant::now();
    for ring in &tx_rings {
        tx_ring_sets.push(HashSet::from_iter(ring.into_iter().map(|x| *x)));
    }
    end_instant = Instant::now();
    println!("Ring sets created in {:?}", end_instant.duration_since(start_instant));

    let mut num_traceable_rings = 0_u32;
    let mut ki_in_closed_set = vec![false; tx_ring_sets.len()];
    start_instant = Instant::now();
    for ki in 0..tx_ring_sets.len() {
        if tx_ring_sets[ki].len() == 1 {
            ki_in_closed_set[ki] = true;
            num_traceable_rings += 1;
        }
    }
    end_instant = Instant::now();
    println!("Counted initial number of traceable rings in {:?}", end_instant.duration_since(start_instant));
    println!("Number of traceable rings = {}", num_traceable_rings);

    let mut flag = true;
    let mut search_iteration: u32 = 0;
    let mut closed_set_size_histogram: BTreeMap<usize, u32> = BTreeMap::new();
    let mut set_of_closed_sets = HashSet::<Vec<u32>>::new();
    let mut set_of_all_closed_set_pks = HashSet::<u32>::new();

    // Clustering algorithm (pp 11 of Yu et al, FC 2019)
    while flag {
        println!("At beginning of clustering algorithm while loop");
        search_iteration += 1;
        flag = false;
        let mut num_clusters_found = 0_u32;
        for ki in 0..tx_ring_sets.len() {
            if tx_ring_sets[ki].len() != 1 && ki_in_closed_set[ki] == false {
                let (clus_ki_set, clus_pk_set) = form_cluster(ki, &tx_ring_sets, &pk_to_ki_map, &ki_in_closed_set);

                if clus_ki_set.len() == clus_pk_set.len() {
                    num_clusters_found += 1;
                    println!("{}: Cluster of size {} found at key index {}. Search iteration = {}",
                        num_clusters_found,
                        clus_pk_set.len(),
                        ki,
                        search_iteration,
                    );
                    let closed_set_ki_pk_set_pairs = decompose_closed_set(&clus_ki_set, &clus_pk_set, &tx_ring_sets);
                    for key_im in &clus_ki_set {
                        ki_in_closed_set[*key_im as usize] = true;
                    }
                    for (closed_ki_set, closed_pk_set) in closed_set_ki_pk_set_pairs {
                        closed_set_size_histogram.entry(closed_ki_set.len()).and_modify(|c| *c += 1).or_insert(1);
                        let mut closet_ki_set_vec: Vec<u32> = closed_ki_set.clone().into_iter().collect();
                        closet_ki_set_vec.sort();
                        set_of_closed_sets.insert(closet_ki_set_vec);
                        set_of_all_closed_set_pks.extend(closed_pk_set.clone());

                        if remove_closed_set_pks(closed_ki_set, closed_pk_set, &mut tx_ring_sets, &pk_to_ki_map) {
                            flag = true;  // If a public key is removed, we should run the cluster search again
                        };
                    }
                }
            }
        }
        
        num_traceable_rings = 0;
        for ki in 0..tx_ring_sets.len() {
            if tx_ring_sets[ki].len() == 1 {
                ki_in_closed_set[ki] = true;
                num_traceable_rings += 1;
            }
            else {
                ki_in_closed_set[ki] = false;
            }
        }
        println!("Number of traceable rings = {}", num_traceable_rings);
        println!("Number of closed sets = {}", set_of_closed_sets.len());
        let mut num_singleton_closed_sets = 0_u32;

        for set in &set_of_closed_sets {
            if set.len() == 1 {
                num_singleton_closed_sets += 1;
            }
        }
        println!("Number of singleton closed sets = {}", num_singleton_closed_sets);
        println!("Number of non-singleton closed sets = {}", set_of_closed_sets.len() - (num_singleton_closed_sets as usize));
        println!("Closed set size histogram: {:?}", closed_set_size_histogram);
        closed_set_size_histogram.clear();
        println!("Number of public keys in all sets = {}", set_of_all_closed_set_pks.len());
        println!("Number of public keys in non-singleton closed sets = {}", set_of_all_closed_set_pks.len() - (num_singleton_closed_sets as usize));
    }

    let max_ring_size = 10_usize;
    let mut pre_attack_mixin_histogram = vec![0_u32; max_ring_size+1];
    let mut post_attack_mixin_histogram = vec![0_u32; max_ring_size+1];
    for ki in 0..tx_ring_sets.len() {
        let mut l = tx_rings[ki].len();
        
        if l <= max_ring_size {
            pre_attack_mixin_histogram[l-1] += 1;
        }
        else {
            pre_attack_mixin_histogram[max_ring_size] += 1;
        }

        l = tx_ring_sets[ki].len();

        if l <= max_ring_size {
            post_attack_mixin_histogram[l-1] += 1;
        }
        else {
            post_attack_mixin_histogram[max_ring_size] += 1;
        }
    }

    println!("Pre attack mixin histogram:\n {:?}", pre_attack_mixin_histogram);
    println!("Post attack mixin histogram:\n {:?}", post_attack_mixin_histogram);

    write_ring_sets(&tx_ring_sets, pk_to_ki_map.len(), output_fname);
}

fn form_cluster(
    keyimage_index: usize,
    tx_ring_sets: &Vec<HashSet<u32>>,
    pk_to_ki_map: &Vec<Vec<u32>>,
    ki_in_closed_set: &Vec<bool>,
) -> (HashSet<u32>, HashSet<u32>) {
    let mut cluster_keyimage_set  = HashSet::from([keyimage_index as u32]);
    let mut cluster_pk_set = tx_ring_sets[keyimage_index].clone();
    let mut continue_search = true;


    while continue_search {
        continue_search = false;

        let mut candidate_ring_kis = HashSet::<u32>::new();
        for pk in &cluster_pk_set {
            for ki in &pk_to_ki_map[*pk as usize] {
                if cluster_keyimage_set.contains(ki) == false && ki_in_closed_set[*ki as usize] == false {
                    candidate_ring_kis.insert(*ki);
                }
            }
        }

        for ki in candidate_ring_kis {
            let diff_set: HashSet<u32> = tx_ring_sets[ki as usize].difference(&cluster_pk_set).map(|x| *x).collect();
            if diff_set.len() <= 1 {
                cluster_pk_set.extend(diff_set.iter());
                cluster_keyimage_set.insert(ki);
                continue_search = true;
            }
        }
    }
    return (cluster_keyimage_set, cluster_pk_set)
}

// Decomposes a closed set into its constituent closed sets
// Returns a vector of set pairs. The first element in the pair
// is a key image subset and the other is the corresponding
// public key set matched to it.
fn decompose_closed_set(
    closed_keyimage_set: &HashSet<u32>,
    closed_pk_set: &HashSet<u32>,
    tx_ring_sets: &Vec<HashSet<u32>>,
) -> Vec<(HashSet<u32>, HashSet<u32>)> {

    assert_eq!(closed_keyimage_set.len(), closed_pk_set.len());

    #[derive(Debug, PartialEq, Clone, Copy)]
    enum NodeType {
        PubKey,
        KeyImage,
    }

    let mut closed_graph = Graph::<(u32, NodeType), (), Undirected>::new_undirected();
    
    let mut ki_node_indices = HashMap::new();
    for ki in closed_keyimage_set {
        ki_node_indices.insert(*ki, closed_graph.add_node((*ki, NodeType::KeyImage)));
    }

    let mut pk_node_indices = HashMap::new();
    for pk in closed_pk_set {
        pk_node_indices.insert(pk, closed_graph.add_node((*pk, NodeType::PubKey)));
    }

    for ki in closed_keyimage_set {
        for pk in &tx_ring_sets[*ki as usize] {
            let ki_index = ki_node_indices.get(ki).unwrap();
            let pk_index = pk_node_indices.get(pk).unwrap();
            closed_graph.add_edge(*ki_index, *pk_index, ());
        }
    }

    let matching = maximum_matching(&closed_graph);
    // for e in matching.edges() {
    //     println!("{:?} {:?}", closed_graph[e.0], closed_graph[e.1]);
    // }

    let mut fd_graph = DiGraphMap::<u32, ()>::new();

    for pk_node_index in pk_node_indices.values() {
        let (pk_index, nt) = closed_graph[*pk_node_index];
        assert_eq!(nt, NodeType::PubKey);
        
        for ki_node_index in closed_graph.neighbors(*pk_node_index) {
            let pk_mate_node_index = matching.mate(ki_node_index).unwrap();
            if *pk_node_index != pk_mate_node_index {
                let (pk_mate_index, nt) = closed_graph[pk_mate_node_index];
                assert_eq!(nt, NodeType::PubKey);

                fd_graph.add_edge(pk_index, pk_mate_index, ());
            }
        }
    }

    // Finding strongly connected components
    let pk_scc = tarjan_scc(&fd_graph);
    println!("Number of blocks in fine decomposition: {}", pk_scc.len());
    let mut singletons = 0_u32;
    let mut ki_scc: Vec<Vec<u32>> = vec![vec![]; pk_scc.len()];

    let mut closed_set_ki_pk_set_pairs: Vec<(HashSet<u32>, HashSet<u32>)> = vec![];
    for i in 0..pk_scc.len() {
        if pk_scc[i].len() == 1 {
            singletons += 1;
        }
        for pk in &pk_scc[i] {
            let pk_node_index = pk_node_indices.get(pk).unwrap();
            let ki_mate_node_index = matching.mate(*pk_node_index).unwrap();
            let (ki, _) = closed_graph[ki_mate_node_index];
            ki_scc[i].push(ki);
        }
        closed_set_ki_pk_set_pairs.push(
            (
                HashSet::from_iter((&ki_scc[i]).into_iter().map(|x| *x)),
                HashSet::from_iter((&pk_scc[i]).into_iter().map(|x| *x))
            )
        );
    }
    println!("Singletons (traceable keyimages): {}", singletons);

    return closed_set_ki_pk_set_pairs;
}
