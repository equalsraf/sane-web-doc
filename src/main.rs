
#[macro_use]
extern crate html5ever;
use html5ever::parse_document;
use html5ever::rcdom::{NodeData, RcDom, Handle};
use html5ever::tendril::TendrilSink;
use html5ever::serialize::{Serialize, SerializeOpts, serialize};

use std::io;

#[derive(Debug)]
pub struct Stats {
    decorative_nodes: usize,
    depth: usize,
    script_tags: usize,
}

fn walk(level: usize, handle: Handle, stats: &mut Stats) {

    if stats.depth < level {
        stats.depth = level;
    }

    let node = handle;

    let mut print_tag = false;
    let children = node.children.borrow();

    match node.data {
        NodeData::Document => (),
        NodeData::Doctype { ref name, ref public_id, ref system_id } => (),
        NodeData::Text { ref contents } => (),
        NodeData::Comment { ref contents } => (),
        NodeData::Element { ref name, ref attrs, .. } => {
            
            if attrs.borrow().len() == 0 && children.len() == 1 {
                if [local_name!("div"), local_name!("span")].contains(&name.local) {
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
        println!("\n BAD tag ========\n");
        serialize(io::stdout(), &node, SerializeOpts {
            scripting_enabled: true,
            traversal_scope: html5ever::serialize::TraversalScope::IncludeNode,
            create_missing_parent: true,
        });
    }
    for child in children.iter() {
        walk(level+1, child.clone(), stats);
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
        depth: 0,
        script_tags: 0,
    };
    walk(0, dom.document, &mut s);

    if !dom.errors.is_empty() {
        println!("\nParse errors:");
        for err in dom.errors.into_iter() {
            println!("    {}", err);
        }
    }

    println!("\n{:#?}", s);
}
