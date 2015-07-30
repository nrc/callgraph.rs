// Copyright 2015 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

// TODO
// tests
// sysroot


#![feature(rustc_private)]

#[macro_use]
extern crate log;

extern crate getopts;
extern crate graphviz as rustc_graphviz;
extern crate rustc;
extern crate rustc_driver;
extern crate rustc_trans;
extern crate syntax;

use rustc::session::Session;
use rustc_driver::{driver, CompilerCalls, Compilation};
use rustc_trans::back::link;

use std::collections::{HashMap, HashSet};
use std::fs::File;

use syntax::ast::NodeId;
use syntax::visit;


// Where all the work is done.
mod visitor;

// Handle graphviz output.
mod graphviz;

const SKIP_UNCONNECTED_FNS: bool = true;

// Coordinates the compiler, doesn't need any state for callgraphs.
struct CallGraphCalls;

// A bunch of callbacks from the compiler. We don't do much, mostly accept the
// default implementations.
impl<'a> CompilerCalls<'a> for CallGraphCalls {
    fn build_controller(&mut self, _: &Session) -> driver::CompileController<'a> {
        // Mostly, we want to copy what rustc does.
        let mut control = driver::CompileController::basic();
        // But we can stop after analysis, we don't need to generate code.
        control.after_analysis.stop = Compilation::Stop;
        control.after_analysis.callback = Box::new(move |state| {
            // Once we stop, then we walk the AST, collecting information
            let ast = state.expanded_crate.unwrap();
            let tcx = state.tcx.unwrap();

            let mut visitor = visitor::FnVisitor::new(tcx);

            // This actually does the walking.
            visit::walk_crate(&mut visitor, ast);

            let crate_name = link::find_crate_name(Some(&state.session),
                                                   &ast.attrs,
                                                   state.input);

            // When we're done, process the info we collected.
            let data = visitor.post_process(crate_name);

            // Then produce output.
            data.dump();
            data.dot();
        });

        control
    }
}

// args are the arguments passed on the command line, generally passed through
// to the compiler.
pub fn run(args: Vec<String>) {
    // Create a data structure to control compilation.
    let mut calls = CallGraphCalls;

    // Run the compiler!
    rustc_driver::run_compiler(&args, &mut calls);
}


// Processed data about our crate. See comments on visitor::FnVisitor for more
// detail.
struct FnData {
    static_calls: HashSet<(NodeId, NodeId)>,
    // (caller def, callee def) c.f., FnVisitor::dynamic_calls.
    dynamic_calls: HashSet<(NodeId, NodeId)>,    
    functions: HashMap<NodeId, String>,

    crate_name: String
}


impl FnData {
    // Make a graphviz dot file.
    // Must be called after post_process.
    pub fn dot(&self) {
        let mut file = File::create(&format!("{}.dot", self.crate_name)).unwrap();
        rustc_graphviz::render(self, &mut file).unwrap();
    }

    // Dump collected and processed information to stdout.
    pub fn dump(&self) {
        println!("Found fns:");
        for (k, d) in self.functions.iter() {
            println!("{}: {}", k, d);
        }

        println!("\nFound calls:");
        for &(ref from, ref to) in self.static_calls.iter() {
            let from = &self.functions[from];
            let to = &self.functions[to];
            println!("{} -> {}", from, to);
        }

        println!("\nFound potential calls:");
        for &(ref from, ref to) in self.dynamic_calls.iter() {
            let from = &self.functions[from];
            let to = &self.functions[to];
            println!("{} -> {}", from, to);
        }
    }

}
