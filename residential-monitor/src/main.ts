import { decodeShellStatus, type ShellStatus } from "./dto";

function render(status: ShellStatus): void {
  const app = document.querySelector("#app");
  if (!(app instanceof HTMLElement)) {
    return;
  }

  app.innerHTML = `
    <h1>家宽流量监控</h1>
    <p>C0 骨架：本窗口只加载打包内本地资源，不采集控制器、不访问数据库。</p>
    <section class="panel">
      <p>标识：<code>${status.identifier}</code></p>
      <p>阶段：<code>${status.phase}</code></p>
      <p>说明：${status.messageZh}</p>
    </section>
  `;
}

const status = decodeShellStatus({
  schemaVersion: 1,
  kind: "shellStatus",
  identifier: "io.github.bahayonghang.residential-monitor",
  phase: "c0-skeleton",
  messageZh: "采集、核算与正式 schema 属于后续任务。"
});

render(status);
