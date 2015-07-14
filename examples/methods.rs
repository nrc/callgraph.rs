// Copyright 2015 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

// I think this is every possible way to call a method in Rust.

struct Foo;

impl Foo {
    fn m1() {
        println!("Hello! m1");
    }

    fn m2(&self) {
        println!("Hello! m2");
    }
}

trait Bar {
    fn m3();
    fn m4(&self);
}

impl Bar for Foo {
    fn m3() {
        println!("Hello! m3");
    }

    fn m4(&self) {
        println!("Hello! m4");
    }   
}

trait Baz {
    fn m5(&self);
}

impl Baz for Foo {
    fn m5(&self) {
        println!("Hello! m5");
    }   
}

fn foo<T: Bar>(x: T) {
    x.m4();
}

fn qux<T: Baz + ?Sized>(x: &T) {
    x.m5();
}

fn main() {
    // Inherant
    Foo::m1();
    // Inherant with receiver
    Foo.m2();
    // Static
    Foo::m3();
    // UFCS static
    <Foo as Bar>::m3();
    // Static with receiver
    Foo.m4();
    // UFCS static with receiver
    Foo::m4(&Foo);

    let x: &Baz = &Foo;
    // Dynamic
    x.m5();
    // UFCS dynamic
    Baz::m5(x);
    // UFCS static
    <Foo as Baz>::m5(&Foo);

    // Static vtable
    foo(Foo);
    // Dynamic vtable
    qux(x);
}
