// Copyright 2015 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use rustc::middle::ty;
use rustc::middle::def_id::{DefId, LOCAL_CRATE};
use rustc_trans::save::{self, SaveContext};

use std::collections::{HashMap, HashSet};

use syntax::ast::NodeId;
use syntax::{ast, visit};

use FnData;


// Records functions and function calls.
pub struct FnVisitor<'l, 'tcx: 'l> {
    // Used by the save-analysis API.
    save_cx: SaveContext<'l, 'tcx>,

    // Track statically dispatched function calls.
    static_calls: HashSet<(NodeId, NodeId)>,
    // (caller def, callee decl).
    dynamic_calls: HashSet<(NodeId, NodeId)>,
    // Track function definitions.
    functions: HashMap<NodeId, String>,
    // Track method declarations.
    method_decls: HashMap<NodeId, String>,
    // Maps a method decl to its implementing methods.
    method_impls: HashMap<NodeId, Vec<NodeId>>,

    // Which function we're calling from, we'll update this as we walk the AST.
    cur_fn: Option<NodeId>,
}

// `this.cur_fn.is_some()` or returns.
macro_rules! ensure_cur_fn {($this: expr, $span: expr) => {
    if $this.cur_fn.is_none() {
        println!("WARNING: call at {:?} without known current function",
                 $span);
        return;
    }
}}

// Backup self.cur_fn, set cur_fn to id, continue to walk the AST by executing
// $walk, then restore self.cur_fn.
macro_rules! push_walk_pop {($this: expr, $id: expr, $walk: expr) => {{
    let prev_fn = $this.cur_fn;
    $this.cur_fn = Some($id);
    $walk;
    $this.cur_fn = prev_fn;
}}}

// Return if we're in generated code.
macro_rules! skip_generated_code {($span: expr) => {
    if save::generated_code($span) {
        return;
    }
}}

// True if the def_id refers to an item in the current crate.
fn is_local(id: DefId) -> bool {
    id.krate == LOCAL_CRATE
}


impl<'l, 'tcx: 'l> FnVisitor<'l, 'tcx> {
    pub fn new(tcx: &'l ty::ctxt<'tcx>) -> FnVisitor<'l, 'tcx> {
        FnVisitor {
            save_cx: SaveContext::new(tcx),

            static_calls: HashSet::new(),
            dynamic_calls: HashSet::new(),
            functions: HashMap::new(),
            method_decls: HashMap::new(),
            method_impls: HashMap::new(),

            cur_fn: None,
        }
    }

    // Processes dynamically dispatched method calls. Converts calls to the decl
    // to a call to every method implementing the decl.
    pub fn post_process(self, crate_name: String) -> FnData {
        let mut processed_calls = HashSet::new();
        let mut processed_fns = HashMap::with_capacity(self.functions.len());

        for &(ref from, ref to) in self.dynamic_calls.iter() {
            for to in self.method_impls[to].iter() {
                processed_calls.insert((*from, *to));
                self.append_fn(&mut processed_fns, *from);
                self.append_fn(&mut processed_fns, *to);
            }
        }

        if ::SKIP_UNCONNECTED_FNS {
            for &(ref from, ref to) in self.static_calls.iter() {
                self.append_fn(&mut processed_fns, *from);
                self.append_fn(&mut processed_fns, *to);
            }
        }

        FnData {
            static_calls: self.static_calls,
            dynamic_calls: processed_calls,
            functions: if ::SKIP_UNCONNECTED_FNS {
                    processed_fns
                } else {
                    self.functions
                },
            crate_name: crate_name,
        }
    }

    // If we are skipping unconnected functions, then keep track of which
    // functions are connected.
    fn append_fn(&self, map: &mut HashMap<NodeId, String>, id: NodeId) {
        if !::SKIP_UNCONNECTED_FNS {
            return;
        }

        if map.contains_key(&id) {
            return;
        }

        map.insert(id, self.functions[&id].clone());
    }

    // Helper function. Record a method call.
    fn record_method_call(&mut self, mrd: &save::MethodCallData) {
        ensure_cur_fn!(self, mrd.span);

        if let Some(ref_id) = mrd.ref_id {
            if is_local(ref_id) {
                self.static_calls.insert((self.cur_fn.unwrap(), ref_id.node));
            }
            return;
        }

        if let Some(decl_id) = mrd.decl_id {
            if is_local(decl_id) {
                self.dynamic_calls.insert((self.cur_fn.unwrap(), decl_id.node));
            }
        }
    }

    // Record that def implements decl.
    fn append_method_impl(&mut self, decl: NodeId, def: NodeId) {
        if !self.method_impls.contains_key(&decl) {
            self.method_impls.insert(decl, vec![]);
        }

        self.method_impls.get_mut(&decl).unwrap().push(def);
    }
}


// A visitor pattern implementation for visiting nodes in the AST. We only
// implement the methods for the nodes we are interested in visiting. Here,
// functions and methods, and references to functions and methods.
//
// Note that a function call (which applies to UFCS methods), `foo()` is just
// an expression involving `foo`, which can be anything with function type.
// E.g., `let x = foo; x();` is legal if `foo` is a function. Since in this
// case we would be interested in `foo`, but not `x`, we don't actually track
// call expressions, but rather path expressions which refer to functions. This
// will give us some false positives (e.g., if a function has `let x = foo;`,
// but `x` is never used).
impl<'v, 'l, 'tcx: 'l> visit::Visitor<'v> for FnVisitor<'l, 'tcx> {
    // Visit a path - the path could point to a function or method.
    fn visit_path(&mut self, path: &'v ast::Path, id: NodeId) {
        skip_generated_code!(path.span);

        let data = self.save_cx.get_path_data(id, path);
        if let Some(save::Data::FunctionCallData(ref fcd)) = data {
            if is_local(fcd.ref_id) {
                let to = fcd.ref_id.node;
                ensure_cur_fn!(self, fcd.span);
                self.static_calls.insert((self.cur_fn.unwrap(), to));
            }
        }
        if let Some(save::Data::MethodCallData(ref mrd)) = data {
            self.record_method_call(mrd);
        }

        // Continue walking the AST.
        visit::walk_path(self, path)
    }

    // Visit an expression
    fn visit_expr(&mut self, ex: &'v ast::Expr) {
        skip_generated_code!(ex.span);

        visit::walk_expr(self, ex);

        // Skip everything except method calls. (We shouldn't have to do this, but
        // calling get_expr_data on an expression it doesn't know about will panic).
        if let ast::Expr_::ExprMethodCall(..) = ex.node {} else {
            return;
        }

        let data = self.save_cx.get_expr_data(ex);
        if let Some(save::Data::MethodCallData(ref mrd)) = data {
            self.record_method_call(mrd);
        }
    }

    fn visit_item(&mut self, item: &'v ast::Item) {
        skip_generated_code!(item.span);

        if let ast::Item_::ItemFn(..) = item.node {
            let data = self.save_cx.get_item_data(item);
            if let save::Data::FunctionData(fd) = data {
                self.functions.insert(fd.id, fd.qualname);

                push_walk_pop!(self, fd.id, visit::walk_item(self, item));

                return;
            }
        }

        visit::walk_item(self, item)
    }

    fn visit_trait_item(&mut self, ti: &'v ast::TraitItem) {
        skip_generated_code!(ti.span);

        // Note to self: it is kinda sucky we have to examine the AST before
        // asking for data here.
        match ti.node {
            // A method declaration.
            ast::TraitItem_::MethodTraitItem(_, None) => {
                let fd = self.save_cx.get_method_data(ti.id, ti.ident.name, ti.span);
                self.method_decls.insert(fd.id, fd.qualname);
                self.method_impls.insert(fd.id, vec![]);
            }
            // A default method. This declares a trait method and provides an
            // implementation.
            ast::TraitItem_::MethodTraitItem(_, Some(_)) => {
                let fd = self.save_cx.get_method_data(ti.id, ti.ident.name, ti.span);
                // Record, a declaration, a definintion, and a reflexive implementation.
                self.method_decls.insert(fd.id, fd.qualname.clone());
                self.functions.insert(fd.id, fd.qualname);
                self.append_method_impl(fd.id, fd.id);
                
                push_walk_pop!(self, fd.id, visit::walk_trait_item(self, ti));

                return;
            }
            _ => {}
        }

        visit::walk_trait_item(self, ti)
    }

    fn visit_impl_item(&mut self, ii: &'v ast::ImplItem) {
        skip_generated_code!(ii.span);

        if let ast::ImplItem_::MethodImplItem(..) = ii.node {
            let fd = self.save_cx.get_method_data(ii.id, ii.ident.name, ii.span);
            // Record the method's existence.
            self.functions.insert(fd.id, fd.qualname);
            if let Some(decl) = fd.declaration {
                if is_local(decl) {
                    // If we're implementing a method in the local crate, record
                    // the implementation of the decl.
                    self.append_method_impl(decl.node, fd.id);
                }
            }

            push_walk_pop!(self, fd.id, visit::walk_impl_item(self, ii));

            return;
        }

        visit::walk_impl_item(self, ii)
    }
}
