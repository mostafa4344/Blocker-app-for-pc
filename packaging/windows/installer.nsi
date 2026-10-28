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
!define VERSION     "1.4.1"
!define REGKEY      "Software\Microsoft\Windows\CurrentVersion\Uninstall\${SERVICE}"

Name "${APPNAME}"
OutFile "HTL-Content-Guard-PC-Setup.exe"
Unicode True
; Keep the folder name stable ("ContentGuard") so upgrades overwrite in place
; regardless of the display name.
InstallDir "$PROGRAMFILES64\ContentGuard"
RequestExecutionLevel admin
BrandingText "Halal Tech Life"

; File properties shown by Windows (and checked by antivirus reputation).
VIProductVersion "${VERSION}.0"
VIAddVersionKey "ProductName"     "${APPNAME}"
VIAddVersionKey "CompanyName"     "Halal Tech Life"
VIAddVersionKey "FileDescription" "${APPNAME} Setup"
VIAddVersionKey "FileVersion"     "${VERSION}"
VIAddVersionKey "ProductVersion"  "${VERSION}"
VIAddVersionKey "LegalCopyright"  "Halal Tech Life"

; Code signing (Authenticode). build-release.ps1 passes -DSIGN_CMD=... when a
; certificate is available; NSIS then signs the uninstaller it generates and the
; finished Setup.exe ("%1" is the file to sign).
!ifdef SIGN_CMD
  !uninstfinalize '${SIGN_CMD} "%1"' = 0
  !finalize '${SIGN_CMD} "%1"' = 0
!endif

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

; Open the UI as a standalone app window (no tabs / address bar) using Edge or
; Chrome in --app mode; fall back to the default browser if neither is found.
!macro LaunchApp
  StrCpy $1 ""
  ${If} ${FileExists} "$PROGRAMFILES32\Microsoft\Edge\Application\msedge.exe"
    StrCpy $1 "$PROGRAMFILES32\Microsoft\Edge\Application\msedge.exe"
  ${ElseIf} ${FileExists} "$PROGRAMFILES64\Google\Chrome\Application\chrome.exe"
    StrCpy $1 "$PROGRAMFILES64\Google\Chrome\Application\chrome.exe"
  ${ElseIf} ${FileExists} "$PROGRAMFILES32\Google\Chrome\Application\chrome.exe"
    StrCpy $1 "$PROGRAMFILES32\Google\Chrome\Application\chrome.exe"
  ${EndIf}
  ${If} $1 != ""
    Exec '"$1" --app=${URL} --window-size=1120,820'
  ${Else}
    ExecShell "open" "${URL}"
  ${EndIf}
!macroend

Function OpenApp
  !insertmacro LaunchApp
FunctionEnd

Section "Install"
  SetOutPath "$INSTDIR"

  ; Upgrade in place. While protection is on the running agent refuses to stop,
  ; so: move the running exe aside (Windows allows renaming a running exe),
  ; drop the new one in, point the service at it, then restart the process.
  ; Settings and blocks in %ProgramData%\ContentGuard are kept.
  Delete "$INSTDIR\contentguard-agent.old"
  ${If} ${FileExists} "$INSTDIR\contentguard-agent.exe"
    Rename "$INSTDIR\contentguard-agent.exe" "$INSTDIR\contentguard-agent.old"
  ${EndIf}
  ; Remove a stray copy left by an earlier installer that used the display name
  ; as the folder, so we don't leave two copies behind.
  RMDir /r "$PROGRAMFILES64\HTL Content Guard PC"

  File "/oname=contentguard-agent.exe" "${AGENT}"
  ; The app logo, used by the Start-menu shortcut and Apps & features.
  File "appicon.ico"
  RMDir /r "$INSTDIR\extension"
  File /r /x test "${EXTDIR}"

  ; Create the service (fails harmlessly if it exists), then (re)configure it.
  nsExec::Exec 'sc.exe create "${SERVICE}" binPath= "\"$INSTDIR\contentguard-agent.exe\" service" start= auto DisplayName= "${APPNAME}"'
  Pop $0
  nsExec::Exec 'sc.exe config "${SERVICE}" binPath= "\"$INSTDIR\contentguard-agent.exe\" service" start= auto DisplayName= "${APPNAME}"'
  Pop $0
  nsExec::Exec 'sc.exe description "${SERVICE}" "HTL Content Guard PC - website and app blocker"'
  Pop $0
  ; Restart automatically if the process ever ends (crash or forced kill).
  nsExec::Exec 'sc.exe failure "${SERVICE}" reset= 86400 actions= restart/3000/restart/3000/restart/10000'
  Pop $0
  nsExec::Exec 'sc.exe failureflag "${SERVICE}" 1'
  Pop $0
  ; Restart into the new exe: a normal stop when allowed, otherwise end the old
  ; process and let the recovery action above start the new one.
  nsExec::Exec 'net stop "${SERVICE}"'
  Pop $0
  nsExec::Exec 'taskkill /F /IM contentguard-agent.exe'
  Pop $0
  Sleep 1500
  nsExec::Exec 'sc.exe start "${SERVICE}"'
  Pop $0
  Sleep 4000
  nsExec::Exec 'sc.exe start "${SERVICE}"'
  Pop $0
  Delete "$INSTDIR\contentguard-agent.old"

  ; Start-menu shortcut that opens the UI as a standalone app window.
  StrCpy $2 ""
  ${If} ${FileExists} "$PROGRAMFILES32\Microsoft\Edge\Application\msedge.exe"
    StrCpy $2 "$PROGRAMFILES32\Microsoft\Edge\Application\msedge.exe"
  ${ElseIf} ${FileExists} "$PROGRAMFILES64\Google\Chrome\Application\chrome.exe"
    StrCpy $2 "$PROGRAMFILES64\Google\Chrome\Application\chrome.exe"
  ${ElseIf} ${FileExists} "$PROGRAMFILES32\Google\Chrome\Application\chrome.exe"
    StrCpy $2 "$PROGRAMFILES32\Google\Chrome\Application\chrome.exe"
  ${EndIf}
  ${If} $2 != ""
    CreateShortCut "$SMPROGRAMS\${APPNAME}.lnk" "$2" "--app=${URL} --window-size=1120,820" "$INSTDIR\appicon.ico" 0
  ${Else}
    FileOpen $0 "$SMPROGRAMS\${APPNAME}.url" w
    FileWrite $0 "[InternetShortcut]$\r$\n"
    FileWrite $0 "URL=${URL}$\r$\n"
    FileWrite $0 "IconFile=$INSTDIR\appicon.ico$\r$\n"
    FileWrite $0 "IconIndex=0$\r$\n"
    FileClose $0
  ${EndIf}

  ; Uninstaller + Add/Remove Programs entry.
  WriteUninstaller "$INSTDIR\uninstall.exe"
  WriteRegStr   HKLM "${REGKEY}" "DisplayName"     "${APPNAME}"
  WriteRegStr   HKLM "${REGKEY}" "DisplayVersion"  "${VERSION}"
  WriteRegStr   HKLM "${REGKEY}" "Publisher"       "Halal Tech Life"
  WriteRegStr   HKLM "${REGKEY}" "DisplayIcon"     "$INSTDIR\appicon.ico"
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

  ; The approval lets the service accept Stop within a second or two.
  Sleep 2500
  nsExec::Exec 'sc.exe failure "${SERVICE}" reset= 0 actions= ""'
  Pop $0
  nsExec::Exec 'net stop "${SERVICE}"'
  Pop $0
  nsExec::Exec 'sc.exe delete "${SERVICE}"'
  Pop $0
  ; Remove our hosts-file section and browser policies.
  nsExec::Exec '"$INSTDIR\contentguard-agent.exe" cleanup'
  Pop $0

  Delete "$SMPROGRAMS\${APPNAME}.lnk"
  Delete "$SMPROGRAMS\${APPNAME}.url"
  RMDir /r "$INSTDIR\extension"
  Delete "$INSTDIR\contentguard-agent.exe"
  Delete "$INSTDIR\contentguard-agent.old"
  Delete "$INSTDIR\uninstall.exe"
  Delete "$INSTDIR\appicon.ico"
  RMDir "$INSTDIR"
  DeleteRegKey HKLM "${REGKEY}"
SectionEnd
