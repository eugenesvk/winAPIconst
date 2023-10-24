use crate::*;
use crate::helper	::*;
use crate::alias 	::*;
use crate::fs    	::*;

use indexmap	::{IndexMap, IndexSet};

use std::fs   	::File;
use std::io   	::{self,prelude::*,BufRead,BufWriter};
use std::path 	::{self,Path,PathBuf};
use std::error	::{Error};

pub const ziggle_src       	:&str	= "./data/ziggle.txt";
pub const data_start_marker	:&str	= ".data.";
pub const MMAP_PATH        	:&str	= "./data/winAPI_Const.rkyv";

pub fn buff_write_kv<W: Write>(b:&mut W,key:&str,val:&str) {
  b.write(key.as_bytes()).unwrap();b.write(tab).unwrap();b.write(val.as_bytes()).unwrap();b.write(nl).unwrap();
}

use std::ffi::{OsStr,OsString};
fn concat_os_strings2(a:&OsStr, b:&OsStr) -> OsString {
  let mut ret = OsString::with_capacity(a.len() + b.len()); // allocate once
  ret.push(a); ret.push(b); // doesn't allocate
  ret
}
fn concat_oss(ss:&[&OsStr]) -> Result<OsString,Box<dyn std::error::Error>> {
  let mut len:usize = 0;
  for s in ss {
    let slen	= s.len();
    let cap 	= usize::MAX - len;
    if slen < cap {
      len += slen;
    } else {return Err(format!("∑ of passed string lengths exceeds usize ‘{}’",usize::MAX).into())}
    }
  let mut ret = OsString::with_capacity(len); // allocate once
  for s in ss { ret.push(s); } // doesn't allocate
  Ok(ret)
}

pub const tab	:&[u8]	= "\t".as_bytes();
pub const nl 	:&[u8]	= "\n".as_bytes();
pub fn get_path_clean_log(src:&str) -> (PathBuf,PathBuf) {
  let mut path_in 	= PathBuf::from(&src);
  let parent      	= path_in.parent()   .unwrap();                 	// ./data/
  let stem_in     	= path_in.file_stem().unwrap_or(OsStr::new(""));	// ziggle
  let ext_in      	= path_in.extension().unwrap_or(OsStr::new(""));	// .txt
  let mut path_out	= PathBuf::from(&parent);
    let fname_out 	= concat_oss(&[
      &stem_in.to_os_string(),
      &OsStr::new("_clean64."),
      &ext_in.to_os_string(),
      ]).unwrap();
    path_out.push(&fname_out);
  let mut log_dupe_p 	= PathBuf::from(&parent);
    let mut fname_out	= PathBuf::from(&fname_out);
    fname_out.set_extension("log");
    log_dupe_p.push(&fname_out);
  (path_out,log_dupe_p)
}

use std::collections::HashSet;
use chrono::prelude::*;
use aho_corasick::{AhoCorasick, PatternID};

pub fn parse_ziggle_vec() -> Result<Vec<(String,String)>,Box<dyn std::error::Error>> { // for rkyv ArchiveHashmap which doesn't benefit from HashMap, but uses an extension to convert Vector to ArchiveHashmap
  let mut win32_const:Vec<(String,String)>	= Vec::with_capacity(200_000 * 2);
  let mut all_keys   :HashSet<String>     	= HashSet::new(); // for checking dupes, though dupes removed during cleanup, some might appear again after replacements, especially given that some constants have the same name differing only by CaSe
    // IID_IXMLDOMImplementation
    // IID_IXmlDomImplementation

  let repl_src  = &["_"	,"ENGLISH"	,"HEADER"	,"DEFAULT"	,"CODEPAGE"	,"NUMBER"	,"NAME"	,"LANGUAGE"	,"WINDOWS"	];
  let repl_with = &[" "	,"En"     	,"Hd"    	,"Def"    	,"CPg"     	,"Num"   	,"Nm"  	,"Lng"     	,"Win"    	];
  let repl_ac = AhoCorasick::builder().ascii_case_insensitive(true).build(repl_src).unwrap();

  let mut ist:i32 = 0;
  let log_at_count = 10000 ;
  let (path_clean,_) = get_path_clean_log(&ziggle_src);

  let log_dupe_p = PathBuf::from(MMAP_PATH.to_string() + ".log");
  if log_dupe_p.is_file()	{return Err(format!("Aborting, file exists {:?}",log_dupe_p).into())};
  let file_log = File::create(&log_dupe_p).unwrap();
  let mut file_log_buff = BufWriter::new(file_log);
  file_log_buff.write("# Removed duplicate key/value pairs".as_bytes()).unwrap();
  file_log_buff.write(nl).unwrap();

  let mut is_data_start	= false;
  if let Ok(lines) = read_lines(path_clean) {
    for line_maybe in lines { // consumes iterator, returns an (Optional) String
      ist += 1;
      if (ist % log_at_count) == 0 {p!("status report: read line # {} @ {}",ist,Utc::now())}
      if let Ok(line) = line_maybe {	// WM_RENDERFORMAT 773
        if line.starts_with(data_start_marker) { is_data_start = true; p!("status report: found data marker ‘{}’ line # {} @ {}",data_start_marker,ist,Utc::now());}
        if !is_data_start { continue; }
        if let Some(val_key) = line.split_once('\t') {
          let (mut key,val)	= (val_key.0.to_string(),val_key.1.to_string()); //WM_RENDERFORMAT 773
          let mut keys     	= vec![key.clone()]; // push original WM_RENDERFORMAT
          let key_upd      	= repl_ac.replace_all(&key, repl_with).to_ascii_lowercase();
          if key_upd != key {keys.push(key_upd);} // push lowercased sub ‘wm renderformat’
          for k in keys { // cleaning script removed all dupes, but replacements here may generate new ones
            if all_keys.contains(&k) { // skip dupes and log
              buff_write_kv(&mut file_log_buff, &key, &val);
            } else {
              all_keys   .insert( k.clone());
              win32_const.push  ((k        ,val.clone()));
            }
          }
        }
      }
    }
  }
  file_log_buff.flush().unwrap();
  // assert_eq!(win32_const[0].0	,"__RPCNDR_H_VERSION__");assert_eq!(win32_const[0].1	,"500");
  Ok(win32_const)
}


pub const col_name_nm     	:&str	= "name";
pub const col_value_nm    	:&str	= "value";
pub const col_namespace_nm	:&str	= "namespace";

pub fn convert_const_csv2vec(src:&Path) -> Result<Vec<(String,String)>,Box<dyn Error>> { // for rkyv ArchiveHashmap which doesn't benefit from HashMap, but uses an extension to convert Vector to ArchiveHashmap
  let mut win32_const:Vec<(String,String)>	= Vec::with_capacity(200_000);
  let mut all_keys   :HashSet<String>     	= HashSet::new(); // for checking dupes, though dupes removed during cleanup, some might appear again after replacements, especially given that some constants have the same name differing only by CaSe
    // IID_IXMLDOMImplementation
    // IID_IXmlDomImplementation

  let repl_src  = &["_"	,"ENGLISH"	,"HEADER"	,"DEFAULT"	,"CODEPAGE"	,"NUMBER"	,"NAME"	,"LANGUAGE"	,"WINDOWS"	];
  let repl_with = &[" "	,"En"     	,"Hd"    	,"Def"    	,"CPg"     	,"Num"   	,"Nm"  	,"Lng"     	,"Win"    	];
  let repl_ac = AhoCorasick::builder().ascii_case_insensitive(true).build(repl_src).unwrap();

  let mut rdr	= csv::ReaderBuilder::new().has_headers(true).delimiter(b'\t').comment(Some(b'#')).from_path(src)?;
  let hd = rdr.headers()?.clone();
  let col_name_i      	= hd.iter().position(|x| x.to_ascii_lowercase() == col_name_nm     ).unwrap();
  let col_value_i     	= hd.iter().position(|x| x.to_ascii_lowercase() == col_value_nm    ).unwrap();
  let col_namespace_i_	= hd.iter().position(|x| x.to_ascii_lowercase() == col_namespace_nm);
  use unescaper::unescape; // required since strings are escaped

  let log_dupe_p = PathBuf::from(MMAP_PATH.to_string() + ".log");
  // if log_dupe_p.is_file()	{return Err(format!("Aborting, file exists {:?}",log_dupe_p).into())};
  let file_log = File::create(&log_dupe_p).unwrap();
  let mut file_log_buff = BufWriter::new(file_log);
  file_log_buff.write("# Removed duplicate key/value pairs".as_bytes()).unwrap();file_log_buff.write(nl).unwrap();

  let log_at_count = 10000;
  for (i, res) in rdr.records().enumerate() {
    if (i % log_at_count) == 0 {p!("status report: read line # {} @ {}",i,Utc::now())}
    let record = res?;
    let (key,val)	= (record[col_name_i ].to_string(),unescape(&record[col_value_i])?); //WM_RENDERFORMAT 773
    let mut keys 	= vec![key.clone()]; // push original WM_RENDERFORMAT
    let key_upd  	= repl_ac.replace_all(&key,repl_with).to_ascii_lowercase();
    if key_upd   	!= key {keys.push(key_upd);} // push lowercased sub ‘wm renderformat’
    for k in keys { // input data should have no dupes, but replacements here may generate new ones
      if all_keys.contains(&k) { // skip dupes and log
        buff_write_kv(&mut file_log_buff, &key, &val);
      } else {
        all_keys   .insert( k.clone());
        win32_const.push  ((k        ,val.clone()));
      }
    }
  }

  file_log_buff.flush().unwrap();
  // assert_eq!(win32_const[0].0	,"__RPCNDR_H_VERSION__");assert_eq!(win32_const[0].1	,"500");
  Ok(win32_const)
}
