use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io;

use fxhash::FxBuildHasher;

use petgraph::algo::{astar};
use petgraph::graph::{Graph, NodeIndex};

use serde::Deserialize;

use structopt::StructOpt;

#[allow(non_snake_case)]
#[derive(Debug, Deserialize)]
struct Relationship {
    PersonID: i32,
    SpouseID: Option<i32>,
    FatherID: Option<i32>,
    MotherID: Option<i32>,

    //PersonID: String,
    Person: String,
    // SpouseID: String,
    // Ehepartner: String,
    //FatherID: String,
    // Vater: String,
    //MotherID: String,
    // Mutter: String,
    // ChildID: String,
    // Kind: String,
    // RelID: String,
    Beziehung: String,
    //RelationKey: String,
}

#[derive(Debug)]
struct Heritage {
    relationship: Relationship,
    node_idx: NodeIndex<u32>,
}

type HeritageMap = HashMap<i32, Heritage, FxBuildHasher>;

// Store person id per node, edges need no information.
// Undirected to allow for indirect heritage paths.
// u32 index space, if you have more than 4B nodes change.
type PersonGraph = Graph<i32, (), petgraph::Undirected, u32>;


fn add_relationships(
    heritage: &mut Heritage,
    relationship: &Relationship
) {
    // TODO find a better way to merge structs holding multiple Options.
    if let Some(_) = relationship.SpouseID {
        heritage.relationship.SpouseID = relationship.SpouseID;
    }

    if let Some(_) = relationship.FatherID {
        heritage.relationship.FatherID = relationship.FatherID;
    }

    if let Some(_) = relationship.MotherID {
        heritage.relationship.MotherID = relationship.MotherID;
    }
}


// Build up graph and companion data structure while parsing csv.
// Not the most beautiful approach, yet should help avoiding unnecessary copies.
fn extract_graph_from_csv<R: io::Read>(
    rdr: R
) -> Result<(PersonGraph, HeritageMap), Box<Error>> {
    let mut rdr = csv::ReaderBuilder::new()
        .delimiter(b';')
        .from_reader(rdr);

    let mut graph = PersonGraph::default();
    let mut heritage_map = HeritageMap::default();

    for result in rdr.deserialize() {
        let relationship: Relationship = result?;
        let person_id: i32 = relationship.PersonID;

        match heritage_map.get_mut(&person_id) {
            Some(mut heritage) => {
                add_relationships(&mut heritage, &relationship);
            }
            None => {
                let heritage = Heritage{
                    relationship: relationship,
                    node_idx: graph.add_node(person_id),
                };

                heritage_map.insert(person_id, heritage);
            }
        }
    }

    Ok((graph, heritage_map))
}


fn add_graph_edges(graph: &mut PersonGraph, heritage_map: &HeritageMap) {
    for (_, heritage) in heritage_map {
        let mut add_optional_relationship = |relative_opt| {
            if let Some(relative_id) = relative_opt {
                if let Some(relative) = heritage_map.get(&relative_id) {
                    graph.add_edge(heritage.node_idx, relative.node_idx, ());
                }
            }
        };

        add_optional_relationship(heritage.relationship.SpouseID);
        add_optional_relationship(heritage.relationship.FatherID);
        add_optional_relationship(heritage.relationship.MotherID);
    }
}


fn get_shortest_path(
    graph: &PersonGraph,
    heritage_map: &HeritageMap,
    start_id: i32,
    finish_id: i32
) -> Result<Vec<i32>, Box<Error>> {
    let start_idx = heritage_map.get(&start_id).ok_or("invalid start id")?.node_idx;
    let finish_idx = heritage_map.get(&finish_id).ok_or("invalid finish id")?.node_idx;

    let path_opt = astar(&graph, start_idx, |e| e == finish_idx, |_| 2, |_| 0)
        .map(|(_cost, indices)| {
            indices.iter().map(|idx| { graph[*idx] }).collect()
        })
        .ok_or("no direct or indirect relationship found");

    Ok(path_opt?)
}


fn fmt_person_id(person_id: i32, heritage_map: &HeritageMap) -> String {
    let rel = &heritage_map.get(&person_id).unwrap().relationship;

    format!("{}({}) {} -> ", rel.Person, person_id, rel.Beziehung)
}


fn fmt_person_ids(person_ids: &Vec<i32>, heritage_map: &HeritageMap) -> String {
    person_ids.iter()
        .map(|person_id| { fmt_person_id(*person_id, &heritage_map) })
        .collect::<Vec<String>>()
        .join("\n")
}


#[derive(Debug, StructOpt)]
#[structopt(name = "heritage-pathfind", about = "Find relationship path")]
struct CmdInput {
    #[structopt(short = "c", long = "relationship-csv")]
    csv_path: String,
    #[structopt(short = "s", long = "start-id")]
    start_id: i32,
    #[structopt(short = "f", long = "finish-id")]
    finish_id: i32,
}


fn main() -> Result<(), Box<Error>> {
    let cmd_input = CmdInput::from_args();

    let csv_file = File::open(cmd_input.csv_path)?;

    let (mut graph, heritage_map) = extract_graph_from_csv(csv_file)?;
    add_graph_edges(&mut graph, &heritage_map);

    get_shortest_path(
        &graph,
        &heritage_map,
        cmd_input.start_id,
        cmd_input.finish_id
    )
        .map(|person_ids| {
            println!("{}", fmt_person_ids(&person_ids, &heritage_map));
            ()
        })
}


// TODO test with csv.
#[test]
fn it_adds_two() {
    assert_eq!([1, 2], vec![1, 2, 3].as_slice());
}
