<p align="center">
Fast lookup of Windows API constants by name
<br>
by calling a DLL function
</p>

<p align="center">  
AutoHotkey example included
</p>


## Introduction

The DLL includes ~170k Windows API constants and a function to look them up by name so instead of 

  - `CLSID_ActiveDesktop := "{75048700-EF1F-11D0-9888-006097DEACF9}"` you can use
  - `CLSID_ActiveDesktop	:= cCLS('ActiveDesktop')`

## Install

Copy `winAPIconst_embed.dll` from the [release page](https://github.com/eugenesvk/winAPIconst/releases) and [win_api_const_lib.ahk](./win_api_const_lib.ahk) libraries to your `lib/` subfolder of your main AutoHotkey script folder

<details>
  <summary>Alternative memory-mapped library</summary>
  The second library uses a memory-mapped file to lookup keys from instead of embedding the whole database in the DLL, though this doesn't seem to have any benefits as the large embedded DLL isn't fully loaded to memory
  
  - Add `winAPIconst_embed.dll` from the [release page](https://github.com/eugenesvk/winAPIconst/releases) to your `lib/` subfolder of your main AutoHotkey script folder
  - Add the extracted database [winAPI_Const_rkyv](https://github.com/eugenesvk/winAPIconst/blob/data/data/winAPI_Const_rkyv.zip) to the `data/` subfolder or the same folder as your main script
  - Change `embed` to `mmap` in `winAPIconst_loader.load('embed')` (and `unload`)
</details>

## Use

- Copy a [win_api_const.ahk](./win_api_const.ahk) example to your main AutoHotkey script folder and run it to show an estimate of the time it took to retrieve a few keys to get the current wallpaper's path

- Or add to your script something like:
```ahk
#include <win_api_const_lib> ; make the library class available for use
get_winAPI_Const_dll()
get_winAPI_Const_dll() {
  winAPI	:= winAPIconst_loader.load('embed') ; load the DLL with the embedded database
  cCLS  	:= winAPI.getKey_CLSID.Bind(winAPI) ; bind a 'CLSID_' auto-prefixing function
  msgbox("The value of ‘ActiveDesktop’ CLSID is "
    cCLS('ActiveDesktop'),"T3") ; lookup 'ActiveDesktop' key without the 'CLSID_' prefix
  ; winAPIconst_loader.unload('embed') ; to conserve ~0.1m memory (but waste time on repeated reloading), the DLL may be unloaded after using it
}
```

Currently there are 4 available functions:
```ahk
  cC  	:= winAPI.getKey_Any   .Bind(winAPI) ; get any key like 'LOCALE_SENGLANGUAGE'
  cLoc	:= winAPI.getKey_Locale.Bind(winAPI) ; get 'SENGLANGUAGE'  without a  'LOCALE_' prefix
  cCLS	:= winAPI.getKey_CLSID .Bind(winAPI) ; get 'ActiveDesktop' without a  'CLSID_' prefix
  cIID	:= winAPI.getKey_IID   .Bind(winAPI) ; get 'ActiveDesktop' without an 'IID_I'  prefix
```

To avoid typing other prefixes you can add methods to `winAPIconst` class in [win_api_const_lib.ahk](./win_api_const_lib.ahk) by duplicating and changing a name/DllCall arguments:

  - `,this.Locale	:= DllCall.Bind(this.lib𝑓, 'Str','LOCALE', 'Str',unset, 'UInt',this.ℯsz,'Ptr',unset, 'Ptr')` and
  - ```ahk
    getKey_Locale(key) {
      𝑓:=this.Locale
      return this.getKey(𝑓,key)
    }
    ```

Keys can either be looked up by their full CaSe-sensitive name or by their 1) case-sensitive 2) abbreviated 3) space-separated name that was derived using this simple substitution table:

|From      	|To    	|
|:-        	|:-    	|
|`_`       	| ` `  	|
|`ENGLISH` 	| `En` 	|
|`HEADER`  	| `Hd` 	|
|`DEFAULT` 	| `Def`	|
|`CODEPAGE`	| `CPg`	|
|`NUMBER`  	| `Num`	|
|`NAME`    	| `Nm` 	|
|`LANGUAGE`	| `Lng`	|
|`WINDOWS` 	| `Win`	|

So you could either lookup

-  `cLoc('SENGLANGUAGE')` or
-  `cLoc('sEngLng')` (the lookup function converts this to a case-insensitive version automatically)

(`cLoc('sEngLanguage')` is invalid to not increase the database size even more)

## Known issues

Case-sensitive lookup is not 100% reliable since some Windows constants unfortunately have identical names differing only by CaSe, so they'd have the same lower-case names

## Credits

[SKAN](https://www.autohotkey.com/boards/memberlist.php?mode=viewprofile&u=54) for compiling the [database](https://www.autohotkey.com/boards/viewtopic.php?f=83&t=99581)
