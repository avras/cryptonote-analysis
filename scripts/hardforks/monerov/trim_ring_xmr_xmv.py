import pickle
import psycopg2
import numpy as np

def main():
    xmr_output_file = open('xmr_xmv_addr.dat', 'rb')
    xmr_amt_idx_out_dict = pickle.load(xmr_output_file)
    xmr_out_amt_idx_dict = {}
    for ((a,i), addr) in xmr_amt_idx_out_dict.items():
        xmr_out_amt_idx_dict[addr] = (a,i)

    conn = psycopg2.connect(database="postgres", user="postgres", password="blah", host="localhost")
    cur = conn.cursor()

    cur.execute('SELECT * FROM xmr_xmv_keyimages;')
    xmr_records= cur.fetchall()
    for (image, id, ring_amount, ring_indices, distinct_ring_indices, block_height, fork_indices) in xmr_records:
        print(block_height)
        xmr_ring_addresses = []
        for i in distinct_ring_indices:
            xmr_ring_addresses.append(xmr_amt_idx_out_dict[(ring_amount, i)])
        cur.execute('SELECT * FROM xmv_keyimages WHERE image=%s;', (image,))
        xmv_record = cur.fetchall()[0]
        other_ring_indices = xmv_record[-2]
        other_ring_amount = xmv_record[2]
        assert(other_ring_amount == ring_amount)
        xmv_ring_addresses = []
        for index in other_ring_indices:
            cur.execute('SELECT * FROM xmv_outputs WHERE amount=%s AND index=%s', (other_ring_amount, index))
            xmv_output = cur.fetchall()[0]
            xmv_ring_addresses.append(xmv_output[0])
        int_ring_addresses = list(set(xmr_ring_addresses) & set(xmv_ring_addresses))
        int_ring_indices = list(map(lambda a: xmr_out_amt_idx_dict[a][1], int_ring_addresses))
        
        cur.execute('UPDATE xmr_xmv_keyimages SET fork_indices=%s WHERE image=%s', (f'{{{", ".join(map(str, int_ring_indices))}}}', image))
    
    cur.execute('SELECT ARRAY_LENGTH(fork_indices,1) FROM xmr_xmv_keyimages;')
    result = cur.fetchall()
    int_ring_indices_lengths = list(map(lambda x: x[0], result))
    max_length = max(int_ring_indices_lengths)
    hist = np.bincount(int_ring_indices_lengths)
    print(hist)
    for i in range(max_length+1):
        if hist[i] !=0:
            print(i, hist[i])
    
    print('Total number of images = ', sum(hist))
    print('Intersection size 1 = ', hist[1])
    print('Rest = ', sum(hist[2:max_length+1]))


    conn.commit()
    cur.close()
    conn.close()
            


if __name__ == "__main__":
    main()

