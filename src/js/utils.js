// utils.js - KurisuGal 工具函数模块

/**
 * 防抖函数
 */
export function debounce(fn, delay = 300) {
  let timer;
  return (...args) => {
    clearTimeout(timer);
    timer = setTimeout(() => fn(...args), delay);
  };
}

/**
 * 显示 Toast 通知
 */
export function showToast(type, message, duration = 3500) {
  const container = document.getElementById('toast-container');
  const toast = document.createElement('div');
  toast.className = `toast toast-${type}`;
  const icons = { success: '✅', error: '❌', warning: '⚠️', info: 'ℹ️' };
  toast.innerHTML = `<span class="toast-icon">${icons[type] || 'ℹ️'}</span><span class="toast-msg">${escapeHtml(message)}</span>`;
  container.appendChild(toast);
  requestAnimationFrame(() => toast.classList.add('show'));
  setTimeout(() => {
    toast.classList.remove('show');
    setTimeout(() => toast.remove(), 300);
  }, duration);
}

/**
 * HTML 转义
 */
export function escapeHtml(str) {
  const div = document.createElement('div');
  div.textContent = str;
  return div.innerHTML;
}

/**
 * 全局加载遮罩
 */
export function setLoading(operation) {
  const loader = document.getElementById('global-loader');
  if (operation) {
    loader.style.display = 'flex';
    document.getElementById('loader-text').textContent = operation;
  } else {
    loader.style.display = 'none';
  }
}

/**
 * 格式化错误信息
 */
export function formatError(context, err) {
  try {
    const parsed = JSON.parse(err);
    return `${context}失败：${parsed.message || err}`;
  } catch {
    return `${context}失败：${err}`;
  }
}
