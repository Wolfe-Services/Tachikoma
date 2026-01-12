; build/windows/installer.nsh

!macro customHeader
  !system "echo Custom Tachikoma NSIS header"
!macroend

!macro preInit
  ; Set installation directory based on architecture
  ${If} ${RunningX64}
    StrCpy $INSTDIR "$PROGRAMFILES64\${PRODUCT_NAME}"
  ${Else}
    StrCpy $INSTDIR "$PROGRAMFILES\${PRODUCT_NAME}"
  ${EndIf}
!macroend

!macro customInit
  ; Check for running instance using mutex
  System::Call 'kernel32::CreateMutex(p 0, i 0, t "TachikomaMutex") p .r1 ?e'
  Pop $R0
  ${If} $R0 != 0
    MessageBox MB_OK|MB_ICONEXCLAMATION "Tachikoma is already running. Please close it before installing." /SD IDOK
    Abort
  ${EndIf}

  ; Check Windows version compatibility
  ${If} ${IsWin10}
    DetailPrint "Windows 10/11 detected - full features available"
  ${ElseIf} ${IsWin8}
    DetailPrint "Windows 8/8.1 detected - some features may be limited"
  ${ElseIf} ${IsWin7}
    DetailPrint "Windows 7 detected - legacy mode"
    MessageBox MB_YESNO|MB_ICONQUESTION "Windows 7 is no longer fully supported. Continue installation?" /SD IDYES IDNO abort_install
  ${Else}
    MessageBox MB_OK|MB_ICONSTOP "Windows 7 or later is required."
    Abort
  ${EndIf}

  abort_install:
    Abort
!macroend

!macro customInstall
  ; Create registry entries for file associations
  WriteRegStr SHCTX "Software\Classes\.tachi" "" "TachikomaProject"
  WriteRegStr SHCTX "Software\Classes\.tachi" "Content Type" "application/x-tachikoma"
  WriteRegStr SHCTX "Software\Classes\.tachi" "PerceivedType" "document"
  WriteRegStr SHCTX "Software\Classes\.tachikoma" "" "TachikomaProject"
  WriteRegStr SHCTX "Software\Classes\.tachikoma" "Content Type" "application/x-tachikoma"
  WriteRegStr SHCTX "Software\Classes\.tachikoma" "PerceivedType" "document"
  
  WriteRegStr SHCTX "Software\Classes\TachikomaProject" "" "Tachikoma Project"
  WriteRegStr SHCTX "Software\Classes\TachikomaProject" "FriendlyTypeName" "Tachikoma Project File"
  WriteRegStr SHCTX "Software\Classes\TachikomaProject\DefaultIcon" "" "$INSTDIR\${APP_EXECUTABLE_FILENAME},0"
  WriteRegStr SHCTX "Software\Classes\TachikomaProject\shell" "" "open"
  WriteRegStr SHCTX "Software\Classes\TachikomaProject\shell\open" "" "Open with Tachikoma"
  WriteRegStr SHCTX "Software\Classes\TachikomaProject\shell\open\command" "" '"$INSTDIR\${APP_EXECUTABLE_FILENAME}" "%1"'
  WriteRegStr SHCTX "Software\Classes\TachikomaProject\shell\edit" "" "Edit with Tachikoma"
  WriteRegStr SHCTX "Software\Classes\TachikomaProject\shell\edit\command" "" '"$INSTDIR\${APP_EXECUTABLE_FILENAME}" "%1"'

  ; Register protocol handler
  WriteRegStr SHCTX "Software\Classes\tachikoma" "" "URL:Tachikoma Protocol"
  WriteRegStr SHCTX "Software\Classes\tachikoma" "URL Protocol" ""
  WriteRegStr SHCTX "Software\Classes\tachikoma" "DefaultIcon" "$INSTDIR\${APP_EXECUTABLE_FILENAME},1"
  WriteRegStr SHCTX "Software\Classes\tachikoma\DefaultIcon" "" "$INSTDIR\${APP_EXECUTABLE_FILENAME},1"
  WriteRegStr SHCTX "Software\Classes\tachikoma\shell" "" "open"
  WriteRegStr SHCTX "Software\Classes\tachikoma\shell\open\command" "" '"$INSTDIR\${APP_EXECUTABLE_FILENAME}" "%1"'

  ; Register application capabilities
  WriteRegStr SHCTX "Software\RegisteredApplications" "Tachikoma" "Software\Tachikoma\Capabilities"
  WriteRegStr SHCTX "Software\Tachikoma\Capabilities" "ApplicationDescription" "Modern development environment for building amazing applications"
  WriteRegStr SHCTX "Software\Tachikoma\Capabilities" "ApplicationName" "Tachikoma"
  WriteRegStr SHCTX "Software\Tachikoma\Capabilities\FileAssociations" ".tachi" "TachikomaProject"
  WriteRegStr SHCTX "Software\Tachikoma\Capabilities\FileAssociations" ".tachikoma" "TachikomaProject"
  WriteRegStr SHCTX "Software\Tachikoma\Capabilities\UrlAssociations" "tachikoma" "TachikomaProject"

  ; Add to App Paths for command line access
  WriteRegStr SHCTX "Software\Microsoft\Windows\CurrentVersion\App Paths\tachikoma.exe" "" "$INSTDIR\${APP_EXECUTABLE_FILENAME}"
  WriteRegStr SHCTX "Software\Microsoft\Windows\CurrentVersion\App Paths\tachikoma.exe" "Path" "$INSTDIR"

  ; Register for Windows notifications (Toast notifications)
  WriteRegStr SHCTX "Software\Classes\AppUserModelId\io.tachikoma.app" "DisplayName" "Tachikoma"
  WriteRegStr SHCTX "Software\Classes\AppUserModelId\io.tachikoma.app" "IconUri" "$INSTDIR\${APP_EXECUTABLE_FILENAME}"

  ; Register with Windows Security Center (SmartScreen compatibility)
  WriteRegStr SHCTX "Software\Microsoft\Windows\CurrentVersion\Ext\PreApproved\{GUID}" "" ""

  ; Windows Defender exclusion hint (user can manually add)
  DetailPrint "Consider adding $INSTDIR to Windows Defender exclusions for better performance"

  ; Create start menu group with additional shortcuts
  CreateDirectory "$SMPROGRAMS\Tachikoma"
  CreateShortcut "$SMPROGRAMS\Tachikoma\Tachikoma.lnk" "$INSTDIR\${APP_EXECUTABLE_FILENAME}" "" "$INSTDIR\${APP_EXECUTABLE_FILENAME}" 0
  CreateShortcut "$SMPROGRAMS\Tachikoma\Uninstall Tachikoma.lnk" "$INSTDIR\Uninstall ${PRODUCT_NAME}.exe" "" "$INSTDIR\Uninstall ${PRODUCT_NAME}.exe" 0

  ; Add quick launch icons for Windows taskbar
  ${If} ${AtLeastWin7}
    ; Pin to taskbar (user action required)
    DetailPrint "You can pin Tachikoma to the taskbar for quick access"
  ${EndIf}

  ; Refresh shell icons and file associations
  System::Call 'Shell32::SHChangeNotify(i 0x8000000, i 0, p 0, p 0)'
  
  ; Update registry timestamp for faster shell integration
  WriteRegDWORD SHCTX "Software\Classes\TachikomaProject" "EditFlags" 0x00210000
!macroend

!macro customUnInstall
  ; Remove file type associations
  DeleteRegKey SHCTX "Software\Classes\.tachi"
  DeleteRegKey SHCTX "Software\Classes\.tachikoma"
  DeleteRegKey SHCTX "Software\Classes\TachikomaProject"
  DeleteRegKey SHCTX "Software\Classes\tachikoma"
  
  ; Remove application capabilities
  DeleteRegKey SHCTX "Software\Tachikoma"
  DeleteRegValue SHCTX "Software\RegisteredApplications" "Tachikoma"
  
  ; Remove app paths
  DeleteRegKey SHCTX "Software\Microsoft\Windows\CurrentVersion\App Paths\tachikoma.exe"
  
  ; Remove toast notification registration
  DeleteRegKey SHCTX "Software\Classes\AppUserModelId\io.tachikoma.app"
  
  ; Remove start menu group
  RMDir /r "$SMPROGRAMS\Tachikoma"

  ; Refresh shell icons
  System::Call 'Shell32::SHChangeNotify(i 0x8000000, i 0, p 0, p 0)'
!macroend

!macro customInstallMode
  ; Default to per-user installation for better security
  StrCpy $isForceCurrentInstall "1"
!macroend

!macro customPageAfterWelcome
  ; Show custom info page
  !insertmacro MUI_PAGE_LICENSE "${LICENSE_FILE}"
!macroend