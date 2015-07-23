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
// tidy up (RecordVisitor is a crap name, pull the data up to lib.rs?)
//   split RecordVisitor?
// docs in README
// tests
// pass crate name to output
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

use syntax::visit;



// Where all the work is done.
mod visitor;

// Hanlde graphviz output.
mod graphviz;


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
            let krate = state.expanded_crate.unwrap();
            let tcx = state.tcx.unwrap();

            let mut visitor = visitor::RecordVisitor::new(tcx);

            // This actually does the walking.
            visit::walk_crate(&mut visitor, krate);

            // When we're done, process the info we collected.
            visitor.post_process();

            // Then produce output.
            visitor.dump();
            visitor.dot();
        });

        control
    }
}

// args are the arguments passed on the command line, generally passed through
// to the compiler.
pub fn run(args: Vec<String>) {
    // Create a data structure to control compilation.
    let mut call_ctxt = CallGraphCalls;

    // Run the compiler!
    rustc_driver::run_compiler(&args, &mut call_ctxt);
}
