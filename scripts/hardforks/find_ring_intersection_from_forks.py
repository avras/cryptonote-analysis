import psycopg2


def main():
    conn = psycopg2.connect(database="postgres", user="postgres", password="blah", host="localhost")
    cur = conn.cursor()

    xmr_image_fork_indices_dict = {}

    cur.execute('SELECT * FROM (SELECT * FROM xmr_xmv_keyimages UNION SELECT * FROM xmr_xmo_keyimages UNION SELECT * FROM xmr_xmrv7_keyimages UNION SELECT * FROM xmr_xmrv9_keyimages) as un;')
    xmr_records= cur.fetchall()
    for (image, id, ring_amount, ring_indices, distinct_ring_indices, block_height, fork_indices) in xmr_records:
        if image in xmr_image_fork_indices_dict:
            old_indices = xmr_image_fork_indices_dict[image]
            xmr_image_fork_indices_dict[image] = list(set(old_indices) & set(fork_indices))
            #print(image, old_indices, fork_indices, xmr_image_fork_indices_dict[image])
        else:
            xmr_image_fork_indices_dict[image] = fork_indices
            #print(image, fork_indices)

    for (i, fidx) in xmr_image_fork_indices_dict.items():
        # print(i, fidx)
        cur.execute('UPDATE xmr_keyimages SET fork_indices=%s WHERE image=%s', (f'{{{", ".join(map(str, fidx))}}}', i))

    conn.commit()
    cur.close()
    conn.close()
            


if __name__ == "__main__":
    main()
