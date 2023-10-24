#Requires AutoHotKey 2.0

; Option 1
 ; Add ‘winAPIconst_embed.dll’ library to your ‘lib/’ folder
 ; Launch this script to estimate the time it took to retrieve a few keys


; Option 2 (doesn't seem to have any benefits as the large embedded dll isn't fully loaded to memory)
 ; to use a memory-mapped database file instead of the dll with the data embedded:
 ; Add ‘winAPIconst_mmap.dll’ library to your ‘lib/’ folder
 ; Add the database to the same folder where this script is located or to a ‘data/’ subfolder
   ; winAPI_Const.rkyv_data_0
   ; winAPI_Const.rkyv_state
 ; Change ‘embed’ to ‘mmap’ in ‘winAPIconst_loader.load('embed')’ below

#include <win_api_const_lib>
get_winAPI_Const_dll()
get_winAPI_Const_dll() {
  winAPI	:= winAPIconst_loader.load('embed')
  cC    	:= winAPI.getKey_Any   .Bind(winAPI) ; get any key like '__WARNING_POSTCONDITION_NULLTERMINATION_VIOLATION'
  cCLS  	:= winAPI.getKey_CLSID .Bind(winAPI) ; get 'ActiveDesktop' without a 'CLSID_' prefix
  cIID  	:= winAPI.getKey_IID   .Bind(winAPI) ; get 'ActiveDesktop' without a 'IID_I'  prefix
  cLoc  	:= winAPI.getKey_Locale.Bind(winAPI) ; get 'ActiveDesktop' without a 'CLSID_' prefix

  DllCall("QueryPerformanceFrequency", "Int64*", &freq:=0)
  DllCall("QueryPerformanceCounter"  , "Int64*", &CounterBefore:=0)
  c_loops := 1000
  loop c_loops {
  ; 1. getting a single key
    ; res_Str := cIID('ActiveDesktop')

  ; 2. getting a couple of keys in a real function to get current Wallpaper path
    cchWallpaper	:= 260
    GetWallpaper	:= 4
    wszWallpaper	:= Buffer(cchWallpaper * 2)
  ; 2.1 Old method with manually inserted constant values
    ; static AD_GETWP_LAST_APPLIED	:= 0x00000002
    ; , CLSID_ActiveDesktop       	:= "{75048700-EF1F-11D0-9888-006097DEACF9}"
    ; , IID_IActiveDesktopP       	:= "{52502ee0-ec80-11d0-89ab-00c04fc2972d}"
    ; AD := ComObject(CLSID_ActiveDesktop, IID_IActiveDesktopP)
    ; ComCall(GetWallpaper, AD, "ptr",wszWallpaper, "uint",cchWallpaper, "uint",AD_GETWP_LAST_APPLIED)

  ; 2.2a New method with dynamic lookup of constant values (just as fast with static variables)
    static key := 'ActiveDesktop'
    , CLSID_ActiveDesktop  	:= cCLS(key)
    , IID_IActiveDesktopP  	:= cIID(key)
    , AD_GETWP_LAST_APPLIED	:= cC('AD_GETWP_LAST_APPLIED')
    AD := ComObject(CLSID_ActiveDesktop, IID_IActiveDesktopP)
    ComCall(GetWallpaper, AD, "ptr",wszWallpaper, "uint",cchWallpaper, "uint",AD_GETWP_LAST_APPLIED)

  ; 2.2b Dynamic calls are even more ergonomic to write (only ~30% slower, still tiny ~ms)
    ; AD := ComObject(cCLS(key), cIID(key))
    ; ComCall(GetWallpaper, AD, "ptr",wszWallpaper, "uint",cchWallpaper, "uint",cC("AD_GETWP_LAST_APPLIED"))


    res_Str := StrGet(wszWallpaper,"UTF-16")
  }
  DllCall("QueryPerformanceCounter", "Int64*", &CounterAfter1:=0)
  Count1 := (CounterAfter1 - CounterBefore) / freq * 1000

  msgbox('#' c_loops ' loops in ' Count1 'ms'
    '`nWallpaper: '  SubStr(res_Str,1,60)
    ,winAPI.libNm '.dll','T3')
  ; winAPIconst_loader.unload('embed') ; to conserve ~90k memory (but waste time on repeated reloading), the DLL may be unloaded after using it
}
