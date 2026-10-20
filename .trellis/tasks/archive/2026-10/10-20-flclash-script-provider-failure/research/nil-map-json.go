package main
import (
 "encoding/json"
 "fmt"
)
type RawConfigShape struct {
 RuleProvider map[string]map[string]any `json:"rule-providers"`
}
func main() {
 for _, row := range []struct{name string; value RawConfigShape}{
  {"omitted section / nil map", RawConfigShape{}},
  {"empty mapping", RawConfigShape{RuleProvider: map[string]map[string]any{}}},
 } {
  data,err:=json.Marshal(row.value)
  if err!=nil {panic(err)}
  fmt.Printf("%s => %s\n",row.name,data)
 }
}
