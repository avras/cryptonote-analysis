#include <iostream>                  // for std::cout
#include <utility>                   // for std::pair
#include <algorithm>                 // for std::for_each
#include <cassert>
#include <vector>
#include <algorithm>
#include <set>
#include <map>
#include <fstream>

using namespace std;

int main(int argc, char** argv)
{ 
    if (argc < 2) {
        cout << "Not enough arguments. Need at least one argument." << endl;
        cout << "Specify block height as first argument and an optional filename prefix as the second argument." << endl;
        cout << "For example: ./a.out 1541236 ringct" << endl;
        exit(1);
    }

    typedef pair<int, int> Edge;
    vector<Edge> edge_vector;
    set<int> keyimage_id_set;
    set<int> output_id_set;
    string id_edges_filename= "";
    if (argc > 2) {
      id_edges_filename.append(argv[2]);
      id_edges_filename.append("-edges-");
    } else {
      id_edges_filename.append("edges-");
    }
    id_edges_filename.append(argv[1]);
    id_edges_filename.append(".txt");

    string csparse_edges_filename = "";
    if (argc > 2) {
      csparse_edges_filename.append(argv[2]);
      csparse_edges_filename.append("-csparse-edges-");
    } else {
      csparse_edges_filename.append("csparse-edges-");
    }
    csparse_edges_filename.append(argv[1]);
    csparse_edges_filename.append(".txt");

    // string index_keyimageid_filename = "index-keyimageid-map-";
    string index_keyimageid_filename = "";
    if (argc > 2) {
      index_keyimageid_filename.append(argv[2]);
      index_keyimageid_filename.append("-index-keyimageid-map-");
    } else {
      index_keyimageid_filename.append("index-keyimageid-map-");
    }
    index_keyimageid_filename.append(argv[1]);
    index_keyimageid_filename.append(".txt");

    // string index_outputid_filename = "index-outputid-map-";
    string index_outputid_filename = "";
    if (argc > 2) {
      index_outputid_filename.append(argv[2]);
      index_outputid_filename.append("-index-outputid-map-");
    } else {
      index_outputid_filename.append("index-outputid-map-");
    }
    index_outputid_filename.append(argv[1]);
    index_outputid_filename.append(".txt");

    ifstream id_edges_file(id_edges_filename);
    ofstream csparse_edges_file(csparse_edges_filename);
    ofstream index_keyimageid_file(index_keyimageid_filename);
    ofstream index_outputid_file(index_outputid_filename);

    cout << "Reading edge file" << endl;
    int a, b;
    while (id_edges_file >> a >> b) {
      keyimage_id_set.insert(a);
      output_id_set.insert(b);
      edge_vector.push_back(Edge(a, b));
    }
    cout << "Finished reading edge file" << endl;
  
    size_t num_edges = edge_vector.size();
    size_t num_keyimages = keyimage_id_set.size();
    size_t num_outputs = output_id_set.size();
    cout << "Number of key images: " << num_keyimages << endl;
    cout << "Number of outputs: " << num_outputs << endl;
  
    size_t num_vertices = num_keyimages+num_outputs;
    cout << "Number of vertices: " << num_vertices << endl;
    cout << "Number of edges: " << num_edges << endl;
  
    map<int, int> keyimage_index_map;
    map<int, int> output_index_map;
  
    cout << "Creating keyimage index map" << endl;
    set<int>::iterator it;
    int index = 0;
    for (it=keyimage_id_set.begin(); it != keyimage_id_set.end(); ++it) {
      keyimage_index_map.insert(make_pair(*it, index));
      index_keyimageid_file << index << " " << *it << endl;
      index = index + 1;
    }
    cout << "Finished creating keyimage index map" << endl;
  
    cout << "Creating output index map" << endl;
    index = 0;
    for (it=output_id_set.begin(); it != output_id_set.end(); ++it) {
      output_index_map.insert(make_pair(*it, index));
      index_outputid_file << index << " " << *it << endl;
      index = index + 1;
    }
    cout << "Finished creating output index map" << endl;
  
    cout << "Adding edges to graph" << endl;
    // add the edges to the output file
    for (int i = 0; i < num_edges; ++i)
    {
      csparse_edges_file << keyimage_index_map[edge_vector.at(i).first] << " " << output_index_map[edge_vector.at(i).second] << " 1" << endl; 
    }
    cout << "Finished adding edges to graph" << endl;
  
  
    id_edges_file.close();
    csparse_edges_file.close();
    return 0;
}
