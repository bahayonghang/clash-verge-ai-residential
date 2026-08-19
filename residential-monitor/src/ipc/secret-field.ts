/** 设置页密钥框。密钥由 JS 写入 input.value，不得插进 HTML 模板。 */

export function secretFieldMarkup(): string {
  return `<label class="secret-label">TCP secret
    <span class="secret-field">
      <input id="controller-secret" type="password" autocomplete="off" spellcheck="false" />
      <button type="button" id="toggle-secret" class="secret-toggle" aria-label="显示密钥" aria-pressed="false" title="显示密钥">
        <svg class="icon-show" width="18" height="18" viewBox="0 0 24 24" aria-hidden="true">
          <path fill="none" stroke="currentColor" stroke-width="1.75" stroke-linecap="round" stroke-linejoin="round" d="M2.5 12s3.5-7 9.5-7 9.5 7 9.5 7-3.5 7-9.5 7-9.5-7-9.5-7Z"/>
          <circle fill="none" stroke="currentColor" stroke-width="1.75" cx="12" cy="12" r="3"/>
        </svg>
        <svg class="icon-hide" width="18" height="18" viewBox="0 0 24 24" aria-hidden="true">
          <path fill="none" stroke="currentColor" stroke-width="1.75" stroke-linecap="round" stroke-linejoin="round" d="M3 3l18 18"/>
          <path fill="none" stroke="currentColor" stroke-width="1.75" stroke-linecap="round" stroke-linejoin="round" d="M10.6 10.6A3 3 0 0 0 12 15a3 3 0 0 0 3-3 3 3 0 0 0-.6-1.8"/>
          <path fill="none" stroke="currentColor" stroke-width="1.75" stroke-linecap="round" stroke-linejoin="round" d="M9.9 5.2A11 11 0 0 1 12 5c6 0 9.5 7 9.5 7a16 16 0 0 1-3.2 4.1M6.1 6.1C4 7.8 2.5 12 2.5 12A16 16 0 0 0 8 16.9"/>
        </svg>
      </button>
    </span>
    <span class="field-hint">保存后回填此框，默认显示圆点。不会写入日志或 Channel。</span>
  </label>`;
}

export function applySecretField(
  root: ParentNode,
  value: string,
  visible: boolean
): void {
  const input = root.querySelector("#controller-secret");
  if (!(input instanceof HTMLInputElement)) {
    return;
  }
  input.value = value;
  input.type = visible ? "text" : "password";
  const button = root.querySelector("#toggle-secret");
  if (button instanceof HTMLButtonElement) {
    button.setAttribute("aria-pressed", visible ? "true" : "false");
    const label = visible ? "隐藏密钥" : "显示密钥";
    button.setAttribute("aria-label", label);
    button.title = label;
  }
}
