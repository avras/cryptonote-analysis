use std::time::Instant;
use clap::{Arg, Command};

use xmrtrace::{read_edges, read_rings};

fn main() {
    let cmd = Command::new("CryptoNote Clustering Algorithm Statistics")
    .bin_name("stats_cla")
    .arg(
        Arg::new("input_file")
            .value_name("Input Edge Filename")
            .required(true)
            .long_help("The name of the input file containing a list of edges")
    )
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
    .after_help("The stats_cla command calculates statistics after the cascade and clustering algorithms attacks on a set of CryptoNote transactions. \
    The transaction graph is described in a file. Each row in the file describes an edge. The first two entries in each row are non-negative integers separated by a space. The first integer is a key image identifier and the second integer is the public key identifier. The identifier spaces can overlap.");


    let m = cmd.get_matches();
    let input_fname = m.get_one::<String>("input_file").unwrap();
    let cascade_output_fname = m.get_one::<String>("post_cascade_attack_rings").unwrap();
    let clustering_output_fname = m.get_one::<String>("post_clustering_algorithm_rings").unwrap();

    let mut start_instant = Instant::now();
    let (pk_indices, ki_indices, max_pk_index, max_ki_index) = read_edges(input_fname);
    let mut end_instant = Instant::now();
    println!("Edge file read in {:?}", end_instant.duration_since(start_instant));


    let num_pks = (max_pk_index+1) as usize;
    let num_kis = (max_ki_index+1) as usize ;
    println!("Num keyimages = {}, Num public keys = {}", num_kis, num_pks);

    // Each index in tx_rings corresponds to one key image.
    // The vector at that index has the ring of public key indices.
    let mut tx_rings: Vec<Vec<u32>> = vec![vec![]; num_kis];

    let num_edges = ki_indices.len();

    for i in 0..num_edges {
       tx_rings[ki_indices[i] as usize].push(pk_indices[i]); 
    }

    let max_ring_size = 10_usize;
    let mut initial_mixin_histogram = vec![0_u32; max_ring_size+1];
    for ki in 0..num_kis {
        let l = tx_rings[ki].len();
        
        if l <= max_ring_size {
            initial_mixin_histogram[l-1] += 1;
        }
        else {
            initial_mixin_histogram[max_ring_size] += 1;
        }
    }
    println!("Initial mixin histogram:\n {:?}", initial_mixin_histogram);

    start_instant = Instant::now();
    let (post_cascade_tx_rings, _pk_to_ki_map) = read_rings(cascade_output_fname);
    end_instant = Instant::now();
    println!("Post cascade attack rings file read in {:?}", end_instant.duration_since(start_instant));

    let mut post_cascade_mixin_histogram = vec![0_u32; max_ring_size+1];
    let mut cascade_traceable_ring_mixin_histogram = vec![0_u32; max_ring_size+1];
    for ki in 0..num_kis {
        let l = post_cascade_tx_rings[ki].len();
        
        if l <= max_ring_size {
            post_cascade_mixin_histogram[l-1] += 1;

            if l == 1 {
                let pre_attack_l = tx_rings[ki].len();
                if pre_attack_l <= max_ring_size {
                    cascade_traceable_ring_mixin_histogram[pre_attack_l-1] += 1;
                }
                else {
                    cascade_traceable_ring_mixin_histogram[max_ring_size] += 1;
                }
            }
        }
        else {
            post_cascade_mixin_histogram[max_ring_size] += 1;
        }
    }
    println!("Post cascade attack mixin histogram:\n {:?}", post_cascade_mixin_histogram);
    println!("Cascade traceable ring pre-attack mixin histogram:\n {:?}", cascade_traceable_ring_mixin_histogram);

    let mut num_rings_traced_by_cascade_attack = 0_u32;
    println!("Pre-attack mixin histogram of rings traced by cascade attack");
    for i in 0..=max_ring_size {
        println!("{} {}",
            i,
            cascade_traceable_ring_mixin_histogram[i],
        );
        num_rings_traced_by_cascade_attack += cascade_traceable_ring_mixin_histogram[i];
    }
    println!("Total number of rings traced by cascade attack = {}", num_rings_traced_by_cascade_attack);

    start_instant = Instant::now();
    let (post_cluster_tx_rings, _pk_to_ki_map) = read_rings(clustering_output_fname);
    end_instant = Instant::now();
    println!("Post clustering algorithm rings file read in {:?}", end_instant.duration_since(start_instant));

    let mut post_cluster_mixin_histogram = vec![0_u32; max_ring_size+1];
    let mut cluster_traceable_ring_mixin_histogram = vec![0_u32; max_ring_size+1];
    for ki in 0..num_kis {
        let l = post_cluster_tx_rings[ki].len();
        
        if l <= max_ring_size {
            post_cluster_mixin_histogram[l-1] += 1;

            if l == 1 {
                let pre_attack_l = tx_rings[ki].len();
                if pre_attack_l <= max_ring_size {
                    cluster_traceable_ring_mixin_histogram[pre_attack_l-1] += 1;
                }
                else {
                    cluster_traceable_ring_mixin_histogram[max_ring_size] += 1;
                }
            }
        }
        else {
            post_cluster_mixin_histogram[max_ring_size] += 1;
        }
    }
    println!("Post clustering algorithm mixin histogram:\n {:?}", post_cluster_mixin_histogram);
    println!("Cluster traceable ring post-attack mixin histogram:\n {:?}", cluster_traceable_ring_mixin_histogram);

    let mut num_rings_traced_by_clustering_algo = 0_u32;
    println!("Post-attack mixin histogram of rings traced by clustering algorithm");
    for i in 0..=max_ring_size {
        let diff = cluster_traceable_ring_mixin_histogram[i] - cascade_traceable_ring_mixin_histogram[i];
        println!("{} {}",
            i,
            diff,
        );
        num_rings_traced_by_clustering_algo += diff;
    }
    println!("Total number of rings traced by clustering algorithm = {}", num_rings_traced_by_clustering_algo);
}