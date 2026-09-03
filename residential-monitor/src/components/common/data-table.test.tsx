import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";
import {
  DataTableEmptyRow,
  DataTableTd,
  DataTableTh,
  dataTableClasses
} from "./data-table";

describe("共享表格规格", () => {
  it("数值列右对齐且等宽数字，表头与正文有档差", () => {
    const html = renderToStaticMarkup(
      <table className={dataTableClasses.table}>
        <thead>
          <tr className={dataTableClasses.headRow}>
            <DataTableTh>名称</DataTableTh>
            <DataTableTh numeric>下行</DataTableTh>
          </tr>
        </thead>
        <tbody>
          <tr className={dataTableClasses.row}>
            <DataTableTd>a.example</DataTableTd>
            <DataTableTd numeric>1.2 MB</DataTableTd>
          </tr>
        </tbody>
      </table>
    );
    expect(html).toContain("text-right");
    expect(html).toContain("tabular-nums");
    expect(html).toContain("text-xs font-medium text-muted-foreground");
    expect(html).toContain("hover:bg-muted/40");
    expect(html).toContain("border-border/40");
  });

  it("表格不无脑全宽拉伸，空态行走固定写法", () => {
    expect(dataTableClasses.table).toContain("w-auto");
    expect(dataTableClasses.table).not.toMatch(/(^| )w-full( |$)/);
    const html = renderToStaticMarkup(
      <table>
        <tbody>
          <DataTableEmptyRow colSpan={3}>暂无</DataTableEmptyRow>
        </tbody>
      </table>
    );
    expect(html).toContain('colSpan="3"');
    expect(html).toContain("text-muted-foreground");
  });
});
