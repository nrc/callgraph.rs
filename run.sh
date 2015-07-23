# TODO would be nice not to require sysroot
# target/debug/callgraph examples/foo.rs --sysroot /usr/local
target/debug/callgraph examples/methods.rs --sysroot /usr/local
dot -oout.png -Tpng <methods.dot
