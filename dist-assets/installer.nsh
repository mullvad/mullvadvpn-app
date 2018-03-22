!macro preInit
    ; This macro is inserted at the beginning of the NSIS .OnInit callback
    ; It is activated both at compile-time and runtime
    Messagebox MB_OK "preInit"
!macroend
 
!macro customInstall
    ; This macro is activated towards the end of the installation
	; after all files are copied, shortcuts created, etc
	Messagebox MB_OK "customInstall"
!macroend
 
 