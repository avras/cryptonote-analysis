import requests
import psycopg2
import pickle

def keyoffsets_to_keyindices(key_offsets):
    key_indices = [key_offsets[0]]
    for x in key_offsets[1:]:
        key_indices.append(key_indices[-1] + x)
    return key_indices

def main():
    url_getouts = 'http://127.0.0.1:18081/get_outs'
    getouts_payload = {
        "outputs": [
            {
                "amount": 0,
                "index": 0
            }
        ]
    }

    conn = psycopg2.connect(database="postgres", user="postgres", password="blah", host="localhost")
    cur = conn.cursor()

    xmr_output_addresses = {}

    cur.execute('SELECT * FROM xmr_xmrv7_keyimages;')
    xmr_records= cur.fetchall()
    for (image, id, ring_amount, ring_indices, distinct_ring_indices, block_height, fork_indices) in xmr_records:
        for index in distinct_ring_indices:
            if (ring_amount, index) not in xmr_output_addresses:
                getouts_payload['outputs'] = [{"amount": ring_amount, "index": index}]
                rout = requests.post(url_getouts, json=getouts_payload).json()
                address = rout['outs'][0]['key']
                # print(ring_amount, index, address)
                xmr_output_addresses[(ring_amount, index)] = address


    conn.commit()
    cur.close()
    conn.close()

    xmr_addr_file = open(r'xmr_xmrv7_addr.dat','wb')
    pickle.dump(xmr_output_addresses, xmr_addr_file)
    xmr_addr_file.close()



if __name__ == "__main__":
    main()

