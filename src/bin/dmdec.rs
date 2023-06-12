use std::{fs::File, io::{BufWriter, Write}};

use std::time::Instant;
use std::collections::{HashSet, BTreeMap};
use clap::{Arg, Command};
use petgraph::{graphmap::DiGraphMap, algo::tarjan_scc};
use sprs::{TriMatBase, CsMatBase};

use xmrtrace::{read_edges, cn_txgraph_maximum_matching, write_ring_sets, remove_closed_set_pks};

fn main() {
    let cmd = Command::new("Dulmage-Mendelsohn Decomposition Calculator")
    .bin_name("dmdec")
    .arg(
        Arg::new("input_file")
            .value_name("Input Edge Filename")
            .required(true)
            .long_help("The name of the input file containing a list of edges")
    )
    .arg(
        Arg::new("pre_dmd_rings_file")
            .value_name("Pre DM Decomposition Rings Output Filename")
            .required(true)
            .long_help("The name of the output file that will have a list of rings before the DM decomposition")
    )
    .arg(
        Arg::new("post_dmd_rings_file")
            .value_name("Post DM Decomposition Rings Output Filename")
            .required(true)
            .long_help("The name of the output file that will have a list of rings after the DM decomposition")
    )
    .arg(
        Arg::new("closed_set_sizes_file")
            .value_name("Post DM Decomposition Closed Set Sizes Output Filename")
            .required(true)
            .long_help("The name of the output file that will have a list of closed set sizes after the DM decomposition")
    )
    .arg(
        Arg::new("fine_decomposition_file")
            .value_name("Post DM Decomposition Fine Decomposition Output Filename")
            .required(true)
            .long_help("The name of the output file that will have a list of closed sets in the DM fine decomposition")
    )
    .after_help("The dmdec command calculates the Dulmage-Mendelsohn decomposition \
    of a matrix described in a file. Each row in the file describes an edge. \
    The first two entries in each row are non-negative integers separated by a \
    space. The first integer is a key image identifier and the second integer is \
    the public key identifier. The identifier spaces can overlap.");


    let m = cmd.get_matches();
    let edge_file = m.get_one::<String>("input_file").unwrap();
    let pre_dmd_rings_file = m.get_one::<String>("pre_dmd_rings_file").unwrap();
    let post_dmd_rings_file = m.get_one::<String>("post_dmd_rings_file").unwrap();
    let closed_set_sizes_file = m.get_one::<String>("closed_set_sizes_file").unwrap();
    let fine_decomposition_file = m.get_one::<String>("fine_decomposition_file").unwrap();

    let start_instant = Instant::now();
    let (pk_indices, ki_indices, max_pk_index, max_ki_index) = read_edges(edge_file);
    let end_instant = Instant::now();
    println!("Edge file read in {:?}", end_instant.duration_since(start_instant));


    let num_pks = (max_pk_index+1) as usize;
    let num_kis = (max_ki_index+1) as usize ;
    println!("Num keyimages = {}, Num public keys = {}", num_kis, num_pks);

    // Each index in tx_ring_sets corresponds to one key image.
    // The vector at that index has the ring of public key indices.
    let mut tx_ring_sets: Vec<HashSet<u32>> = vec![HashSet::new(); num_kis];
    // Each index in pk_to_ki_map corresponds to one public key.
    // The vector at that index has the indices of the corresponding key images.
    let mut pk_to_ki_map: Vec<Vec<u32>> = vec![vec![]; num_pks];

    assert_eq!(pk_indices.len(), ki_indices.len());
    let num_edges = ki_indices.len();

    for i in 0..num_edges {
       tx_ring_sets[ki_indices[i] as usize].insert(pk_indices[i]); 
       pk_to_ki_map[pk_indices[i] as usize].push(ki_indices[i]);
    }
    write_ring_sets(&tx_ring_sets, num_pks, pre_dmd_rings_file);


    let data = vec![1_u8; pk_indices.len()];
    let tripmat: TriMatBase<Vec<u32>, Vec<u8>> = TriMatBase::from_triplets(
        (num_pks, num_kis),
        pk_indices,
        ki_indices,
        data,
    );
    let g: CsMatBase<u8, u32,_,_,_> = tripmat.to_csc();
    let pk_mates = cn_txgraph_maximum_matching(&g);


    // Finding sets of reachable and unreachable public keys and key images
    // from unmatched public keys (rows)
    let mut num_matched_pks: u32 = 0;
    let mut ki_mates = vec![u32::MAX; num_kis];
    let mut queue = vec![u32::MAX; num_pks];
    let mut reachable_pubkeys = vec![false; num_pks];
    let mut reachable_keyimages = vec![false; num_kis];

    let mut qhead = 0_usize;
    let mut qtail = 0_usize;

    for i in 0..num_pks {
        if pk_mates[i] != u32::MAX {
            ki_mates[pk_mates[i] as usize] = i as u32;
            num_matched_pks += 1;
        }
        else {
            reachable_pubkeys[i] = true;
            queue[qtail] = i as u32;
            qtail += 1;
        }
    }
    println!("Matched {} out of {} rows (public keys)", num_matched_pks, num_pks);

    let g_csr = g.to_csr();
    let csr_row_ptr = g_csr.indptr();
    let csr_row_ptr_slice = csr_row_ptr.as_slice().unwrap();
    let csr_col_ptr_slice = g_csr.indices();

    let mut q : u32;
    while qhead < qtail {
        let row_idx = queue[qhead]; // row_idx points to a pubkey
        qhead += 1;
        q = csr_row_ptr_slice[row_idx as usize];
        while q < csr_row_ptr_slice[(row_idx+1) as usize] {

            let col_idx = csr_col_ptr_slice[q as usize]; // This points to a keyimage
            if reachable_keyimages[col_idx as usize] {
                q += 1;
                continue;
            }
            reachable_keyimages[col_idx as usize] = true;
            
            let row_mate = ki_mates[col_idx as usize]; //row_mate is a pubkey
            if reachable_pubkeys[row_mate as usize] {
                q += 1;
                continue;
            }
            reachable_pubkeys[row_mate as usize] = true;
            queue[qtail] = row_mate;
            qtail += 1;

            q += 1;
        }
    }

    let mut num_unreachable_keyimages = num_kis;
    for i in 0..num_kis {
        if reachable_keyimages[i] {
            num_unreachable_keyimages -= 1;
        }
    }

    let mut num_unreachable_pubkeys = num_pks;
    for i in 0..num_pks {
        if reachable_pubkeys[i] {
            num_unreachable_pubkeys -= 1;
        }
    }
    println!("Number of unreachable pubkeys and keyimages = {} {}", num_unreachable_pubkeys, num_unreachable_keyimages);

        // Fine decomposition graph
    let mut fd_graph = DiGraphMap::<u32, ()>::new();

    // Iterating over all public keys in the square submatrix
    // and building the directed graph
    for pk_idx in 0..num_pks {
        if reachable_pubkeys[pk_idx] {
            continue;
        }
        fd_graph.add_node(pk_idx as u32);
        q = csr_row_ptr_slice[pk_idx];
        while q < csr_row_ptr_slice[pk_idx+1] {
            let ki_idx = csr_col_ptr_slice[q as usize]; // This points to a keyimage
            if reachable_keyimages[ki_idx as usize] == false && ki_mates[ki_idx as usize] != pk_idx as u32 {
                fd_graph.add_edge(ki_mates[ki_idx as usize], pk_idx as u32, ());
            }
            q += 1;
        }
    }

    // Finding strongly connected components
    let pk_scc = tarjan_scc(&fd_graph);
    println!("Number of blocks in fine decomposition: {}", pk_scc.len());
    let mut singletons = 0_u32;
    
    let file_err_msg = "Unable to create or write to file";
    let closed_set_sizes_file = File::create(closed_set_sizes_file).expect(file_err_msg);
    let fine_decomp_file = File::create(fine_decomposition_file).expect(file_err_msg);
    let mut closed_set_sizes_file_buf = BufWriter::new(closed_set_sizes_file);
    let mut fine_decomp_file_buf = BufWriter::new(fine_decomp_file);
    let mut closed_set_size_histogram: BTreeMap<usize, u32> = BTreeMap::new();
    
    // Write the number of blocks in the fine decomposition
    writeln!(fine_decomp_file_buf, "{}", pk_scc.len()).expect(file_err_msg);

    for comp_vec in pk_scc {
        closed_set_size_histogram.entry(comp_vec.len()).and_modify(|c| *c += 1).or_insert(1);
        write!(closed_set_sizes_file_buf, "{} ", comp_vec.len()).expect(file_err_msg);
        writeln!(fine_decomp_file_buf, "{}", comp_vec.len()).expect(file_err_msg);

        if comp_vec.len() == 1 {
            singletons += 1;
        }
        let mut ki_set: HashSet<u32> = HashSet::new();
        let mut pk_set: HashSet<u32> = HashSet::new();
        for pk in &comp_vec {
            write!(fine_decomp_file_buf, "{} ", pk).expect(file_err_msg);
            let ki = pk_mates[*pk as usize];
            pk_set.insert(*pk);
            ki_set.insert(ki);
        }

        writeln!(fine_decomp_file_buf, "").expect(file_err_msg); // End the line
        for ki in &ki_set {
            write!(fine_decomp_file_buf, "{} ", *ki).expect(file_err_msg);
        }
        writeln!(fine_decomp_file_buf, "").expect(file_err_msg); // End the line

        remove_closed_set_pks(ki_set, pk_set, &mut tx_ring_sets, &pk_to_ki_map);
    }
    write!(closed_set_sizes_file_buf, "\n").expect(file_err_msg);

    println!("Singletons (traceable keyimages): {}", singletons);
    println!("Closed set size histogram: {:?}", closed_set_size_histogram);

    write_ring_sets(&tx_ring_sets, num_pks, post_dmd_rings_file);
}
