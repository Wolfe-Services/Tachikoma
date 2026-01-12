; build/installer.nsh

!macro customHeader
  !system "echo custom header"
!macroend

!macro preInit
  ; Check for admin rights
  UserInfo::GetAccountType
  pop $0
  ${If} $0 != "admin"
    MessageBox mb_iconstop "Administrator rights required!"
    SetErrorLevel 740
    Quit
  ${EndIf}
!macroend

!macro customInit
  ; Custom initialization
!macroend

!macro customInstall
  ; Create registry entries for file associations
  WriteRegStr SHCTX "Software\Classes\.tachi" "" "TachikomaProject"
  WriteRegStr SHCTX "Software\Classes\TachikomaProject" "" "Tachikoma Project"
  WriteRegStr SHCTX "Software\Classes\TachikomaProject\DefaultIcon" "" "$INSTDIR\${APP_EXECUTABLE_FILENAME},0"
  WriteRegStr SHCTX "Software\Classes\TachikomaProject\shell\open\command" "" '"$INSTDIR\${APP_EXECUTABLE_FILENAME}" "%1"'

  ; Register protocol handler
  WriteRegStr SHCTX "Software\Classes\tachikoma" "" "URL:Tachikoma Protocol"
  WriteRegStr SHCTX "Software\Classes\tachikoma" "URL Protocol" ""
  WriteRegStr SHCTX "Software\Classes\tachikoma\shell\open\command" "" '"$INSTDIR\${APP_EXECUTABLE_FILENAME}" "%1"'

  ; Add to PATH (optional)
  ; EnVar::AddValue "PATH" "$INSTDIR"
!macroend

!macro customUnInstall
  ; Remove registry entries
  DeleteRegKey SHCTX "Software\Classes\.tachi"
  DeleteRegKey SHCTX "Software\Classes\TachikomaProject"
  DeleteRegKey SHCTX "Software\Classes\tachikoma"
!macroend