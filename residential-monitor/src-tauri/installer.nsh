; 卸载时保留数据目录。Tauri 卸载器结尾会 RMDir /r $INSTDIR，
; PREUNINSTALL 先把 data 搬到临时位置，POSTUNINSTALL 再搬回，
; 维持 docs/data-directory.md「普通卸载保留数据」契约。
;
; 若上次安装目录落在 $TEMP 下（例如 hook 试装把路径写进注册表），
; PREINSTALL 改到 $LOCALAPPDATA\${PRODUCTNAME} 再拷贝文件，
; 并把已有 data 一并搬走。$R9 在 PREINSTALL 记下旧路径，POSTINSTALL 清理旧二进制。

!macro NSIS_HOOK_PREINSTALL
  StrCpy $R9 ""
  StrLen $R6 $TEMP
  StrCpy $R5 $INSTDIR $R6
  ${StrCase} $R8 $R5 "L"
  ${StrCase} $R7 $TEMP "L"
  ${If} $R8 == $R7
    StrCpy $R9 $INSTDIR
    StrCpy $INSTDIR "$LOCALAPPDATA\${PRODUCTNAME}"
    CreateDirectory "$INSTDIR"
    ${If} ${FileExists} "$R9\data\monitor.sqlite3"
    ${AndIfNot} ${FileExists} "$INSTDIR\data\monitor.sqlite3"
      RMDir "$INSTDIR\data"
      Rename "$R9\data" "$INSTDIR\data"
    ${EndIf}
    SetOutPath $INSTDIR
  ${EndIf}
!macroend

!macro NSIS_HOOK_POSTINSTALL
  ${If} $R9 != ""
    Delete "$R9\${MAINBINARYNAME}.exe"
    Delete "$R9\monitor-bench.exe"
    Delete "$R9\uninstall.exe"
    RMDir "$R9"
  ${EndIf}
!macroend

!macro NSIS_HOOK_PREUNINSTALL
  ${If} ${FileExists} "$INSTDIR\data\monitor.sqlite3"
    CreateDirectory "$TEMP\residential-monitor-data-keep"
    Rename "$INSTDIR\data" "$TEMP\residential-monitor-data-keep\data"
  ${EndIf}
!macroend

!macro NSIS_HOOK_POSTUNINSTALL
  ${If} ${FileExists} "$TEMP\residential-monitor-data-keep\data"
    CreateDirectory "$INSTDIR"
    Rename "$TEMP\residential-monitor-data-keep\data" "$INSTDIR\data"
    RMDir "$TEMP\residential-monitor-data-keep"
  ${EndIf}
!macroend
