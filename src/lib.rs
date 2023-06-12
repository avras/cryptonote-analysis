use std::collections::HashSet;
use std::{fs::File, path::Path};
use std::io::{self, BufRead, BufWriter, Write};

use sprs::CsMatI;

// Code from https://doc.rust-lang.org/rust-by-example/std_misc/file/read_lines.html
fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where P: AsRef<Path>, {
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}
// Reads a text file containing one edge per row
// and returns vectors of row and col indices.
// Also returns the maximum row and column index values 
pub fn read_edges(
    filename: &String,
) -> (Vec<u32>, Vec<u32>, u32, u32) {
    let mut row_indices: Vec<u32> = vec![];
    let mut col_indices: Vec<u32> = vec![];
    let mut max_col_index: u32 = 0;
    let mut max_row_index: u32 = 0;

    if let Ok(lines) = read_lines(filename) {

        for line in lines {
            if let Ok(edge) = line {
                let items: Vec<&str> = edge.trim().split(' ').collect();
                assert!(items.len() > 1);

                let keyimage_index = items[0].parse::<u32>().unwrap();
                let pubkey_index = items[1].parse::<u32>().unwrap();

                col_indices.push(keyimage_index);
                row_indices.push(pubkey_index);
                max_col_index = max_col_index.max(keyimage_index);
                max_row_index = max_row_index.max(pubkey_index);
            }
        }
    }
    assert!(row_indices.len() == col_indices.len());
    return (row_indices, col_indices, max_row_index, max_col_index);
}

// Writes a text file containing one edge per line
// Each line has a keyimage index and a pk index
// separated by a space
pub fn write_edges(
    tx_ring_sets: Vec<HashSet<u32>>,
    filename: &String,
) -> () {
    let file_err_msg = "Unable to create or write to file";
    let output_file = File::create(filename).expect(file_err_msg);
    let mut buf = BufWriter::new(output_file);

    for ki_index in 0..tx_ring_sets.len() {
        for pk_index in &tx_ring_sets[ki_index]  {
            writeln!(buf, "{} {}", ki_index, pk_index).expect(file_err_msg);
        }
    }
}

// Read a list of rings from an input file.
// The first line of the file has the number of rings and number of distinct
// public keys separated by a single space.
// Each subsequent line begins with the index of the key image, followed by
// the indices of the public keys in the rings. The indices are separated by
// a single space. The key image indices range from 0 to one less than the
// number of rings. The public key indices range from 0 to one less than
// the number of distinct public keys.
// Two output vectors:
// - The ith index of the first vector has the ring of public keys corresponding
//   to the ith key image
// - The ith index of the second vector has the list of key images in whose rings
//   the ith public key appeared in.
pub fn read_rings(
    filename: &String
) -> (Vec<Vec<u32>>, Vec<Vec<u32>>) {
    let mut tx_rings: Vec<Vec<u32>> = vec![];
    let mut pk_to_ki_map: Vec<Vec<u32>> = vec![];

    if let Ok(mut lines) = read_lines(filename) {
        if let Some(dim_str) = lines.next() {
            
            let dim_str = dim_str.unwrap();
            let items: Vec<&str> = dim_str.trim().split(' ').collect();
            assert!(items.len() > 1);

            let num_keyimages = items[0].parse::<usize>().unwrap();
            tx_rings = vec![vec![]; num_keyimages];
            let num_pks = items[1].parse::<usize>().unwrap();
            pk_to_ki_map = vec![vec![]; num_pks];

            for line in lines {
                if let Ok(ring) = line {
                    let items: Vec<&str> = ring.trim().split(' ').collect();
                    assert!(items.len() > 1);

                    let keyimage_index = items[0].parse::<usize>().unwrap();
                    for pk in items[1..].into_iter() {
                        let pk_index = pk.parse::<u32>().unwrap();
                        tx_rings[keyimage_index].push(pk_index);
                        pk_to_ki_map[pk_index as usize].push(keyimage_index as u32);
                    }
                }
            }
        }
    }

    (tx_rings, pk_to_ki_map)
}

// Write a list of rings to an output file.
// The first line of the file has the number of rings and number of distinct
// public keys separated by a single space.
// Each subsequent line begins with the index of the key image, followed by
// the indices of the public keys in the rings. The indices are separated by
// a single space. The key image indices range from 0 to one less than the
// number of rings. The public key indices range from 0 to one less than
// the number of distinct public keys.
pub fn write_rings(
    tx_rings: Vec<Vec<u32>>,
    num_pks: usize,
    filename: &String,
) -> () {
    let file_err_msg = "Unable to create or write to file";
    let output_file = File::create(filename).expect(file_err_msg);
    let mut buf = BufWriter::new(output_file);

    // Write number of key images and pks on first line
    writeln!(buf, "{} {}", tx_rings.len(), num_pks).expect(file_err_msg);

    for ki_index in 0..tx_rings.len() {
        write!(buf, "{}", ki_index).expect(file_err_msg);
        for pk_index in &tx_rings[ki_index]  {
            write!(buf, " {}", pk_index).expect(file_err_msg);
        }
        writeln!(buf, "").expect(file_err_msg);
    }
}

// Write a list of ring sets to an output file.
// The first line of the file has the number of rings and number of distinct
// public keys separated by a single space.
// Each subsequent line begins with the index of the key image, followed by
// the indices of the public keys in the rings. The indices are separated by
// a single space. The key image indices range from 0 to one less than the
// number of rings. The public key indices range from 0 to one less than
// the number of distinct public keys.
pub fn write_ring_sets(
    tx_ring_sets: &Vec<HashSet<u32>>,
    num_pks: usize,
    filename: &String,
) -> () {
    let file_err_msg = "Unable to create or write to file";
    let output_file = File::create(filename).expect(file_err_msg);
    let mut buf = BufWriter::new(output_file);

    // Write number of key images and pks on first line
    writeln!(buf, "{} {}", tx_ring_sets.len(), num_pks).expect(file_err_msg);

    for ki_index in 0..tx_ring_sets.len() {
        write!(buf, "{}", ki_index).expect(file_err_msg);
        for pk_index in &tx_ring_sets[ki_index]  {
            write!(buf, " {}", pk_index).expect(file_err_msg);
        }
        writeln!(buf, "").expect(file_err_msg);
    }
}

// Function finds a maximum matching in a graph in the case
// when every column is matched.
// The output is a vector of u32 values where a value of
// u32::MAX at index i implies that the ith row is unmatched.
// Otherwise, the value equals the index of the column that is
// matched to row i.
pub fn cn_txgraph_maximum_matching(
    g: &CsMatI<u8, u32>,
) -> Vec<u32> {
    let num_rows = g.rows();
    let num_cols = g.cols();
    let col_ptr = g.indptr();
    let row_ptr_slice = g.indices();

    let col_ptr_slice = col_ptr.as_slice().unwrap();
    let mut row_mates = vec![u32::MAX; num_rows];

    let mut dfs_stack = vec![0_u32; num_cols];
    let mut col_mates_in_dfs_stack = vec![0_u32; num_cols];
    let mut next_dfs_row = vec![0_u32; num_cols];
    let mut visited_cols = vec![u32::MAX; num_cols];
    let mut initial_row_for_search : Vec<u32> = col_ptr_slice.to_vec();
    initial_row_for_search.pop(); // We do not need the last element
    
    let mut found_unmatched_row: bool;
    let mut head: i64;
    let mut p: i64;

    for current_col in 0..num_cols as u32 {
        dfs_stack[0] = current_col;
        found_unmatched_row = false;
        head = 0_i64;

        while head >= 0 {
            let col_idx = dfs_stack[head as usize];
            let mut row_idx: u32 = u32::MAX;

            if visited_cols[col_idx as usize] != current_col {
                visited_cols[col_idx as usize] = current_col;

                p = initial_row_for_search[col_idx as usize] as i64;
                while (p as u32) < col_ptr_slice[(col_idx+1) as usize] && !found_unmatched_row {
                    row_idx = row_ptr_slice[p as usize];
                    found_unmatched_row = row_mates[row_idx as usize] == u32::MAX;
                    p += 1;
                }
                initial_row_for_search[col_idx as usize] = p as u32;
                
                if found_unmatched_row {
                    col_mates_in_dfs_stack[head as usize] = row_idx;
                    break;
                }
                next_dfs_row[head as usize] = col_ptr_slice[col_idx as usize];
            }

            p = next_dfs_row[head as usize] as i64;
            while (p as u32) < col_ptr_slice[(col_idx + 1) as usize] {
                row_idx = row_ptr_slice[p as usize];
                if visited_cols[row_mates[row_idx as usize] as usize] == current_col {
                    p += 1;
                    continue;
                }
                next_dfs_row[head as usize] = (p+1) as u32;
                col_mates_in_dfs_stack[head as usize] = row_idx;
                head += 1;
                dfs_stack[head as usize] = row_mates[row_idx as usize];
                break;
            }
            if (p as u32) == col_ptr_slice[(col_idx+1) as usize] {
                head -= 1;
            }
        }
        if found_unmatched_row {
            p = head;
            while p >= 0 {
                row_mates[col_mates_in_dfs_stack[p as usize] as usize] = dfs_stack[p as usize];
                p -= 1;
            }
        }
    }

    return row_mates;
}

pub fn remove_closed_set_pks(
    closed_keyimage_set: HashSet<u32>,
    closed_pk_set: HashSet<u32>,
    tx_ring_sets: &mut Vec<HashSet<u32>>,
    pk_to_ki_map: &Vec<Vec<u32>>,
) -> bool {
    let mut anonymity_set_reduced = false;

    for pk in &closed_pk_set {
        for ki in &pk_to_ki_map[*pk as usize] {
            if closed_keyimage_set.contains(ki) == false {
                if tx_ring_sets[*ki as usize].remove(pk) {
                    anonymity_set_reduced = true;
                };
            } 
        }
    }

    return anonymity_set_reduced;
}