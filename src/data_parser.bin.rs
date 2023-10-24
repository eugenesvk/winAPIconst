#![allow(unused_imports,unused_variables,unreachable_code,dead_code,non_upper_case_globals)]
extern crate helper;
use helper        	::*; // gets macros
use helper::alias 	::*;
use helper::helper	::*;

pub mod win_api_const_dump;
use win_api_const_dump::{win32const_save_rkyv_mmap,ziggle_clean,win32const_check_rkyv_mmap,Win32const};

use std      	::env;
use std::fs  	::File;
use std::io  	::{BufWriter, Write};
use std::path	::Path;

pub const win32const_codegen_p	:&str	= "./data/win32const_codegen.rs";

use chrono::prelude::*;
fn codegen_win32const() { // generate win32const_codegen.rs file with hashmap to be embedded
  let     path	= Path     ::new(win32const_codegen_p);
  if path.is_file() {
    p!("skiping existing file ={:?}",&path.as_os_str());
    return
  }
  p!("writing to path={:?}",&path.as_os_str());
  let mut file	= BufWriter::new(File::create(&path).unwrap());

  // let win32_const = parser::parse_ziggle();
  let ziggle_vec	= parser::parse_ziggle_vec().unwrap();

  let mut phf_win32_const = phf_codegen::Map::new();
  let mut ist:i32 = 0;
  let log_at_count = 10000 ;
  for key_val in ziggle_vec {
    ist += 1;
    if (ist % log_at_count) == 0 {p!("parsed line # {} @ {}",ist,Utc::now())}
    let (key,val)	= (key_val.0,key_val.1); //WM_RENDERFORMAT 773
    phf_win32_const.entry(key, &format!("{:?}",val)); // "quote" manually since values are added literally
  }
  write!(&mut file, "static win32_const: phf::Map<&'static str, &'static str> = {}", phf_win32_const.build())
    .unwrap();
  write!(&mut file, ";\n").unwrap();
}


use rkyv::{
  collections::hash_map::{ArchivedHashMap, HashMapResolver},
  with::{ArchiveWith, DeserializeWith, SerializeWith},
  ser::{Serializer,serializers::{AllocSerializer}},
  bytecheck::CheckBytes,
  validation::validators::DefaultValidator,
  string::ArchivedString,
  Archive,Deserialize,Serialize,AlignedVec,Infallible,
  };
// /// Example data-structure shared between writer and reader(s)
// #[derive(Archive, Deserialize, Serialize, Debug, PartialEq)]
// #[archive_attr(derive(CheckBytes))]
// pub struct HelloWorld {
//     pub version: u32,
//     pub messages: Vec<String>,
// }

use std::collections::HashMap;
use rkyv_wrappers;
#[derive(rkyv::Archive,rkyv::Deserialize,rkyv::Serialize,Debug,PartialEq)]
// #[archive(compare(PartialEq))] // This will generate a PartialEq impl between our unarchived and archived types
#[archive_attr(derive(Debug))] // We can pass attributes through to generated types with archive_attr
#[archive(check_bytes)]
struct Test {
  pub inner: String,
  pub hm: HashMap<String,String>,
  #[with(rkyv_wrappers::as_hashmap::AsHashMap)]
  pub hash_map_vec: Vec<(String, String)>,
}
fn codegen_win32const_rkyv_example() {
  let serializer = AllocSerializer::<0>::default();
  const STR_VAL: &'static str = "I'm in an Test!";
  let mut loc_hm:HashMap<String,String> = HashMap::new();
  loc_hm.insert("a".to_string(),"1".to_string());
  let loc_hm_vec = vec!(("hm_vec_key".to_string(),"hm_vec_val".to_string()));
  let value = Test{ inner: STR_VAL.to_string(), hm: loc_hm, hash_map_vec:loc_hm_vec};
  // let bytes = rkyv::to_bytes::<_, 256>(&value).unwrap();

  // alternative
  use rkyv::ser::{Serializer, serializers::AllocSerializer};
  let mut serializer = AllocSerializer::<4096>::default();
  serializer.serialize_value(&value).unwrap();
  let buffer = serializer.into_serializer().into_inner();

  let output = unsafe { rkyv::archived_root::<Test>(&buffer)};
  assert_eq!(output.hm.get("a").unwrap(), &"1");
  if output.hm.contains_key("a") {
    assert_eq!(output.hm.get("a").unwrap(), &"1");
  }
  let deserialized: Test = output.deserialize(&mut Infallible).unwrap();
  assert_eq!(deserialized, value);

  // let archived = rkyv::check_archived_root::<Test>(&bytes[..]); // derive #[archive(check_bytes)]
  // let archived = unsafe { rkyv::archived_root::<Test>(&bytes[..]) };
  // assert_eq!(archived, &value);

  // let deserialized: Test = archived.deserialize(&mut rkyv::Infallible).unwrap();
  // assert_eq!(deserialized, value);

  // It works!
  // serializer.serialize_value(&value).expect("failed to archive test");
  // let buf = serializer.into_inner();
  // let archived = unsafe { archived_root::<OwnedStr>(buf.as_ref()) };
  // Let's make sure our data got written correctly
  // assert_eq!(archived.as_str(), STR_VAL);
  p!("sdfd")
}



/*
use redb::{Database,Error,ReadableTable,TableDefinition};
const redb_path: &str = "data/win32const.redb";
const TABLE: TableDefinition<&str, &str> = TableDefinition::new("win32const");
fn codegen_win32const_db() -> Result<(), Error> {
  let db = Database::create(redb_path)?;
  let write_txn = db.begin_write()?;
  {
    let mut table = write_txn.open_table(TABLE)?;
    // table.insert("my_key", &123)?;
    let testval = "val";
    table.insert("my_str", testval)?;
  }
  write_txn.commit()?;

  let read_txn = db.begin_read()?;
  let table = read_txn.open_table(TABLE)?;
  // assert_eq!(table.get("my_key")?.unwrap().value(), 123);
  assert_eq!(table.get("my_str")?.unwrap().value(), "val");
  Ok(())
}
*/
// todo: add windows sys and get constants from there
fn codegen_win32const_test() {
  // use windows::Win32::Globalization::*;
  // p!("LOCALE_SENGCURRNAME = {:?}",LOCALE_SENGCURRNAME)
}
#[derive(Debug)] pub enum ConstFrom {Ziggle,WinMD,}

fn main() {
  let c_win_md:&Path	= Path::new("./data/winConst_bindgen_All_185k_dedupe");
  let args:Vec<String> = std::env::args().skip(1).collect();
  if        let Some(pos) = args.iter().position(|x| *x == "ziggle_clean") {
    ziggle_clean(); // generate a cleaned up key/value (from value/key) database
  } else if let Some(pos) = args.iter().position(|x| *x == "gen_winmd") {
    codegen_win32const(ConstFrom::WinMD,&c_win_md); // 1 WinMD → win32const_codegen.rs (embed)
  } else if let Some(pos) = args.iter().position(|x| *x == "rkyv_save_winmd") {
    win32const_save_rkyv_mmap(ConstFrom::WinMD,&c_win_md); // 2 WinMD → rkyv file (mmap)
  } else if let Some(pos) = args.iter().position(|x| *x == "rkyv_save_ziggle") {
    let db_p:&Path	= Path::new("");
    win32const_save_rkyv_mmap(ConstFrom::Ziggle,&db_p);
  } else if let Some(pos) = args.iter().position(|x| *x == "rkyv_check") {
    win32const_check_rkyv_mmap();
  } else if let Some(pos) = args.iter().position(|x| *x == "ziggle_parse") {
    // parser::parse_ziggle();
    // key2bit  for (key, value) in win32_const {
      // phf_win32_const.entry(key, &format!("{:?}",value));
    // }
    // let test_key = win32_const["__RPCNDR_H_VERSION__"];
    // let test_key = "__RPCNDR_H_VERSION__";
    // println!("Hello, world! libret42{} libadd{} libadd_ext{} libret42_ext{} test_key{}"
     // ,                      libret42  ,libadd   ,libadd_ext  ,libret42_ext ,test_key);
  } else if let Some(pos) = args.iter().position(|x| *x == "gen_ziggle") {
    let db_p:&Path	= Path::new("");
    codegen_win32const(ConstFrom::Ziggle,&db_p); // generates win32const_codegen.rs
  } else if let Some(pos) = args.iter().position(|x| *x == "gen_test") {
    codegen_win32const_test();
  } else if let Some(pos) = args.iter().position(|x| *x == "gen_db") {
    // match codegen_win32const_db() {
      // Ok(())	=> p!("redb crated successfully at {:?}",redb_path),
      // Error 	=> p!("redb FAILED to create db at {:?} due to {:?}",redb_path,Error),
    // }
  }

}
