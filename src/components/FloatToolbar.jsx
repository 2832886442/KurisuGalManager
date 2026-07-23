import { useState, useEffect } from 'react';
import Icon from './Icon';

/**
 * 悬浮快捷工具栏(灵动岛风格)
 * 默认隐藏,鼠标移到游戏列表上方区域才显示。
 * 主题切换按钮在不支持亮暗切换的主题(neon/glass/nexus)时禁用。
 */
export default function FloatToolbar({ onAddGame, onRefresh }) {
  const [isLight, setIsLight] = useState(() => document.documentElement.classList.contains('light'));
  const [refreshing, setRefreshing] = useState(false);
  const [visible, setVisible] = useState(false);
  const [themeFixed, setThemeFixed] = useState(false);

  useEffect(() => {
    const observer = new MutationObserver(() => {
      const html = document.documentElement;
      setIsLight(html.classList.contains('light'));
      // neon/glass/nexus 主题不支持亮暗切换
      setThemeFixed(html.classList.contains('neon') || html.classList.contains('glass') || html.classList.contains('nexus'));
    });
    observer.observe(document.documentElement, { attributes: true, attributeFilter: ['class'] });
    // 初始化检查
    const html = document.documentElement;
    setThemeFixed(html.classList.contains('neon') || html.classList.contains('glass') || html.classList.contains('nexus'));
    return () => observer.disconnect();
  }, []);

  const toggleTheme = () => {
    if (themeFixed) return;
    const nextIsLight = !isLight;
    if (nextIsLight) {
      document.documentElement.classList.add('light');
      localStorage.setItem('theme', 'light');
    } else {
      document.documentElement.classList.remove('light');
      localStorage.setItem('theme', 'dark');
    }
    setIsLight(nextIsLight);
  };

  const handleRefresh = async () => {
    if (refreshing) return;
    setRefreshing(true);
    try {
      if (onRefresh) await onRefresh();
    } finally {
      setTimeout(() => setRefreshing(false), 500);
    }
  };

  return (
    <>
      <div
        className="float-toolbar-trigger"
        onMouseEnter={() => setVisible(true)}
        onMouseLeave={() => setVisible(false)}
      />
      <div
        className={`float-toolbar${visible ? ' visible' : ''}`}
        role="toolbar"
        aria-label="快捷操作"
        onMouseEnter={() => setVisible(true)}
        onMouseLeave={() => setVisible(false)}
      >
        <button className="toolbar-btn" title="添加游戏" onClick={onAddGame}>
          <Icon name="plus-circle" size={16} />
        </button>
        <button className="toolbar-btn" title="刷新游戏库" onClick={handleRefresh} disabled={refreshing}>
          <Icon name="refresh-cw" size={16} className={refreshing ? 'spin' : ''} />
        </button>
        <button
          className={`toolbar-btn${themeFixed ? ' disabled' : ''}`}
          title={themeFixed ? '当前主题不支持亮暗切换' : (isLight ? '切换到暗色' : '切换到亮色')}
          onClick={toggleTheme}
          disabled={themeFixed}
        >
          <Icon name={isLight ? 'sun' : 'moon'} size={16} />
        </button>
      </div>
    </>
  );
}
