import { useGame } from '../hooks/useGameData';
import { invoke } from '@tauri-apps/api/core';
import { useState, useEffect } from 'react';
import Icon from './Icon';

export default function Sidebar({ onAddGame, onManageCats, onSettings, onRefresh }) {
  const { state, dispatch } = useGame();
  const [logoUrl, setLogoUrl] = useState('');

  useEffect(() => {
    invoke('get_logo').then(uri => { if (uri) setLogoUrl(uri); }).catch(() => { });
  }, []);

  const handleLogoClick = async () => {
    try {
      const selected = await invoke('open_file_dialog', { title: '选择 Logo 图片', extensions: ['png', 'jpg', 'jpeg', 'gif', 'webp'] });
      if (!selected) return;
      const dataUri = await invoke('copy_cover', { sourcePath: selected, gameId: '__logo__' });
      await invoke('save_logo', { dataUri });
      setLogoUrl(dataUri);
    } catch (e) { console.warn('Logo 更新失败:', e); }
  };

  const handleNav = (nav, filters) => {
    if (nav === 'ranking') {
      dispatch({ type: 'SET_ACTIVE_NAV', payload: 'ranking' });
    } else if (nav === 'settings') {
      dispatch({ type: 'SET_ACTIVE_NAV', payload: 'settings' });
    } else if (nav === 'welcome' || nav === 'overview' || nav === 'about') {
      dispatch({ type: 'SET_ACTIVE_NAV', payload: nav });
    } else {
      dispatch({ type: 'SET_ACTIVE_NAV', payload: nav === 'favorites' ? 'favorites' : 'games' });
      if (filters) dispatch({ type: 'SET_FILTER', payload: filters });
    }
    dispatch({ type: 'EXIT_BATCH' });
  };

  return (
    <nav className="sidebar">
      <div className="sidebar-logo" onClick={handleLogoClick} title="点击更换 Logo">
        {logoUrl ? <img src={logoUrl} alt="Logo" /> : <Icon name="gamepad-2" size={22} />}
      </div>

      {/* 1. 欢迎 */}
      <button
        className={`sidebar-btn${state.activeNav === 'welcome' ? ' active' : ''}`}
        data-nav="welcome"
        title="欢迎"
        onClick={() => handleNav('welcome')}
      >
        <Icon name="sparkles" size={20} />
      </button>

      {/* 2. 数据概览 */}
      <button
        className={`sidebar-btn${state.activeNav === 'overview' ? ' active' : ''}`}
        data-nav="overview"
        title="数据概览"
        onClick={() => handleNav('overview')}
      >
        <Icon name="home" size={20} />
      </button>

      {/* 3. 关于我们 */}
      <button
        className={`sidebar-btn${state.activeNav === 'about' ? ' active' : ''}`}
        data-nav="about"
        title="关于我们"
        onClick={() => handleNav('about')}
      >
        <Icon name="info" size={20} />
      </button>

      <span className="sidebar-divider" aria-hidden="true" />

      {/* 4. 所有游戏 */}
      <button
        className={`sidebar-btn${state.activeNav === 'games' && !state.filters.favorites ? ' active' : ''}`}
        data-nav="all"
        title="所有游戏"
        onClick={() => handleNav('all', { category: 'all', tag: null, favorites: false })}
      >
        <Icon name="library" size={20} />
      </button>

      {/* 5. 收藏夹 */}
      <button
        className={`sidebar-btn${state.activeNav === 'favorites' ? ' active' : ''}`}
        data-nav="favorites"
        title="收藏夹"
        onClick={() => handleNav('favorites', { favorites: true, category: 'all', tag: null })}
      >
        <Icon name="star" size={20} />
      </button>

      {/* 6. 排名 */}
      <button
        className={`sidebar-btn${state.activeNav === 'ranking' ? ' active' : ''}`}
        data-nav="ranking"
        title="排名"
        onClick={() => handleNav('ranking', {})}
      >
        <Icon name="trophy" size={20} />
      </button>

      <span className="sidebar-spacer"></span>

      <button className="sidebar-btn" title="添加游戏" onClick={onAddGame}>
        <Icon name="plus-circle" size={20} />
      </button>
      <button className="sidebar-btn" title="管理分类" onClick={onManageCats}>
        <Icon name="folder-tree" size={20} />
      </button>
      <button
        className={`sidebar-btn${state.activeNav === 'settings' ? ' active' : ''}`}
        title="设置"
        onClick={() => handleNav('settings', {})}
      >
        <Icon name="settings" size={20} />
      </button>
    </nav>
  );
}
