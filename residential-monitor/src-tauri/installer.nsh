; 卸载时保留数据目录。Tauri 卸载器结尾会 RMDir /r $INSTDIR，
; PREUNINSTALL 先把 data 搬到临时位置，POSTUNINSTALL 再搬回，
; 维持 docs/data-directory.md「普通卸载保留数据」契约。

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
