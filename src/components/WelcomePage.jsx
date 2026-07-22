import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { useGame } from '../hooks/useGameData';
import useAppVersion from '../hooks/useAppVersion';
import Icon from './Icon';

/**
 * WelcomePage - 欢迎页(打开软件最先进入的界面)
 * 简短介绍项目,提供主要功能入口
 */
export default function WelcomePage() {
  const { dispatch, state } = useGame();
  const appVersion = useAppVersion();
  const [stats, setStats] = useState(null);
  const [appIcon, setAppIcon] = useState('');

  useEffect(() => {
    invoke('get_home_stats').then(setStats).catch(() => { });
    invoke('get_app_icon').then(setAppIcon).catch(() => { });
  }, []);

  const totalGames = stats?.total_games ?? state.games.length;
  const totalFavorites = stats?.total_favorites ?? state.games.filter(g => g.favorite).length;

  const features = [
    {
      icon: 'library',
      title: '游戏库管理',
      desc: '集中管理所有 Galgame,支持分类、标签、收藏与多视图浏览',
      nav: 'games',
      navLabel: '进入游戏库',
    },
    {
      icon: 'trophy',
      title: '排名管理',
      desc: '拖拽式分级排名,自定义等级与颜色,打造专属榜单',
      nav: 'ranking',
      navLabel: '管理排名',
    },
    {
      icon: 'image',
      title: 'Bangumi 数据填充',
      desc: '通过 Bangumi API 自动获取封面、简介、标签等元数据',
      nav: 'games',
      navLabel: '添加游戏',
    },
    {
      icon: 'database',
      title: '数据概览',
      desc: '可视化统计游玩时长、分类分布与最近游玩记录',
      nav: 'overview',
      navLabel: '查看概览',
    },
  ];

  const goTo = (nav) => {
    dispatch({ type: 'SET_ACTIVE_NAV', payload: nav });
    if (nav === 'games') dispatch({ type: 'SET_FILTER', payload: { category: 'all', tag: null, favorites: false } });
  };

  return (
    <div className="welcome-page">
      <div className="welcome-inner">
        <section className="welcome-hero">
          <div className="welcome-hero-bg" aria-hidden="true">
            <span className="welcome-orb welcome-orb-1" />
            <span className="welcome-orb welcome-orb-2" />
            <span className="welcome-orb welcome-orb-3" />
          </div>
          <div className="welcome-hero-content">
            <div className="welcome-logo-badge">
              {appIcon ? (
                <img src={appIcon} alt="KurisuGal" width="36" height="36" />
              ) : (
                <Icon name="logo-gamepad" size={36} />
              )}
            </div>
            <h1 className="welcome-title">KurisuGal</h1>
            <p className="welcome-tagline">现代化的 Galgame 游戏库管理器</p>
            <p className="welcome-subtitle">
              基于 Tauri + React + Vue3 + Rust 构建,提供轻量、快速、跨平台的视觉小说管理体验
            </p>
            <div className="welcome-hero-stats">
              <div className="welcome-stat-pill">
                <Icon name="library" size={14} />
                <span>{totalGames} 款游戏</span>
              </div>
              <div className="welcome-stat-pill">
                <Icon name="star" size={14} />
                <span>{totalFavorites} 项收藏</span>
              </div>
            </div>
            <div className="welcome-hero-actions">
              <button className="btn btn-primary welcome-cta" onClick={() => goTo('games')}>
                <Icon name="play" size={14} /> 开始浏览游戏库
              </button>
              <button className="btn btn-secondary welcome-cta" onClick={() => goTo('overview')}>
                <Icon name="database" size={14} /> 查看数据概览
              </button>
            </div>
          </div>
        </section>

        <section className="welcome-features">
          <h2 className="welcome-section-title">核心功能</h2>
          <div className="welcome-feature-grid">
            {features.map((f, i) => (
              <div key={i} className="welcome-feature-card">
                <div className="welcome-feature-icon">
                  <Icon name={f.icon} size={22} />
                </div>
                <h3 className="welcome-feature-title">{f.title}</h3>
                <p className="welcome-feature-desc">{f.desc}</p>
                <button className="welcome-feature-link" onClick={() => goTo(f.nav)}>
                  {f.navLabel}
                  <Icon name="arrow-right" size={12} />
                </button>
              </div>
            ))}
          </div>
        </section>

        <section className="welcome-quicklinks">
          <div className="welcome-quicklinks-card">
            <h3 className="welcome-quicklinks-title">
              <Icon name="sparkles" size={14} /> 快速入口
            </h3>
            <div className="welcome-quicklinks-row">
              <button className="welcome-quicklink" onClick={() => goTo('favorites')}>
                <Icon name="star" size={14} /> 收藏夹
              </button>
              <button className="welcome-quicklink" onClick={() => goTo('ranking')}>
                <Icon name="trophy" size={14} /> 排名管理
              </button>
              <button className="welcome-quicklink" onClick={() => goTo('settings')}>
                <Icon name="settings" size={14} /> 设置
              </button>
              <button className="welcome-quicklink" onClick={() => goTo('about')}>
                <Icon name="info" size={14} /> 关于我们
              </button>
            </div>
          </div>
        </section>

        <footer className="welcome-footer">
          <span>KurisuGal · v{appVersion || '...'}</span>
          <span className="welcome-footer-dot">·</span>
          <span>制作: CoolSomeBody</span>
        </footer>
      </div>
    </div>
  );
}
