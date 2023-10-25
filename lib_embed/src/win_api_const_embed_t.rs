#![allow(unused_imports,unused_variables,dead_code)]
/*! Get a Windows API constant type and value from an embedded database

See win_api_const_t.ahk for an example on how to use in AutoHotkey
*/
// todo: move to a shared type crate and use here and in the helper::parser
// #[derive(Debug)] pub enum WinConstVal {

// 0 Return value from a key found in a PHF hashmap
include!(concat!("../../data/","/win32const_codegen_t.rs"));	// imports static win32_const:phf::Map<&'static str,WinConstVal >
use widestring::{U16Str,WideChar,u16cstr,
  U16CString,U16CStr,	//   0 U16/U32-CString wide version of the standard CString type
  Utf16Str   ,       	// no0 UTF-16 encoded, growable owned string
};


use std     	::{self,slice,ptr};
use std::ffi	::{CString};

fn ret_error(err_msg:&U16CStr, err_sz:u32,err_ptr:*mut WideChar) -> *mut WinConstFFI { // create a buffer from pointer/size and fill it in
  let err_msg_bufer   	= unsafe{slice::from_raw_parts_mut::<WideChar>(err_ptr, err_sz as usize)};
  let err_msg_b:&[u16]	= err_msg.as_slice_with_nul(); // converts to a slice of the underlying elements, including the nul terminator.
  let max_buff_len    	= std::cmp::min(err_msg_b.len(),err_sz as usize);
  err_msg_bufer[..max_buff_len].copy_from_slice(&err_msg_b[..max_buff_len]);
  Box::into_raw(Box::new(const_null))
}

#[derive(Debug)] pub enum WinConstVal {
  Str   (&'static str),
  Array (&'static str), // [1,2,3,]
  Struct(&'static str), //_
   I8( i8), U8( u8),
  I16(i16),U16(u16),
  I32(i32),U32(u32),F32(f32),
  I64(i64),U64(u64),F64(f64),
  ISize(isize),
}
impl WinConstVal {
  pub fn get_val(&self) -> String {
    type WCVal = WinConstVal;
    match *self {
      WCVal:: I8(n)=>n.to_string(),WCVal:: U8(n)=>n.to_string(),
      WCVal::I16(n)=>n.to_string(),WCVal::U16(n)=>n.to_string(),
      WCVal::I32(n)=>n.to_string(),WCVal::U32(n)=>n.to_string(),WCVal::F32(n)=>n.to_string(),
      WCVal::I64(n)=>n.to_string(),WCVal::U64(n)=>n.to_string(),WCVal::F64(n)=>n.to_string(),
      WCVal::ISize (n)=>n.to_string(),
      WCVal::Str   (s)=>s.to_string(),
      WCVal::Array (s)=>s.to_string(),
      WCVal::Struct(s)=>s.to_string(),
    }
  }
  pub fn get_type(&self) -> String {
    type WCVal = WinConstVal;
    match *self {
      WCVal:: I8(n)=>"Char" .to_string(),WCVal:: U8(n)=>"UChar" .to_string(),
      WCVal::I16(n)=>"Short".to_string(),WCVal::U16(n)=>"UShort".to_string(),
      WCVal::I32(n)=>"Int"  .to_string(),WCVal::U32(n)=>"UInt"  .to_string(),WCVal::F32(n)=>"Float" .to_string(),
      WCVal::I64(n)=>"Int64".to_string(),WCVal::U64(n)=>"UInt64".to_string(),WCVal::F64(n)=>"Double".to_string(),
      WCVal::ISize (n)=>"Int64".to_string(), //todo check platform size???
      WCVal::Str   (s)=>"Str".to_string(),
      WCVal::Array (s)=>"Str".to_string(),
      WCVal::Struct(s)=>"Str".to_string(),
    }
  }
}

#[repr(C)] pub struct WinConstFFI {
  type_	: *const WideChar,
  value	: *const WideChar,
}
const const_null:WinConstFFI = WinConstFFI{type_:ptr::null(), value:ptr::null()};

#[no_mangle] pub extern "C"
fn get_win32_const_t(pre:&WideChar,s:&WideChar, err_sz:u32,err_ptr:*mut WideChar) -> *mut WinConstFFI { // call dealloc from AHK to avoid memory leak!
  let err_cstr 	= u16cstr!("Some null lurking inside!");
  let err_utf16	= u16cstr!("Found invalid UTF16 sequences!");

  let pre_wc	        	= unsafe {U16CStr::from_ptr_str(pre)}; // Constructs a wide C string slice from a nul-terminated string pointer // LOCALE_
  let s_wc  	        	= unsafe {U16CStr::from_ptr_str(s  )}; // ... panics if on null
  let pre_wx	:&U16Str	= pre_wc.as_ustr(); // 16b wide string slice with undefined encoding
  let s_wx  	:&U16Str	= s_wc  .as_ustr(); // NO NULL-term
  // reject invalid UTF16 (skip check with from_ustr_unchecked if certain input is valid UTF16)
  let pre_w	:&Utf16Str = match Utf16Str::from_ustr(pre_wx){Ok(s)=>s, Err(_e)=>return ret_error(err_utf16,err_sz,err_ptr)};
  let s_w  	:&Utf16Str = match Utf16Str::from_ustr(s_wx  ){Ok(s)=>s, Err(_e)=>return ret_error(err_utf16,err_sz,err_ptr)};
  // Convert to UTF8
  let pre_s	:String = pre_w.to_string(); // since it's valid UTF16, conversion is lossless and non-fallible
  let s_s  	:String = s_w  .to_string();
  // Find key
  let keys:[&str; 4] = [&(pre_s.clone()       + &s_s) // search the original 1st
   ,                    &(pre_s.clone() + "_" + &s_s) // then with a _
   ,                    &(pre_s.clone() + " " + &s_s).to_ascii_lowercase() // search lowercase (with all subs)
   ,                    &(pre_s               + &s_s).to_ascii_lowercase()];
  for k in keys {
    if let Some(val_t) = win32_const.get(k) {
      if   let Ok(val_w16cs) = U16CString::from_str(val_t.get_val()) 	{
        if let Ok(typ_w16cs) = U16CString::from_str(val_t.get_type())	{
          let wc_val = WinConstFFI{type_:typ_w16cs.into_raw(), value:val_w16cs.into_raw()};
          return Box::into_raw(Box::new(wc_val))
        } else	{return ret_error(err_cstr,err_sz,err_ptr)}
      }   else	{return ret_error(err_cstr,err_sz,err_ptr)}}
  }
  ret_error(u16cstr!("✗ Value not found!"),err_sz,err_ptr)
}

// todo: should the struct strings be deallocated as well or will Rust take care of it when deallocating the struct itself?
/** # SAFETY
  Must be called only with a pointer generated by another Rust function via `.into_raw`. The pointer can't be used after this call, and the FFI receiver of this pointer can't edit it*/
#[no_mangle] pub extern "system"
fn dealloc_lib_struct(ptr:*mut WinConstVal) {
  if ptr.is_null() {return;}
  unsafe{let _ = Box::from_raw(ptr);}
}
