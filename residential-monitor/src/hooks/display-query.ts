export interface DisplayRequest {
  signal: AbortSignal;
  isCurrent: () => boolean;
}

interface Intent {
  key: string;
  run: (request: DisplayRequest) => Promise<void>;
  done: () => void;
}

/** 每个视图只保留一个在途查询和最新意图；隐藏期间不启动查询。 */
export class DisplayQuery {
  private active = false;
  private revision = 0;
  private key: string | null = null;
  private latest: Intent | null = null;
  private pending: Intent | null = null;
  private running: AbortController | null = null;
  private scheduled = false;

  request(key: string, run: Intent["run"], replay = true): Promise<void> {
    if (key !== this.key) {
      this.revision += 1;
      this.running?.abort();
    }
    this.key = key;
    this.pending?.done();
    return new Promise((done) => {
      const intent = { key, run, done };
      if (replay) this.latest = intent;
      this.pending = intent;
      this.schedule();
    });
  }

  setActive(active: boolean): void {
    if (active === this.active) return;
    this.active = active;
    this.revision += 1;
    if (!active) this.running?.abort();
    this.pending = this.pending ?? this.latest;
    this.schedule();
  }

  clear(): void {
    this.setActive(false);
    this.pending?.done();
    this.pending = null;
    this.latest = null;
    this.key = null;
  }

  private schedule(): void {
    if (this.scheduled || !this.active || this.running || !this.pending) return;
    this.scheduled = true;
    // 同一轮的可见性、时钟和筛选变化先合并，再发出唯一请求。
    void Promise.resolve().then(() => {
      this.scheduled = false;
      if (!this.active || this.running || !this.pending) return;
      const intent = this.pending;
      this.pending = null;
      const revision = this.revision;
      const controller = new AbortController();
      this.running = controller;
      const request: DisplayRequest = {
        signal: controller.signal,
        isCurrent: () => this.active && revision === this.revision && !controller.signal.aborted
      };
      // run 自己处理本视图的错误；finally 总会让最新意图继续。
      void intent.run(request).finally(() => {
        intent.done();
        this.running = null;
        this.schedule();
      });
    });
  }
}
