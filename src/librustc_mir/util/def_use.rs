// Copyright 2016 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Def-use analysis.

use rustc::mir::{Local, Location, Mir};
use rustc::mir::visit::{PlaceContext, MutVisitor, Visitor};
use rustc_data_structures::indexed_vec::IndexVec;
use std::marker::PhantomData;
use std::mem;
use std::slice;
use std::iter;

pub struct DefUseAnalysis {
    info: IndexVec<Local, Info>,
}

#[derive(Clone)]
pub struct Info {
    pub defs_and_uses: Vec<Use>,
}

#[derive(Clone)]
pub struct Use {
    pub context: PlaceContext,
    pub location: Location,
}

impl DefUseAnalysis {
    pub fn new(mir: &Mir) -> DefUseAnalysis {
        DefUseAnalysis {
            info: IndexVec::from_elem_n(Info::new(), mir.local_decls.len()),
        }
    }

    pub fn analyze(&mut self, mir: &Mir) {
        self.clear();

        let mut finder = DefUseFinder {
            info: mem::replace(&mut self.info, IndexVec::new()),
        };
        finder.visit_mir(mir);
        self.info = finder.info
    }

    fn clear(&mut self) {
        for info in &mut self.info {
            info.clear();
        }
    }

    pub fn local_info(&self, local: Local) -> &Info {
        &self.info[local]
    }

    fn mutate_defs_and_uses<F>(&self, local: Local, mir: &mut Mir, mut callback: F)
                               where F: for<'a> FnMut(&'a mut Local,
                                                      PlaceContext,
                                                      Location) {
        for place_use in &self.info[local].defs_and_uses {
            MutateUseVisitor::new(local,
                                  &mut callback,
                                  mir).visit_location(mir, place_use.location)
        }
    }

    /// FIXME(pcwalton): This should update the def-use chains.
    pub fn replace_all_defs_and_uses_with(&self,
                                          local: Local,
                                          mir: &mut Mir,
                                          new_local: Local) {
        self.mutate_defs_and_uses(local, mir, |local, _, _| *local = new_local)
    }
}

struct DefUseFinder {
    info: IndexVec<Local, Info>,
}

impl<'tcx> Visitor<'tcx> for DefUseFinder{
    fn visit_local(&mut self,
                   &local: &Local,
                   context: PlaceContext,
                   location: Location) {
        self.info[local].defs_and_uses.push(Use {
            context,
            location,
        });
    }
}

impl Info {
    fn new() -> Info {
        Info {
            defs_and_uses: vec![],
        }
    }

    fn clear(&mut self) {
        self.defs_and_uses.clear();
    }

    pub fn def_count(&self) -> usize {
        self.defs_and_uses.iter().filter(|place_use| place_use.context.is_mutating_use()).count()
    }

    pub fn def_count_not_including_drop(&self) -> usize {
        self.defs_not_including_drop().count()
    }

    pub fn defs_not_including_drop(
        &self,
    ) -> iter::Filter<slice::Iter<Use>, fn(&&Use) -> bool> {
        self.defs_and_uses.iter().filter(|place_use| {
            place_use.context.is_mutating_use() && !place_use.context.is_drop()
        })
    }

    pub fn use_count(&self) -> usize {
        self.defs_and_uses.iter().filter(|place_use| {
            place_use.context.is_nonmutating_use()
        }).count()
    }
}

struct MutateUseVisitor<'tcx, F> {
    query: Local,
    callback: F,
    phantom: PhantomData<&'tcx ()>,
}

impl<'tcx, F> MutateUseVisitor<'tcx, F> {
    fn new(query: Local, callback: F, _: &Mir<'tcx>)
           -> MutateUseVisitor<'tcx, F>
           where F: for<'a> FnMut(&'a mut Local, PlaceContext, Location) {
        MutateUseVisitor {
            query,
            callback,
            phantom: PhantomData,
        }
    }
}

impl<'tcx, F> MutVisitor<'tcx> for MutateUseVisitor<'tcx, F>
              where F: for<'a> FnMut(&'a mut Local, PlaceContext, Location) {
    fn visit_local(&mut self,
                    local: &mut Local,
                    context: PlaceContext,
                    location: Location) {
        if *local == self.query {
            (self.callback)(local, context, location)
        }
    }
}
