import { useState, useEffect, useRef } from 'react';
import Icon from './Icon';

export default function FloatToolbar({ onAddGame, onRefresh }) {
  const [isLight, setIsLight] = useState(() => document.documentElement.classList.contains('light'));
  const [refreshing, setRefreshing] = useState(false);
  const [visible, setVisible] = useState(false);
  const [themeFixed, setThemeFixed] = useState(false);
  const toolbarRef = useRef(null);

  useEffect(() => {
    const observer = new MutationObserver(() => {
      const html = document.documentElement;
      setIsLight(html.classList.contains('light'));
      setThemeFixed(html.classList.contains('neon') || html.classList.contains('glass') || html.classList.contains('nexus'));
    });
    observer.observe(document.documentElement, { attributes: true, attributeFilter: ['class'] });
    const html = document.documentElement;
    setThemeFixed(html.classList.contains('neon') || html.classList.contains('glass') || html.classList.contains('nexus'));
    return () => observer.disconnect();
  }, []);

  useEffect(() => {
    const handleMouseMove = (e) => {
      if (toolbarRef.current && toolbarRef.current.contains(e.target)) {
        setVisible(true);
        return;
      }

      const grid = document.querySelector('.game-grid');
      if (!grid) {
        setVisible(false);
        return;
      }

      const rect = grid.getBoundingClientRect();
      const isInGrid = e.clientX >= rect.left && e.clientX <= rect.right &&
        e.clientY >= rect.top && e.clientY <= rect.bottom;

      if (isInGrid) {
        const distanceFromTop = e.clientY - rect.top;
        setVisible(distanceFromTop <= 60);
      } else {
        setVisible(false);
      }
    };

    document.addEventListener('mousemove', handleMouseMove);
    return () => document.removeEventListener('mousemove', handleMouseMove);
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
    <div
      ref={toolbarRef}
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
  );
}
