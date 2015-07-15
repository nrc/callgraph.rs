// Copyright 2015 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use rustc_graphviz as graphviz;
use rustc_graphviz::{Labeller, GraphWalk};

use std::fs::File;
use std::iter::FromIterator;

use syntax::ast::NodeId;

use visitor::RecordVisitor;

impl<'l, 'tcx: 'l> RecordVisitor<'l, 'tcx> {
    // Make a graphviz dot file.
    // Must be called after post_process.
    pub fn dot(&self) {
        // TODO use crate name 
        let mut file = File::create("out.dot").unwrap();
        graphviz::render(self, &mut file).unwrap();
    }
}

// Graphviz interaction.
//
// We use NodeIds to identify nodes in the graph to Graphviz. We label them by
// looking up the name for the id in self.functions. Edges are the union of
// static and dynamic calls. We don't label edges, but potential calls due to
// dynamic dispatch get dotted edges.
//
// Invariants: all edges must be beween nodes which are in self.functions.
//             post_process must have been called (i.e., no decls left in the graph)

// Whether a call certainly happens (e.g., static dispatch) or only might happen
// (e.g., all possible receiving methods of dynamic dispatch).
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum CallKind {
    Definite,
    Potential,
}

// An edge in the callgraph, only used with graphviz.
pub type Edge = (NodeId, NodeId, CallKind);

// Issues ids, labels, and styles for graphviz.
impl<'a, 'l, 'tcx: 'l> Labeller<'a, NodeId, Edge> for RecordVisitor<'l, 'tcx> {
    fn graph_id(&'a self) -> graphviz::Id<'a> {
        graphviz::Id::new("Callgraph_for_TODO").unwrap()
    }

    fn node_id(&'a self, n: &NodeId) -> graphviz::Id<'a> {
        graphviz::Id::new(format!("n_{}", n)).unwrap()
    }

    fn node_label(&'a self, n: &NodeId) -> graphviz::LabelText<'a> {
        // To find the label, we just lookup the function name.
        graphviz::LabelText::label(&*self.functions[n])
    }

    // TODO styles
}

// Drives the graphviz visualisation.
impl<'a, 'l, 'tcx: 'l> GraphWalk<'a, NodeId, Edge> for RecordVisitor<'l, 'tcx> {
    fn nodes(&'a self) -> graphviz::Nodes<'a, NodeId> {
        graphviz::Nodes::from_iter(self.functions.keys().cloned())
    }

    fn edges(&'a self) -> graphviz::Edges<'a, Edge> {
        let static_iter = self.static_calls.iter().map(|&(ref f, ref t)| (f.clone(),
                                                                          t.clone(),
                                                                          CallKind::Definite));
        let dyn_iter = self.dynamic_calls.iter().map(|&(ref f, ref t)| (f.clone(),
                                                                        t.clone(),
                                                                        CallKind::Potential));
        graphviz::Edges::from_iter(static_iter.chain(dyn_iter))
    }

    fn source(&'a self, &(from, _, _): &Edge) -> NodeId {
        from
    }

    fn target(&'a self, &(_, to, _): &Edge) -> NodeId {
        to
    }
}
