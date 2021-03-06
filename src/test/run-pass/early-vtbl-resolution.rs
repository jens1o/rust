// Copyright 2012 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#![allow(non_camel_case_types)]
#![allow(dead_code)]
// pretty-expanded FIXME #23616

trait thing<A> {
    fn foo(&self) -> Option<A>;
}
impl<A> thing<A> for isize {
    fn foo(&self) -> Option<A> { None }
}
fn foo_func<A, B: thing<A>>(x: B) -> Option<A> { x.foo() }

struct A { a: isize }

pub fn main() {
    let _x: Option<f64> = foo_func(0);
}
