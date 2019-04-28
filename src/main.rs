use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::fs::File;
use std::io;

use fxhash::FxBuildHasher;

use petgraph::algo::{astar};
use petgraph::graph::{Graph, EdgeIndex, NodeIndex};

use serde::Deserialize;

use structopt::StructOpt;

#[allow(non_snake_case)]
#[derive(Debug, Deserialize)]
struct Person {
    PersonID: i32,
    SpouseID: Option<i32>,
    FatherID: Option<i32>,
    MotherID: Option<i32>,

    // PersonID: String,
    /// Name of the person.
    Person: String,
    // SpouseID: String,
    // Ehepartner: String,
    // FatherID: String,
    // Vater: String,
    // MotherID: String,
    // Mutter: String,
    // ChildID: String,
    // Kind: String,
    // RelID: String,
    // Beziehung: String,
    // RelationKey: String,
}


#[derive(Debug)]
struct Heritage {
    person: Person,
    node_idx: NodeIndex<u32>,
}


#[derive(Copy, Clone, Debug)]
enum Relationship {
    Spouse,
    Father,
    Mother,
}


#[derive(Debug)]
struct PersonRelationship {
    id: i32,
    name: String,
    relationship: Option<Relationship>,
}


type HeritageMap = HashMap<i32, Heritage, FxBuildHasher>;

// Store person id per node, and relationship type as edge information.
// Undirected to allow for indirect heritage paths.
// u32 index space, if you have more than 4B nodes change.
type PersonGraph = Graph<i32, Relationship, petgraph::Undirected, u32>;


impl fmt::Display for PersonRelationship {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.relationship {
            Some(rel) => {
                write!(f, "-> {}({}) is {:?} of", self.name, self.id, rel)
            },
            None => { write!(f, "-> {}({})", self.name, self.id) }
        }
    }
}


fn add_persons(
    heritage: &mut Heritage,
    person: &Person
) {
    // TODO find a better way to merge structs holding multiple Options.
    if person.SpouseID.is_some() {
        heritage.person.SpouseID = person.SpouseID;
    }

    if person.FatherID.is_some() {
        heritage.person.FatherID = person.FatherID;
    }

    if person.MotherID.is_some() {
        heritage.person.MotherID = person.MotherID;
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
        let person: Person = result?;
        let person_id: i32 = person.PersonID;

        match heritage_map.get_mut(&person_id) {
            Some(mut heritage) => {
                add_persons(&mut heritage, &person);
            }
            None => {
                let heritage = Heritage{
                    person,
                    node_idx: graph.add_node(person_id),
                };

                heritage_map.insert(person_id, heritage);
            }
        }
    }

    Ok((graph, heritage_map))
}


fn add_graph_edges(graph: &mut PersonGraph, heritage_map: &HeritageMap) {
    for heritage in heritage_map.values() {
        let mut add_optional_person = |relative_opt, rel| {
            if let Some(relative_id) = relative_opt {
                if let Some(relative) = heritage_map.get(&relative_id) {
                    graph.add_edge(heritage.node_idx, relative.node_idx, rel);
                }
            }
        };

        add_optional_person(heritage.person.SpouseID, Relationship::Spouse);
        add_optional_person(heritage.person.FatherID, Relationship::Father);
        add_optional_person(heritage.person.MotherID, Relationship::Mother);
    }
}


// Pulls node + optional edge information from node indices.
// Imo this should be library feature.
fn map_edges(
    nodes: &[NodeIndex<u32>],
    graph: &PersonGraph
) -> Vec<(i32, Option<EdgeIndex<u32>>)> {
    let mut indicies = nodes.iter().rev().peekable();
    let mut vec = Vec::new();

    while let Some(a) = indicies.next() {
        let edge_opt = indicies.peek()
            .map(|b| { graph.find_edge(*a, **b) })
            .unwrap_or(None);

        vec.push((graph[*a], edge_opt));
    }

    vec
}


fn get_shortest_path(
    graph: &PersonGraph,
    heritage_map: &HeritageMap,
    child_id: i32,
    ancestor_id: i32
) -> Result<Vec<PersonRelationship>, Box<Error>> {
    let start_idx = heritage_map.get(&child_id)
        .ok_or("invalid start id")?.node_idx;

    let finish_idx = heritage_map.get(&ancestor_id)
        .ok_or("invalid finish id")?.node_idx;

    let nodes = astar(&graph, start_idx, |e| e == finish_idx, |_| 1, |_| 0)
        .map(|(_cost, nodes)| { nodes })
        .ok_or("no direct or indirect relationship found")?;

    let lookup_name = |person_id| {
        heritage_map.get(&person_id)
            .map(|heritage| { heritage.person.Person.clone() })
            .ok_or("invalid person_id")
    };

    let mut rels = Vec::new();

    for (person_id, edge_opt) in map_edges(&nodes, &graph) {
        rels.push(PersonRelationship{
            id: person_id,
            name: lookup_name(person_id)?,
            relationship: edge_opt.map(|edge| {
                graph.edge_weight(edge).cloned()
            }).unwrap_or(None)
        });
    }

    Ok(rels)
}


fn fmt_person_relationships(rels: &[PersonRelationship]) -> String {
    // Imperative style would probably be faster and easier and maintain.
    rels.iter()
        .map(|person_relationship| { format!("{}", person_relationship) })
        .collect::<Vec<String>>()
        .join("\n")
}


#[derive(Debug, StructOpt)]
#[structopt(name = "heritage-pathfind", about = "Find person path")]
struct CmdInput {
    #[structopt(short = "r", long = "relationship-csv")]
    csv_path: String,
    #[structopt(short = "c", long = "child-id")]
    child_id: i32,
    #[structopt(short = "a", long = "ancestor-id")]
    ancestor_id: i32,
}


fn main() -> Result<(), Box<Error>> {
    let cmd_input = CmdInput::from_args();

    let csv_file = File::open(cmd_input.csv_path)?;

    let (mut graph, heritage_map) = extract_graph_from_csv(csv_file)?;
    add_graph_edges(&mut graph, &heritage_map);

    get_shortest_path(
        &graph,
        &heritage_map,
        cmd_input.child_id,
        cmd_input.ancestor_id
    )
        .map(|person_relationships| {
            println!("{}", fmt_person_relationships(&person_relationships));
        })
}


// TODO test with csv.
#[test]
fn it_adds_two() {
    assert_eq!([1, 2], vec![1, 2, 3].as_slice());
}
