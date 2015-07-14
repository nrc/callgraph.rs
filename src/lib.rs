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
// docs
// methods
// tidy up (RecordVisitor is a crap name, move graphviz stuff to its own mod)
// tests
// sysroot
// pass crate name to output


#![feature(rustc_private)]

#[macro_use]
extern crate log;

extern crate getopts;
extern crate graphviz;
extern crate rustc;
extern crate rustc_driver;
extern crate rustc_trans;
extern crate syntax;

use rustc::session::Session;
use rustc::session::config as rustc_config;
use rustc::session::config::Input;
use rustc_driver::{driver, CompilerCalls, Compilation};

use syntax::diagnostics;
use syntax::visit;

use std::path::PathBuf;

mod visitor;

struct CallGraphCalls {
    input_path: Option<PathBuf>,
}

impl<'a> CompilerCalls<'a> for CallGraphCalls {
    fn early_callback(&mut self,
                      _: &getopts::Matches,
                      _: &diagnostics::registry::Registry)
                      -> Compilation {
        Compilation::Continue
    }

    fn some_input(&mut self,
                  input: Input,
                  input_path: Option<PathBuf>)
                  -> (Input, Option<PathBuf>) {
        match input_path {
            Some(ref ip) => self.input_path = Some(ip.clone()),
            _ => {
                panic!("No input path");
            }
        }
        (input, input_path)
    }

    fn no_input(&mut self,
                _: &getopts::Matches,
                _: &rustc_config::Options,
                _: &Option<PathBuf>,
                _: &Option<PathBuf>,
                _: &diagnostics::registry::Registry)
                -> Option<(Input, Option<PathBuf>)> {
        panic!("No input supplied to RustFmt");
    }

    fn late_callback(&mut self,
                     _: &getopts::Matches,
                     _: &Session,
                     _: &Input,
                     _: &Option<PathBuf>,
                     _: &Option<PathBuf>)
                     -> Compilation {
        Compilation::Continue
    }

    fn build_controller(&mut self, _: &Session) -> driver::CompileController<'a> {
        let mut control = driver::CompileController::basic();
        control.after_analysis.stop = Compilation::Stop;
        control.after_analysis.callback = Box::new(move |state| {
            let krate = state.expanded_crate.unwrap();
            let tcx = state.tcx.unwrap();

            let mut visitor = visitor::RecordVisitor::new(tcx);
            visit::walk_crate(&mut visitor, krate);
            visitor.dump();
            visitor.dot();
        });

        control
    }
}

// args are the arguments passed on the command line, generally passed through
// to the compiler.
pub fn run(args: Vec<String>) {
    let mut call_ctxt = CallGraphCalls { input_path: None };
    rustc_driver::run_compiler(&args, &mut call_ctxt);
}
