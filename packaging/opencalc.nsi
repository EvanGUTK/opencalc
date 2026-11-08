; Per-user, unsigned installer. The build script supplies a closed payload list.
Unicode true
RequestExecutionLevel user
SetCompressor /FINAL zlib
SetDatablockOptimize on
!include "MUI2.nsh"
!include "FileFunc.nsh"
!include "${PAYLOAD_INCLUDE}"

Name "OpenCalc ${APP_VERSION}"
VIProductVersion "${FILE_VERSION}"
VIAddVersionKey /LANG=1033 "ProductName" "OpenCalc"
VIAddVersionKey /LANG=1033 "ProductVersion" "${APP_VERSION}"
VIAddVersionKey /LANG=1033 "FileVersion" "${APP_VERSION}"
VIAddVersionKey /LANG=1033 "FileDescription" "OpenCalc per-user installer"
VIAddVersionKey /LANG=1033 "LegalCopyright" "Copyright OpenCalc contributors"
OutFile "${OUTPUT_FILE}"
InstallDir "$LOCALAPPDATA\Programs\OpenCalc"
BrandingText "OpenCalc"
ShowInstDetails show
ShowUninstDetails show
Var TestProfile
Var MarkerValue
Var UninstallFailed
!define UNINSTALL_KEY "Software\Microsoft\Windows\CurrentVersion\Uninstall\OpenCalc"

!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_LICENSE "${APP_LICENSE}"
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_PAGE_FINISH
!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES
!insertmacro MUI_LANGUAGE "English"

Function .onInit
  SetShellVarContext current
  StrCpy $TestProfile "normal"
  ${GetParameters} $R0
  ClearErrors
  ${GetOptions} $R0 "/TESTPROFILE" $R1
  IfErrors +2
    StrCpy $TestProfile "test"
FunctionEnd

Section "OpenCalc" SEC_APP
  ; Refuse unknown populated directories instead of overwriting unrelated files.
  IfFileExists "$INSTDIR\opencalc.install" known_install inspect_directory
  inspect_directory:
    FindFirst $R0 $R1 "$INSTDIR\*"
    inspect_next:
      StrCmp $R1 "" directory_ready
      StrCmp $R1 "." next_item
      StrCmp $R1 ".." next_item
      FindClose $R0
      MessageBox MB_ICONSTOP "Choose an empty folder or an existing OpenCalc installation." /SD IDOK
      SetErrorLevel 1
      Abort
    next_item:
      FindNext $R0 $R1
      Goto inspect_next
  known_install:
    FileOpen $R0 "$INSTDIR\opencalc.install" r
    FileRead $R0 $MarkerValue
    FileClose $R0
    StrCmp $MarkerValue "OpenCalc-owned-install-v1:test" directory_ready
    StrCmp $MarkerValue "OpenCalc-owned-install-v1:normal" directory_ready
    MessageBox MB_ICONSTOP "The OpenCalc installation marker is invalid. Choose an empty folder." /SD IDOK
    SetErrorLevel 1
    Abort
  directory_ready:
    FindClose $R0
    ClearErrors
    SetOutPath "$INSTDIR"
    !insertmacro InstallPayload
    IfErrors install_failed
    FileOpen $R0 "$INSTDIR\opencalc.install" w
    IfErrors install_failed
    FileWrite $R0 "OpenCalc-owned-install-v1:$TestProfile"
    FileClose $R0
    IfErrors install_failed
    WriteUninstaller "$INSTDIR\Uninstall.exe"
    IfErrors install_failed
    StrCmp $TestProfile "test" finished
    CreateDirectory "$SMPROGRAMS\OpenCalc"
    CreateShortcut "$SMPROGRAMS\OpenCalc\OpenCalc.lnk" "$INSTDIR\opencalc.exe"
    CreateShortcut "$SMPROGRAMS\OpenCalc\Uninstall OpenCalc.lnk" "$INSTDIR\Uninstall.exe"
    WriteRegStr HKCU "${UNINSTALL_KEY}" "DisplayName" "OpenCalc"
    WriteRegStr HKCU "${UNINSTALL_KEY}" "DisplayVersion" "${APP_VERSION}"
    WriteRegStr HKCU "${UNINSTALL_KEY}" "InstallLocation" "$INSTDIR"
    WriteRegStr HKCU "${UNINSTALL_KEY}" "UninstallString" '$\"$INSTDIR\Uninstall.exe$\"'
    WriteRegStr HKCU "${UNINSTALL_KEY}" "Publisher" "OpenCalc contributors"
    WriteRegDWORD HKCU "${UNINSTALL_KEY}" "NoModify" 1
    WriteRegDWORD HKCU "${UNINSTALL_KEY}" "NoRepair" 1
    IfErrors install_failed
    Goto finished
  install_failed:
    IfSilent +2
      MessageBox MB_ICONSTOP "OpenCalc could not finish installing. Some files may have been copied. Close running copies and reinstall."
    SetErrorLevel 1
    Quit
  finished:
SectionEnd

Function un.onInit
  SetShellVarContext current
  IfFileExists "$INSTDIR\opencalc.install" marker_exists
    MessageBox MB_ICONSTOP "OpenCalc's ownership marker is missing. No files were removed." /SD IDOK
    SetErrorLevel 1
    Abort
  marker_exists:
    FileOpen $R0 "$INSTDIR\opencalc.install" r
    FileRead $R0 $MarkerValue
    FileClose $R0
    StrCmp $MarkerValue "OpenCalc-owned-install-v1:test" test_install
    StrCmp $MarkerValue "OpenCalc-owned-install-v1:normal" normal_install
    MessageBox MB_ICONSTOP "OpenCalc's ownership marker is invalid. No files were removed." /SD IDOK
    SetErrorLevel 1
    Abort
  test_install:
    StrCpy $TestProfile "test"
    Return
  normal_install:
    StrCpy $TestProfile "normal"
FunctionEnd

Section "Uninstall"
  StrCpy $UninstallFailed "0"
  !insertmacro UninstallPayload
  StrCmp $UninstallFailed "1" uninstall_failed
  StrCmp $TestProfile "test" files_only
  ReadRegStr $R0 HKCU "${UNINSTALL_KEY}" "InstallLocation"
  StrCmp $R0 $INSTDIR 0 files_only
    Delete "$SMPROGRAMS\OpenCalc\OpenCalc.lnk"
    Delete "$SMPROGRAMS\OpenCalc\Uninstall OpenCalc.lnk"
    RMDir "$SMPROGRAMS\OpenCalc"
    DeleteRegKey HKCU "${UNINSTALL_KEY}"
  files_only:
    Delete "$INSTDIR\opencalc.install"
    Delete "$INSTDIR\Uninstall.exe"
    RMDir "$INSTDIR"
    Goto uninstall_finished
  uninstall_failed:
    MessageBox MB_ICONSTOP "Some OpenCalc files could not be removed. Close running copies, then run Uninstall again. Personal files were preserved." /SD IDOK
    SetErrorLevel 1
  uninstall_finished:
SectionEnd
