# Callgraph

Computes the callgraph of Rust programs.

## Usage

```
callgraph foo.rs --sysroot /usr/local
```

To build foo.rs, where you would usually use `rustc foo.rs`. You can also use
any arguments you would usually use with rustc. It is unfortunate that you must
specify your sysroot.

This will generate a dot file which is graphviz output, you can then convert
that it an image or pdf or whatever. For example, to create a png image called
out.png, use `dot -oout.png -Tpng <foo.dot`.


## Architecture

Uses rustc's driver APIs to run rustc up to the end of the analysis stage. We
then walk the expanded AST and query the save-analysis API for every function or
function call. We do a little post-processing of this (to map method decls to
their implementations), and then output a dot file for graphviz using
librustc_graphviz.
