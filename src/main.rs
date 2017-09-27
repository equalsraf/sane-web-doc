
#[macro_use]
extern crate html5ever;
use html5ever::parse_document;
use html5ever::rcdom::{NodeData, RcDom, Handle};
use html5ever::tendril::TendrilSink;
use html5ever::serialize::{Serialize, SerializeOpts, serialize};

extern crate string_cache;

use std::io;

#[derive(Debug)]
pub struct Stats {
    decorative_nodes: usize,
    depths: Vec<usize>,
    script_tags: usize,
    nav_tags: usize,
}

const NONDECORATIVE: &[string_cache::atom::Atom<html5ever::LocalNameStaticSet>] = &[
    local_name!("li"), local_name!("svg"), local_name!("a"), local_name!("table"), local_name!("tbody"),
    local_name!("h1"), local_name!("h2"), local_name!("h3"),
    local_name!("td"),
];

fn walk(level: usize, handle: Handle, stats: &mut Stats) {
    let node = handle;

    let mut print_tag = false;
    let children = node.children.borrow();

    match node.data {
        NodeData::Document => (),
        NodeData::Doctype { ref name, ref public_id, ref system_id } => (),
        NodeData::Text { ref contents } => (),
        NodeData::Comment { ref contents } => (),
        NodeData::Element { ref name, ref attrs, .. } => {
            
            if name.local == local_name!("script") {
                stats.script_tags += 1;
            }

            if name.local == local_name!("nav") {
                stats.nav_tags += 1;
            }

            if children.len() == 1 {
                if !NONDECORATIVE.contains(&name.local) {
                    match children[0].clone().data {
                        NodeData::Text {..} => (),
                        _ => {
                            stats.decorative_nodes += 1;
                            print_tag = true;
                        }
                    }
                }
            }
        }

        NodeData::ProcessingInstruction { .. } => unreachable!()
    }

    if print_tag {
        println!("\n BAD tag ========");
        serialize(io::stdout(), &node, SerializeOpts {
            scripting_enabled: true,
            traversal_scope: html5ever::serialize::TraversalScope::IncludeNode,
            create_missing_parent: true,
        });
    }
    for child in children.iter() {
        walk(level+1, child.clone(), stats);
    }

    if children.len() == 0 {
        stats.depths.push(level);
    }
}

fn main() {
    let stdin = io::stdin();
    let dom = parse_document(RcDom::default(), Default::default())
        .from_utf8()
        .read_from(&mut stdin.lock())
        .unwrap();

    let mut s = Stats {
        decorative_nodes: 0,
        depths: Vec::new(),
        script_tags: 0,
        nav_tags: 0,
    };
    walk(0, dom.document, &mut s);

    if !dom.errors.is_empty() {
        println!("\nParse errors:");
        for err in dom.errors.into_iter() {
            println!("    {}", err);
        }
    }

    println!("\n");
    println!("<script> tags: {}", s.script_tags);
    println!("decorative nodes: {}", s.decorative_nodes);
    let sum: usize = s.depths.iter().sum();
    println!("average depth: {}", sum / s.depths.len());
    println!("max depth: {}", s.depths.iter().max().unwrap_or(&0));
    println!("<nav> tags: {}", s.nav_tags);
}
