# TODO would be nice not to require sysroot
target/debug/callgraph examples/foo.rs --sysroot /usr/local
dot -oout.png -Tpng <out.dot
