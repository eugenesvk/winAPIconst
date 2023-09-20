#![allow(unused_imports,unused_variables,unreachable_code,dead_code,non_upper_case_globals,unused_mut)]
extern crate helper;
use helper        	::*; // gets macros
use helper::alias 	::*;
use helper::helper	::*;
use helper::parser	::*;
use helper::fs    	::*;

use rkyv::{
  collections::hash_map::{ArchivedHashMap, HashMapResolver},
  with::{ArchiveWith, DeserializeWith, SerializeWith},
  ser::{Serializer,serializers::{AllocSerializer}},
  bytecheck::CheckBytes,
  validation::validators::DefaultValidator,
  string::ArchivedString,
  Archive,Deserialize,Serialize,AlignedVec,Infallible,
  };


#[derive(rkyv::Archive,rkyv::Deserialize,rkyv::Serialize,Debug,PartialEq)]
// #[archive(compare(PartialEq))] // This will generate a PartialEq impl between our unarchived and archived types
#[archive_attr(derive(Debug))] // We can pass attributes through to generated types with archive_attr
#[archive(check_bytes)]
pub struct Win32const { #[with(rkyv_wrappers::as_hashmap::AsHashMap)] pub hash_map_vec:Vec<(String,String)> }

use aho_corasick::{AhoCorasick, PatternID};

use std::path::{self,Path,PathBuf};
use std::ffi::{OsStr,OsString};

use std::io::prelude::*;
use std::fs	::File;
use std::io	::{self,BufRead,BufWriter};
use std::collections::HashSet;

const chr_id_rep 	:&str	= r"(?x:\(Chr\((?<chID>\d+)\)\))";
const ptrsize_rep	:&str	= r"(?x:\(
  A_PtrSize\s*=\s*8
  \s*\?\s*  (?<x64>[^:\s]+)
  \s* :\s*  (?<x32>[^)  ]+)
  \))";
const parenhex_rep	:&str	= r#"(?x:\(
  (?<hex>\"0[xX][0-9a-fA-F]+\")
  \))"#; // ("0x7FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF") INT128_MAX
use fancy_regex::Regex;
fn get_wrapped_val(val:&str) -> Result<String,Box<dyn std::error::Error>> {
  // return actual symbol from char :  (Chr(124)) CRED_TARGETNAME_DOMAIN_EXTENDED_USERNAME_SEPARATOR_A
  let re          	= Regex::new(chr_id_rep).unwrap();
  let result      	= re.captures(val); // on Chr(124)
  let captures    	= result.expect("Error running regex");
  match           	captures {
    Some(groups)  	=> {match groups.name("chID") { //124
      Some(gmatch)	=> {
        let fchar 	= char::from_u32(gmatch.as_str().parse::<u32>() // str 124 → int 124 → char(124) → |
          .unwrap()).unwrap().to_string();
        if fchar != "\t" {return Ok(fchar)}},
      None	=> (),};},
    None  	=> (),
  };

  // return first (x64,ptr=8) value: (A_PtrSize=8 ? "0xFFFFFFFFFFFFFFFF" : 4294967295) DWORD_PTR_MAX
  let re          	= Regex::new(ptrsize_rep).unwrap();
  let result      	= re.captures(val); // on (A_PtrSize=8 ? "0xFFFFFFFFFFFFFFFF" : 4294967295)
  let captures    	= result.expect("Error running regex");
  match           	captures {
    Some(groups)  	=> {match groups.name("x64") { // "0xFFFFFFFFFFFFFFFF"
      Some(gmatch)	=> return Ok(gmatch.as_str().to_string()),
      None        	=> (),};},
    None          	=> (),
  };

  // remove () from: ("0x7FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF") INT128_MAX
  let re          	= Regex::new(parenhex_rep).unwrap();
  let result      	= re.captures(val); // on ("0x7FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF")
  let captures    	= result.expect("Error running regex");
  match           	captures {
    Some(groups)  	=> {match groups.name("hex") { // "0x7FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF"
      Some(gmatch)	=> return Ok(gmatch.as_str().to_string()),
      None        	=> (),};},
    None          	=> (),
  };
  Err("Found no matches".into())
}
#[cfg(test)] pub(crate) fn test_get_wrapped_val() {
  let x = match get_wrapped_val("(Chr(124))") {
    Ok (s)	=> s,
    Err(e)	=> e.to_string(),};
  assert_eq!(x, "|");
  let y = match get_wrapped_val(r#"(A_PtrSize=8 ? "0xFFFFFFFFFFFFFFFF" : 4294967295)"#) {
    Ok (s)	=> s,
    Err(e)	=> e.to_string(),};
  assert_eq!(y, r#""0xFFFFFFFFFFFFFFFF""#);
  let y = match get_wrapped_val(r#"("0x7FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF")"#) {
    Ok (s)	=> s,
    Err(e)	=> e.to_string(),};
  assert_eq!(y, r#""0x7FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF""#);
}

pub fn ziggle_clean() {
  let (path_out,path_out_log) = get_path_clean_log(&ziggle_src);
  // p!("path_in {:?}\npath_out {:?}\npath_out_log {:?}",ziggle_src,path_out,path_out_log);

  // Clean and write data to file, excluding dupes to a log file
  let mut all_keys   :HashSet<String>	= HashSet::new(); // store all keys to check for dupes
  if path_out    .is_file()          	{p!("aborting, file exists {:?}", path_out    ); return}
  if path_out_log.is_file()          	{p!("aborting, file exists {:?}", path_out_log); return}
  let file_out = File::create(&path_out    ).unwrap();
  let file_log = File::create(&path_out_log).unwrap();

  let mut file_out_buff = BufWriter::new(file_out);
  let mut file_log_buff = BufWriter::new(file_log);
  file_log_buff.write("Removed duplicate key/value pairs".as_bytes()).unwrap();
  file_log_buff.write(nl).unwrap();
  // for i in 0..10 {file_out_buff.write("\t".as_bytes()).unwrap();}

  if let Ok(lines) = read_lines(ziggle_src) {
    for line in lines { // consumes iterator, returns an (Optional) String
      if let Ok(val_tab_key) = line {	// 773	WM_RENDERFORMAT
        if let Some(val_key) = val_tab_key.split_once('\t') {
          let (key,mut val)	= (val_key.1.to_string(),val_key.0.to_string()); //WM_RENDERFORMAT 773
          match get_wrapped_val(&val) {
            Ok (s)	=> {val = s},
            Err(e)	=> (),};
          if all_keys.contains(&key) { // skip dupes and log
            buff_write_kv(&mut file_log_buff, &key, &val);
          } else {
            all_keys.insert(key.clone());
            buff_write_kv(&mut file_out_buff, &key, &val);
          }
        }
      }
    }
  }
  file_out_buff.flush().unwrap();file_log_buff.flush().unwrap();
}

use mmap_sync::synchronizer::Synchronizer;
use std::time::Duration;
pub fn win32const_save_rkyv_mmap() { // Write
  let ziggle_vec	= parse_ziggle_vec().unwrap();
  let data      	= Win32const{hash_map_vec:ziggle_vec};
  p!("starting writing data to an mmaped file...");
  let mut synchronizer = Synchronizer::new(MMAP_PATH); // Initialize the Synchronizer
  let (written, reset) = synchronizer.write(&data, Duration::from_secs(1)).expect("failed to write data");
  p!("written: {} bytes | reset: {}", written, reset); // Show how many bytes written and whether state was reset
}

pub fn win32const_check_rkyv_mmap() { // Read & verify
  let repl_src  = &["LOCALE_"	,"ENGLISH"	,"_"	,"HEADER"	,"DEFAULT"	,"CODEPAGE"	,"NUMBER"	,"NAME"	,"LANGUAGE"	,"WINDOWS"	];
  let repl_with = &[""       	,"En"     	," "	,"Hd"    	,"Def"    	,"CPg"     	,"Num"   	,"Nm"  	,"Lng"     	,"Win"    	];
  let repl_ac = AhoCorasick::new(repl_src).unwrap();
  let mut synchronizer = Synchronizer::new(MMAP_PATH); // Initialize the Synchronizer
  let data = unsafe { synchronizer.read::<Win32const>() }.expect("failed to read data"); // Read data from shared memory
  for k in vec!["IID_IEventTrigger","LOCALE_SENGCOUNTRY","LOCALE_SENGLISHCOUNTRYNAME",
    "SEnCOUNTRYNm",
    "DISPID_SCROLLBARS",
    "DISPID_ScrollBars",
    "dispid scrollbars",
    ] {
    match data.hash_map_vec.get(k) {  // Access fields of the struct
      Some(val)	=> p!("{}={}"      , k,val),
      None     	=> {
        let key_upd = repl_ac.replace_all("LOCALE_SENGLISHCOUNTRYNAME", repl_with);
        p!("!!!missing {}, {}", k, key_upd);
      },
    }
  };
}
