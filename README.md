# cryptonote-analysis

This repository contains the programs needed to perform the analysis described in the paper [Analysis of CryptoNote Transaction Graphs using the Dulmage-Mendelsohn Decomposition](https://eprint.iacr.org/2021/760). Instructions on how to use these programs can be found at [here](https://www.respectedsir.com/cna).

These instructions are maintained by [Saravanan Vijayakumaran](https://www.ee.iitb.ac.in/~sarva/). They have been tested only on  Ubuntu 22.04. If you find a bug in these instructions, please file an [issue](https://github.com/avras/cryptonote-analysis/issues/).

The scripts (Python, Bash, SQL, C++) and programs are organized as follows.
```bash
cryptonote-analysis
├── Cargo.lock
├── Cargo.toml
├── scripts
│   ├── hardforks
│   │   ├── find_ring_intersection_from_forks.py
│   │   ├── monero-original
│   │   │   ├── create_keyimages_table.sql
│   │   │   ├── create_outputs_table.sql
│   │   │   ├── find_xmr_xmo_addresses.py
│   │   │   ├── populate_xmo_tables.py
│   │   │   ├── sql_table_creation.sh
│   │   │   └── trim_ring_xmr_xmo.py
│   │   ├── monerov
│   │   │   ├── create_keyimages_table.sql
│   │   │   ├── create_outputs_table.sql
│   │   │   ├── Dockerfile
│   │   │   ├── find_xmr_xmv_addresses.py
│   │   │   ├── populate_xmv_tables.py
│   │   │   ├── sql_table_creation.sh
│   │   │   └── trim_ring_xmr_xmv.py
│   │   ├── monero-v7
│   │   │   ├── create_keyimages_table.sql
│   │   │   ├── create_outputs_table.sql
│   │   │   ├── find_xmr_xmrv7_addresses.py
│   │   │   ├── populate_xmrv7_tables.py
│   │   │   ├── sql_table_creation.sh
│   │   │   └── trim_ring_xmr_xmrv7.py
│   │   └── monero-v9
│   │       ├── create_keyimages_table.sql
│   │       ├── create_outputs_table.sql
│   │       ├── find_xmr_xmrv9_addresses.py
│   │       ├── populate_xmrv9_tables.py
│   │       ├── sql_table_creation.sh
│   │       └── trim_ring_xmr_xmrv9.py
│   └── monero
│       ├── create_csparse_edges.cpp
│       ├── create_keyimages_table.sql
│       ├── create_outputs_table.sql
│       ├── keyimage_table_creation.sh
│       ├── output_table_creation.sh
│       └── populate_keyimage_table.py
└── src
    ├── bin
    │   ├── cascade.rs
    │   ├── cluster.rs
    │   ├── dmdec.rs
    │   ├── stats_cla.rs
    │   └── stats_dm.rs
    └── lib.rs
```