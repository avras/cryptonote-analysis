use std::time::Instant;
use clap::{Arg, Command};

use xmrtrace::{read_edges, write_rings};

fn main() {
    let cmd = Command::new("CryptoNote Cascade Attack")
    .bin_name("cascade")
    .arg(
        Arg::new("input_file")
            .value_name("Input Edge Filename")
            .required(true)
            .long_help("The name of the input file containing a list of edges")
    )
    .arg(
        Arg::new("output_file")
            .value_name("Output Ring Filename")
            .required(true)
            .long_help("The name of the output file containing a list of rings")
    )
    .arg(
        Arg::new("num_iterations")
            .value_name("Number of iterations")
            .value_parser(clap::value_parser!(u16))
            .required(false)
            .default_value("1")
            .long_help("The number of Cascade Attack iterations")
    )
    .after_help("The cascade command executes the Cascade Attack aka zero-mixin chain reaction attack on a set of CryptoNote transactions. \
    The transaction graph is described in a file. Each row in the file describes an edge. The first two entries in each row are non-negative integers separated by a space. The first integer is a key image identifier and the second integer is the public key identifier. The identifier spaces can overlap.");


    let m = cmd.get_matches();
    let input_fname = m.get_one::<String>("input_file").unwrap();
    let output_fname = m.get_one::<String>("output_file").unwrap();
    let num_iterations = m.get_one::<u16>("num_iterations").unwrap();

    let start_instant = Instant::now();
    let (pk_indices, ki_indices, max_pk_index, max_ki_index) = read_edges(input_fname);
    let end_instant = Instant::now();
    println!("Edge file read in {:?}", end_instant.duration_since(start_instant));


    let num_pks = (max_pk_index+1) as usize;
    let num_kis = (max_ki_index+1) as usize ;
    println!("Num keyimages = {}, Num public keys = {}", num_kis, num_pks);

    // Each index in tx_rings corresponds to one key image.
    // The vector at that index has the ring of public key indices.
    let mut tx_rings: Vec<Vec<u32>> = vec![vec![]; num_kis];
    // Each index in pk_to_ki_map corresponds to one public key.
    // The vector at that index has the indices of the corresponding key images.
    let mut pk_to_ki_map: Vec<Vec<u32>> = vec![vec![]; num_pks];

    assert_eq!(pk_indices.len(), ki_indices.len());
    let num_edges = ki_indices.len();

    for i in 0..num_edges {
       tx_rings[ki_indices[i] as usize].push(pk_indices[i]); 
       pk_to_ki_map[pk_indices[i] as usize].push(ki_indices[i]);
    }
    // println!("{:?}", tx_rings);
    // println!("{:?}", pk_to_ki_map);

    let mut tx_ring_traced: Vec<bool> = vec![false; num_kis];
    let mut num_traceable_rings = 0_usize;

    for i in 0..num_kis {
        if tx_rings[i].len() == 1 {
            num_traceable_rings += 1;
        }
    }
    println!("Zero-mixin rings before CA = {}", num_traceable_rings);
    let mut prev_num_traceable_rings = num_traceable_rings;

    for iter_index in 0..*num_iterations {
        let start_instant = Instant::now();
        for i in 0..num_kis {
            if tx_ring_traced[i] == false && tx_rings[i].len() == 1 {
                let traced_pk = tx_rings[i][0];

                for ki in pk_to_ki_map[traced_pk as usize].clone().into_iter() {
                    if ki != (i as u32) {
                        let ring = tx_rings[ki as usize].clone();
                        for j in 0..ring.len() {
                            if ring[j] == traced_pk {
                                tx_rings[ki as usize].swap_remove(j);
                                break; // Assumes that ring keys are unique
                            }
                        }
                    }
                }

                tx_ring_traced[i] = true;
            }
        }
        num_traceable_rings = 0;
        for i in 0..num_kis {
            if tx_rings[i].len() == 1 {
                num_traceable_rings += 1;
            }
        }

        let end_instant = Instant::now();

        println!("Zero-mixin rings after CA iteration {} = {}. Time taken = {:?}.",
            iter_index+1,
            num_traceable_rings,
            end_instant.duration_since(start_instant)
        );

        if prev_num_traceable_rings == num_traceable_rings {
            println!("No change in number of traceable rings. Exiting cascade attack loop");
            break;
        }
        prev_num_traceable_rings = num_traceable_rings;
    }
    write_rings(tx_rings, num_pks, output_fname);
}