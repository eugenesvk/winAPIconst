- (when AHK 2.1 alpha gets support for Structs with String fields) return type and value in 1 call and use the result in DllCall without having to manually lookup types
  - pass a `{type, value}` struct with via FFI to pass type/value strings
  - ?(not possible in AHK) return a Value enum that has all the WinConst data types to use "native" types instead of having to use only strings
    - how does this interplay with rkyv? (phf is fine)
- (for the rkyv database file) how to keep memory mapped file permamapped, it gets unload after every function call despite the fact that the dll is still loaded (can do once cel, would need to patch mmap-ssync crate, though not sure this is needed) 
- move replacement pairs to a config file from source code
- add manual rules for shorter more ergonomic keys