/** HTML 转义 */
export function escapeHtml(str) {
  if (!str) return '';
  const div = document.createElement('div');
  div.textContent = String(str);
  return div.innerHTML;
}

/** 防抖 */
export function debounce(fn, delay = 300) {
  let timer;
  return (...args) => {
    clearTimeout(timer);
    timer = setTimeout(() => fn(...args), delay);
  };
}

/** 格式化错误信息 */
export function formatError(context, err) {
  try {
    const parsed = JSON.parse(err);
    return `${context ? context + '：' : ''}${parsed.message || err}`;
  } catch {
    return `${context ? context + '：' : ''}${err}`;
  }
}

/** 生成唯一 ID */
export function generateId() {
  return Date.now().toString(36) + Math.random().toString(36).slice(2, 8);
}
