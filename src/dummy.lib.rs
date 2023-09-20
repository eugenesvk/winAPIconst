#![allow(unused_imports,unused_variables,unreachable_code,dead_code,non_upper_case_globals)]
extern crate helper;
use helper        	::*; // gets macros
use helper::alias 	::*;
use helper::helper	::*;

pub mod libmod;
use crate::libmod::ret42;

fn lib() {
  ret42();
}
