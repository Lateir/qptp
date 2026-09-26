!macro NSIS_HOOK_POSTINSTALL
  IfFileExists "$INSTDIR\driver_qptp\bin\win64\driver_qptp.dll" +2 0
    MessageBox MB_ICONEXCLAMATION "QPTP SteamVR driver was not bundled. Rebuild the installer with build-installer.bat."
  ExecWait '"$INSTDIR\qptp.exe" --register-steamvr' $0
  ${If} $0 != 0
    MessageBox MB_ICONEXCLAMATION "QPTP could not register its SteamVR driver. Install SteamVR, then launch QPTP to retry registration."
  ${EndIf}
!macroend

!macro NSIS_HOOK_PREUNINSTALL
  ExecWait '"$INSTDIR\qptp.exe" --unregister-steamvr' $0
  ${If} $0 != 0
    DetailPrint "QPTP SteamVR driver was not registered or SteamVR is unavailable."
  ${EndIf}
!macroend
