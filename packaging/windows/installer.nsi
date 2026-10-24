; HTL Content Guard PC — one-click Windows installer (built with NSIS).
; Produces a single Setup .exe: the user double-clicks it, approves the UAC
; prompt, and everything is installed (service registered, started, shortcut,
; Add/Remove Programs entry). The uninstaller honours uninstall protection.
;
; Build (on Linux or Windows):
;   makensis -DAGENT=<path to contentguard-agent.exe> -DEXTDIR=<path to extension/> installer.nsi

!include "MUI2.nsh"
!include "LogicLib.nsh"

!ifndef AGENT
  !define AGENT "contentguard-agent.exe"
!endif
!ifndef EXTDIR
  !define EXTDIR "extension"
!endif

!define APPNAME     "HTL Content Guard PC"
!define SERVICE     "ContentGuard"
!define URL         "http://127.0.0.1:17345/"
!define VERSION     "1.0.0"
!define REGKEY      "Software\Microsoft\Windows\CurrentVersion\Uninstall\${SERVICE}"

Name "${APPNAME}"
OutFile "HTL-Content-Guard-PC-Setup.exe"
Unicode True
InstallDir "$PROGRAMFILES64\${APPNAME}"
RequestExecutionLevel admin
BrandingText "Halal Tech Life"

!define MUI_ICON   "appicon.ico"
!define MUI_UNICON "appicon.ico"
!define MUI_ABORTWARNING

!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES
!define MUI_FINISHPAGE_RUN ""
!define MUI_FINISHPAGE_RUN_TEXT "Open HTL Content Guard PC now"
!define MUI_FINISHPAGE_RUN_FUNCTION OpenApp
!insertmacro MUI_PAGE_FINISH

!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES
!insertmacro MUI_LANGUAGE "English"

Function OpenApp
  ExecShell "open" "${URL}"
FunctionEnd

Section "Install"
  SetOutPath "$INSTDIR"

  ; Stop an existing service before overwriting its exe.
  nsExec::Exec 'net stop "${SERVICE}"'
  Pop $0

  File "/oname=contentguard-agent.exe" "${AGENT}"
  File /r "${EXTDIR}"

  ; (Re)create the Windows service that does the blocking.
  nsExec::Exec 'sc.exe create "${SERVICE}" binPath= "\"$INSTDIR\contentguard-agent.exe\" service" start= auto DisplayName= "${APPNAME}"'
  Pop $0
  nsExec::Exec 'sc.exe description "${SERVICE}" "HTL Content Guard PC — website and app blocker"'
  Pop $0
  ; Auto-restart the service if it ever stops unexpectedly.
  nsExec::Exec 'sc.exe failure "${SERVICE}" reset= 86400 actions= restart/5000/restart/5000/restart/30000'
  Pop $0
  nsExec::Exec 'sc.exe start "${SERVICE}"'
  Pop $0

  ; Start-menu shortcut to the local UI.
  FileOpen $0 "$SMPROGRAMS\${APPNAME}.url" w
  FileWrite $0 "[InternetShortcut]$\r$\n"
  FileWrite $0 "URL=${URL}$\r$\n"
  FileWrite $0 "IconFile=$INSTDIR\contentguard-agent.exe$\r$\n"
  FileWrite $0 "IconIndex=0$\r$\n"
  FileClose $0

  ; Uninstaller + Add/Remove Programs entry.
  WriteUninstaller "$INSTDIR\uninstall.exe"
  WriteRegStr   HKLM "${REGKEY}" "DisplayName"     "${APPNAME}"
  WriteRegStr   HKLM "${REGKEY}" "DisplayVersion"  "${VERSION}"
  WriteRegStr   HKLM "${REGKEY}" "Publisher"       "Halal Tech Life"
  WriteRegStr   HKLM "${REGKEY}" "DisplayIcon"     "$INSTDIR\contentguard-agent.exe"
  WriteRegStr   HKLM "${REGKEY}" "InstallLocation" "$INSTDIR"
  WriteRegStr   HKLM "${REGKEY}" "UninstallString" '"$INSTDIR\uninstall.exe"'
  WriteRegDWORD HKLM "${REGKEY}" "NoModify" 1
  WriteRegDWORD HKLM "${REGKEY}" "NoRepair" 1
SectionEnd

Section "Uninstall"
  ; Honour uninstall protection: the agent exits non-zero when uninstalling is
  ; not currently allowed (cooldown / challenge not completed).
  nsExec::Exec '"$INSTDIR\contentguard-agent.exe" uninstall-check'
  Pop $0
  ${If} $0 != 0
    MessageBox MB_OK|MB_ICONSTOP "Uninstall is protected right now. Open HTL Content Guard PC, complete the uninstall request / wait for the cooldown, then try again."
    Abort
  ${EndIf}

  nsExec::Exec 'net stop "${SERVICE}"'
  Pop $0
  nsExec::Exec 'sc.exe delete "${SERVICE}"'
  Pop $0

  Delete "$SMPROGRAMS\${APPNAME}.url"
  RMDir /r "$INSTDIR\extension"
  Delete "$INSTDIR\contentguard-agent.exe"
  Delete "$INSTDIR\uninstall.exe"
  RMDir "$INSTDIR"
  DeleteRegKey HKLM "${REGKEY}"
SectionEnd
