!macro NSIS_HOOK_PREINSTALL
  ; 0.4.1 renamed the installed executable from ai_pc_fac.exe to
  ; CS2BotImproverAssistant.exe. Tauri's generated check only sees the new
  ; binary name, so explicitly close the legacy process before file migration.
  !insertmacro CheckIfAppIsRunning "ai_pc_fac.exe" "${PRODUCTNAME}"
  Delete "$INSTDIR\ai_pc_fac.exe"
!macroend

!macro UpdateShortcutMainBinary shortcut
  !define UniqueID ${__LINE__}
  IfFileExists "${shortcut}" 0 done_${UniqueID}
    !insertmacro ComHlpr_CreateInProcInstance ${CLSID_ShellLink} ${IID_IShellLink} r0 ""
    ${If} $0 P<> 0
      ${IUnknown::QueryInterface} $0 '("${IID_IPersistFile}",.r1)'
      ${If} $1 P<> 0
        ${IPersistFile::Load} $1 '("${shortcut}", ${STGM_READWRITE})'
        ${IShellLink::SetPath} $0 '(w "$INSTDIR\${MAINBINARYNAME}.exe")'
        ${IShellLink::SetWorkingDirectory} $0 '(w "$INSTDIR")'
        ${IShellLink::SetIconLocation} $0 '(w "$INSTDIR\${MAINBINARYNAME}.exe", i 0)'
        ${IPersistFile::Save} $1 '("${shortcut}",1)'
        ${IUnknown::Release} $1 ""
      ${EndIf}
      ${IUnknown::Release} $0 ""
    ${EndIf}
    !insertmacro SetLnkAppUserModelId "${shortcut}"
  done_${UniqueID}:
  !undef UniqueID
!macroend

!macro NSIS_HOOK_POSTINSTALL
  ; Tauri migrates old shortcut targets when the main binary name changes, but
  ; it does not update stale IconLocation values. Keep this hook limited to
  ; shortcut metadata; do not refresh Explorer icon caches from the installer.
  !insertmacro UpdateShortcutMainBinary "$DESKTOP\${PRODUCTNAME}.lnk"
  !insertmacro UpdateShortcutMainBinary "$SMPROGRAMS\${PRODUCTNAME}.lnk"
  !insertmacro UpdateShortcutMainBinary "$SMPROGRAMS\$AppStartMenuFolder\${PRODUCTNAME}.lnk"
!macroend
