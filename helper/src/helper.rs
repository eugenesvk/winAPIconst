use crate::alias::*;
// pub allows use in other files
pub fn type_of      <T>(_: T) -> &'static str {          type_name::<T>() }
pub fn print_type_of<T>(_:&T)                 { p!("{}", type_name::<T>());}
