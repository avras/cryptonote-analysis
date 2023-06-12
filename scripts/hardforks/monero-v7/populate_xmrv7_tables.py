import requests
import json
import psycopg2

def keyoffsets_to_keyindices(key_offsets):
    key_indices = [key_offsets[0]]
    for x in key_offsets[1:]:
        key_indices.append(key_indices[-1] + x)
    return key_indices

def main():
    min_height = 1685555
    max_height = 1685583
    url_getblock = 'http://127.0.0.1:18081/json_rpc'
    url_gettransactions = 'http://127.0.0.1:18081/gettransactions'
    url_getouts = 'http://127.0.0.1:18081/get_outs'
    getblock_payload = {
        "method": "getblock",
        "jsonrpc": "2.0",
        "id": "0"
    }

    gettransactions_payload = {
        "decode_as_json":True
    }
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

    for i in range(min_height, max_height+1):
        getblock_payload["params"] = {"height":str(i)}
        r = requests.post(url_getblock, json=getblock_payload).json()
        blockheight = r['result']['block_header']['height']
        blockjson = json.loads(r['result']['json'])
        tx_hashes = blockjson['tx_hashes']
        if tx_hashes != []:
            print(blockheight)
            gettransactions_payload['txs_hashes'] = tx_hashes
            rt = requests.post(url_gettransactions, json=gettransactions_payload).json()
            for tx in rt['txs']:
                txjson = json.loads(tx['as_json'])
                txinputs = txjson['vin']
                for txi in txinputs:
                    amount = txi['key']['amount']
                    keyimage = txi['key']['k_image']
                    keyoffsets = txi['key']['key_offsets']
                    keyindices = keyoffsets_to_keyindices(keyoffsets)
                    #print(amount, keyimage, keyoffsets, keyindices)

                    cur.execute('INSERT INTO xmrv7_keyimages (image, ring_amount, ring_indices, block_height) VALUES(%s, %s, %s, %s) ON CONFLICT(image) DO NOTHING;', (keyimage, amount, keyindices, blockheight))
                    for ki in keyindices:
                        getouts_payload['outputs'] = [{"amount": amount, "index": ki}]
                        rout = requests.post(url_getouts, json=getouts_payload).json()
                        address = rout['outs'][0]['key']
                        cur.execute('INSERT INTO xmrv7_outputs (address, amount, index) VALUES(%s, %s, %s) ON CONFLICT(amount, index) DO NOTHING;', (address, amount, ki))
            conn.commit()
    
    cur.close()
    conn.close()
            


if __name__ == "__main__":
    main()
