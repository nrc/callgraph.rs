# TODO would be nice not to require sysroot
# target/debug/callgraph examples/foo.rs --sysroot /usr/local
#target/debug/callgraph examples/methods.rs --sysroot /usr/local
#target/debug/callgraph examples/methods.rs --sysroot /home/ncameron/rust3/x86_64-unknown-linux-gnu/stage2 -Ztreat-err-as-bug
#dot -oout.png -Tpng <methods.dot
target/debug/callgraph src/lib.rs --sysroot /home/ncameron/rust3/x86_64-unknown-linux-gnu/stage2 --crate-type lib --crate-name callgraph
#dot -ocallgraph.png -Tpng <callgraph.dot
