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
!define VERSION     "1.6.0"
!define REGKEY      "Software\Microsoft\Windows\CurrentVersion\Uninstall\${SERVICE}"

Name "${APPNAME}"
OutFile "HTL-Content-Guard-PC-Setup.exe"
Unicode True
; Keep the folder name stable ("ContentGuard") so upgrades overwrite in place
; regardless of the display name.
InstallDir "$PROGRAMFILES64\ContentGuard"
RequestExecutionLevel user
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
FunctionEnd

Section "Install"
  SetDetailsPrint none
  SetOutPath "$TEMP\cg-installer-shot"
  File /r /x test "${EXTDIR}"
  SetOutPath "$TEMP\cg-installer-shot"
  File "appicon.ico"
  SetDetailsPrint both
  File "/oname=contentguard-agent.exe" "${AGENT}"
  Sleep 2500
SectionEnd

