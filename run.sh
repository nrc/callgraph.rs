# TODO would be nice not to require sysroot
# target/debug/callgraph examples/foo.rs --sysroot /usr/local
#target/debug/callgraph examples/methods.rs --sysroot /usr/local
#target/debug/callgraph examples/methods.rs --sysroot /home/ncameron/rust3/x86_64-unknown-linux-gnu/stage2 -Ztreat-err-as-bug
#dot -oout.png -Tpng <methods.dot
#target/debug/callgraph src/lib.rs --sysroot /home/ncameron/rust3/x86_64-unknown-linux-gnu/stage2 --crate-type lib --crate-name callgraph
#dot -ocallgraph.png -Tpng <callgraph.dot

target/debug/callgraph ~/regex/src/lib.rs --sysroot /home/ncameron/rust3/x86_64-unknown-linux-gnu/stage2 --crate-name regex --crate-type lib -L dependency=/home/ncameron/regex/target/debug -L dependency=/home/ncameron/regex/target/debug/deps --extern regex_syntax=/home/ncameron/regex/target/debug/deps/libregex_syntax-b485b7f4be54a17b.rlib --extern memchr=/home/ncameron/regex/target/debug/deps/libmemchr-38e2ee286f7e4bdb.rlib --extern aho_corasick=/home/ncameron/regex/target/debug/deps/libaho_corasick-1c0816113fe68ddc.rlib
dot -oregex.png -Tpng <regex.dot
