// Copyright 2015 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use graphviz::{self, Labeller, GraphWalk};

use rustc::middle::ty;
use rustc_trans::save::{self, SaveContext};

use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::iter::FromIterator;

use syntax::ast::NodeId;
use syntax::{ast, visit};

pub struct RecordVisitor<'l, 'tcx: 'l> {
    save_cx: SaveContext<'l, 'tcx>,

    static_calls: HashSet<(NodeId, NodeId)>,
    dynamic_calls: HashMap<NodeId, NodeId>,
    functions: HashMap<NodeId, String>,
    method_decls: HashMap<NodeId, String>,

    cur_fn: Option<NodeId>,
}

impl<'l, 'tcx: 'l> RecordVisitor<'l, 'tcx> {
    pub fn new(tcx: &'l ty::ctxt<'tcx>) -> RecordVisitor<'l, 'tcx> {
        RecordVisitor {
            save_cx: SaveContext::new(tcx),

            static_calls: HashSet::new(),
            dynamic_calls: HashMap::new(),
            functions: HashMap::new(),
            method_decls: HashMap::new(),

            cur_fn: None,
        }
    }

    pub fn dump(&self) {
        println!("Found fns:");
        for (k, d) in self.functions.iter() {
            println!("{}: {}", k, d);
        }

        println!("\nFound method decls:");
        for (k, d) in self.method_decls.iter() {
            println!("{}: {}", k, d);
        }

        println!("\nFound calls:");
        for &(ref from, ref to) in self.static_calls.iter() {
            let from = &self.functions[from];
            let to = &self.functions[to];
            println!("{} -> {}", from, to);
        }
    }

    // Make a graphviz dot file
    pub fn dot(&self) {
        // TODO use crate name 
        let mut file = File::create("out.dot").unwrap();
        graphviz::render(self, &mut file).unwrap();
    }

    fn record_method_call(&mut self, mrd: &save::MethodCallData) {
        if self.cur_fn.is_none() {
            println!("WARNING: call at {:?} without known current function",
                     mrd.span);
            return;
        }

        if let Some(ref_id) = mrd.ref_id {
            if ref_id.krate == ast::LOCAL_CRATE {
                self.static_calls.insert((self.cur_fn.unwrap(), ref_id.node));
            }
            return;
        }

        if let Some(decl_id) = mrd.decl_id {
            if decl_id.krate == ast::LOCAL_CRATE {
                // TODO dynamic calls
            }
        }
    }
}

impl<'v, 'l, 'tcx: 'l> visit::Visitor<'v> for RecordVisitor<'l, 'tcx> {
    fn visit_path(&mut self, path: &'v ast::Path, id: NodeId) {
        if save::generated_code(path.span) {
            return;
        }

        let data = self.save_cx.get_path_data(id, path);
        // TODO need to check for method calls too
        if let save::Data::FunctionCallData(ref fcd) = data {
            if fcd.ref_id.krate == ast::LOCAL_CRATE {
                let to = fcd.ref_id.node;
                if let Some(from) = self.cur_fn {
                    self.static_calls.insert((from, to));
                } else {
                    println!("WARNING: call at {:?} without known current function",
                             fcd.span);
                }
            }
        }
        if let save::Data::MethodCallData(ref mrd) = data {
            self.record_method_call(mrd);
        }

        visit::walk_path(self, path)
    }

    fn visit_expr(&mut self, ex: &'v ast::Expr) {
        if save::generated_code(ex.span) {
            return;
        }

        visit::walk_expr(self, ex);

        match ex.node {
            // ast::Expr_::ExprCall(_) => {

            // }
            ast::Expr_::ExprMethodCall(..) => {}
            _ => {
                return;
            }
        }

        let data = self.save_cx.get_expr_data(ex);
        if let Some(save::Data::MethodCallData(ref mrd)) = data {
            self.record_method_call(mrd);
        }
    }

    fn visit_item(&mut self, item: &'v ast::Item) {
        if save::generated_code(item.span) {
            return;
        }

        if let ast::Item_::ItemFn(..) = item.node {
            let data = self.save_cx.get_item_data(item);
            if let save::Data::FunctionData(fd) = data {
                self.functions.insert(fd.id, fd.qualname);

                let prev_fn = self.cur_fn;
                self.cur_fn = Some(fd.id);
                visit::walk_item(self, item);
                self.cur_fn = prev_fn;

                return;
            }
        }

        visit::walk_item(self, item)
    }

    fn visit_trait_item(&mut self, ti: &'v ast::TraitItem) {
        if save::generated_code(ti.span) {
            return;
        }

        match ti.node {
            ast::TraitItem_::MethodTraitItem(_, None) => {
                let fd = self.save_cx.get_method_data(ti.id, ti.ident.name, ti.span);
                self.method_decls.insert(fd.id, fd.qualname);
            }
            ast::TraitItem_::MethodTraitItem(_, Some(_)) => {
                let fd = self.save_cx.get_method_data(ti.id, ti.ident.name, ti.span);
                self.method_decls.insert(fd.id, fd.qualname.clone());
                self.functions.insert(fd.id, fd.qualname);
                
                let prev_fn = self.cur_fn;
                self.cur_fn = Some(fd.id);
                visit::walk_trait_item(self, ti);
                self.cur_fn = prev_fn;

                return;
            }
            _ => {}
        }

        visit::walk_trait_item(self, ti)
    }

    fn visit_impl_item(&mut self, ii: &'v ast::ImplItem) {
        if save::generated_code(ii.span) {
            return;
        }

        if let ast::ImplItem_::MethodImplItem(..) = ii.node {
            let fd = self.save_cx.get_method_data(ii.id, ii.ident.name, ii.span);
            self.functions.insert(fd.id, fd.qualname);

            let prev_fn = self.cur_fn;
            self.cur_fn = Some(fd.id);
            visit::walk_impl_item(self, ii);
            self.cur_fn = prev_fn;

            return;
        }

        visit::walk_impl_item(self, ii)
    }
}

pub type Edge = (NodeId, NodeId);

impl<'a, 'l, 'tcx: 'l> Labeller<'a, NodeId, Edge> for RecordVisitor<'l, 'tcx> {
    fn graph_id(&'a self) -> graphviz::Id<'a> {
        graphviz::Id::new("Callgraph_for_TODO").unwrap()
    }

    fn node_id(&'a self, n: &NodeId) -> graphviz::Id<'a> {
        graphviz::Id::new(format!("n_{}", n)).unwrap()
    }

    fn node_label(&'a self, n: &NodeId) -> graphviz::LabelText<'a> {
        graphviz::LabelText::label(&*self.functions[n])
    }
}

impl<'a, 'l, 'tcx: 'l> GraphWalk<'a, NodeId, Edge> for RecordVisitor<'l, 'tcx> {
    fn nodes(&'a self) -> graphviz::Nodes<'a, NodeId> {
        graphviz::Nodes::from_iter(self.functions.keys().cloned())
    }

    fn edges(&'a self) -> graphviz::Edges<'a, Edge> {
        graphviz::Edges::from_iter(
            self.static_calls.iter().map(|&(ref f, ref t)| (f.clone(), t.clone())))
    }

    fn source(&'a self, &(from, _): &Edge) -> NodeId {
        from
    }

    fn target(&'a self, &(_, to): &Edge) -> NodeId {
        to
    }
}
