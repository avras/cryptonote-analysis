use std::time::Instant;
use clap::{Arg, Command};

use xmrtrace::{read_edges, read_rings};

fn main() {
    let cmd = Command::new("DM Decomposition Statistics")
    .bin_name("stats_dm")
    .arg(
        Arg::new("input_file")
            .value_name("Input Edge Filename")
            .required(true)
            .long_help("The name of the input file containing a list of edges")
    )
    .arg(
        Arg::new("pre_dmd_rings_file")
            .value_name("Pre DM Decomposition Rings Filename")
            .required(true)
            .long_help("The name of the file containing a list of rings before the DM decomposition")
    )
    .arg(
        Arg::new("post_dmd_rings_file")
            .value_name("Post DM Decomposition Rings Filename")
            .required(true)
            .long_help("The name of the file containing a list of rings after the DM decomposition")
    )
    .after_help("The stats_dm command calculates statistics after the Dulmage-Mendelsohn decomposition on a set of CryptoNote transactions.");


    let m = cmd.get_matches();
    let input_fname = m.get_one::<String>("input_file").unwrap();
    let pre_dmd_rings_file = m.get_one::<String>("pre_dmd_rings_file").unwrap();
    let post_dmd_rings_file = m.get_one::<String>("post_dmd_rings_file").unwrap();

    let mut start_instant = Instant::now();
    let (_pk_indices, _ki_indices, max_pk_index, max_ki_index) = read_edges(input_fname);
    let mut end_instant = Instant::now();
    println!("Edge file read in {:?}", end_instant.duration_since(start_instant));

    let num_pks = (max_pk_index+1) as usize;
    let num_kis = (max_ki_index+1) as usize ;
    println!("Num keyimages = {}, Num public keys = {}", num_kis, num_pks);


    start_instant = Instant::now();
    let (pre_dmd_tx_rings, _pk_to_ki_map) = read_rings(pre_dmd_rings_file);
    end_instant = Instant::now();
    println!("Pre DM decomposition rings file read in {:?}", end_instant.duration_since(start_instant));

    let max_ring_size = 10_usize;
    let mut pre_dm_mixin_histogram = vec![0_u32; max_ring_size+1];
    for ki in 0..num_kis {
        let l = pre_dmd_tx_rings[ki].len();
        
        if l <= max_ring_size {
            pre_dm_mixin_histogram[l-1] += 1;
        }
        else {
            pre_dm_mixin_histogram[max_ring_size] += 1;
        }
    }
    println!("Pre DM decomposition mixin histogram:\n {:?}", pre_dm_mixin_histogram);

    start_instant = Instant::now();
    let (post_dmd_tx_rings, _pk_to_ki_map) = read_rings(post_dmd_rings_file);
    end_instant = Instant::now();
    println!("Post DM decomposition rings file read in {:?}", end_instant.duration_since(start_instant));

    let mut post_dm_mixin_histogram = vec![0_u32; max_ring_size+1];
    let mut dm_traceable_ring_mixin_histogram = vec![0_u32; max_ring_size+1];
    for ki in 0..num_kis {
        let l = post_dmd_tx_rings[ki].len();
        
        if l <= max_ring_size {
            post_dm_mixin_histogram[l-1] += 1;

            if l == 1 {
                let pre_attack_l = pre_dmd_tx_rings[ki].len();
                if pre_attack_l <= max_ring_size {
                    dm_traceable_ring_mixin_histogram[pre_attack_l-1] += 1;
                }
                else {
                    dm_traceable_ring_mixin_histogram[max_ring_size] += 1;
                }
            }
        }
        else {
            post_dm_mixin_histogram[max_ring_size] += 1;
        }
    }
    println!("Post DM decomposition mixin histogram:\n {:?}", post_dm_mixin_histogram);
    println!("DM decomposition traceable ring mixin histogram:\n {:?}", dm_traceable_ring_mixin_histogram);

}