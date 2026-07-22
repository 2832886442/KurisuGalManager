import React from 'react';
import ReactDOM from 'react-dom/client';
import App from './App';

/* CSS 加载顺序(Qt 风格):
 *   variables.css  —— 设计令牌(颜色/字体/间距/圆角/动画时长)
 *   base.css       —— 重置 + 滚动条 + 选区
 *   fonts.css      —— 字体栈
 *   animations.css —— 关键帧 + 动画工具类
 *   layout.css     —— 三栏布局 + 顶部工具栏 + 悬浮快捷栏 + 分页
 *   components.css —— 按钮/卡片/表单/弹窗/Toast 等组件样式
 *   responsive.css —— 响应式适配
 */
import './css/variables.css';
import './css/base.css';
import './css/fonts.css';
import './css/animations.css';
import './css/layout.css';
import './css/components.css';
import './css/responsive.css';

const initTheme = () => {
  const savedTheme = localStorage.getItem('theme');
  if (savedTheme && savedTheme !== 'system') {
    document.documentElement.classList.add(savedTheme);
  } else {
    const isDark = window.matchMedia('(prefers-color-scheme: dark)').matches;
    document.documentElement.classList.add(isDark ? 'dark' : 'light');
  }
};

initTheme();

ReactDOM.createRoot(document.getElementById('root')).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>
);
