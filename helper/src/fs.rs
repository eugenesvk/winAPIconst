use crate::alias::*;
use crate::helper::*;

use std::fs  	::File;
use std::io  	::{self, BufRead};
use std::path	::Path;

pub fn read_lines<P>(filename:P) -> io::Result<io::Lines<io::BufReader<File>>> // → Iterator to the Reader of the lines of the file (wrapped in Result to allow matching on errors)
  where P:AsRef<Path>, {
  let file = File::open(filename)?;
  Ok(io::BufReader::new(file).lines())
}
