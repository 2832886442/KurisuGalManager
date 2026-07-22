import { useState, useEffect, useCallback, useMemo } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { useGame } from '../hooks/useGameData';
import { escapeHtml } from '../utils';
import Icon from './Icon';

function formatPlayTime(minutes) {
  if (!minutes || minutes <= 0) return '0m';
  const h = Math.floor(minutes / 60);
  const m = minutes % 60;
  return h > 0 ? `${h}h ${m}m` : `${m}m`;
}

function formatLastPlay(value) {
  if (!value) return '从未';
  // 兼容已有的字符串/数字时间戳
  if (typeof value === 'number') {
    const d = new Date(value);
    if (isNaN(d.getTime())) return String(value);
    const now = new Date();
    const diffMs = now - d;
    const day = 24 * 60 * 60 * 1000;
    if (diffMs < day) return '今天';
    if (diffMs < 2 * day) return '昨天';
    if (diffMs < 7 * day) return `${Math.floor(diffMs / day)} 天前`;
    return `${d.getMonth() + 1}/${d.getDate()}`;
  }
  return String(value);
}

export default function HomePage({ onShowDetail }) {
  const { state } = useGame();
  const [stats, setStats] = useState(null);

  const loadStats = useCallback(async () => {
    try {
      const data = await invoke('get_home_stats');
      setStats(data);
    } catch (e) {
      console.warn('加载统计数据失败:', e);
    }
  }, []);

  useEffect(() => { loadStats(); }, [loadStats]);

  // 客户端计算最近游玩(基于 state.games 的 last_play 字段)
  const recentPlays = useMemo(() => {
    return state.games
      .filter(g => g.last_play)
      .sort((a, b) => {
        // 统一按数值比较(若 last_play 是日期字符串则转时间戳)
        const ta = typeof a.last_play === 'number' ? a.last_play : new Date(a.last_play).getTime() || 0;
        const tb = typeof b.last_play === 'number' ? b.last_play : new Date(b.last_play).getTime() || 0;
        return tb - ta;
      })
      .slice(0, 6);
  }, [state.games]);

  if (!stats) {
    return (
      <div className="home-page">
        <div className="home-header">
          <h2 className="home-title">数据概览</h2>
        </div>
        <div className="chart-empty">加载中...</div>
      </div>
    );
  }

  const timeStr = formatPlayTime(stats.total_play_time || 0);

  const catDist = stats.category_distribution || [];
  const maxCount = Math.max(...catDist.map(c => c.count), 1);

  const top5 = stats.top5_playtime || [];
  const rankClasses = ['r1', 'r2', 'r3', 'rn', 'rn'];

  const statCards = [
    { icon: 'library', value: stats.total_games || 0, label: '游戏总数' },
    { icon: 'star', value: stats.total_favorites || 0, label: '收藏游戏' },
    { icon: 'clock', value: timeStr, label: '总游玩时长' },
  ];

  return (
    <div className="home-page">
      <div className="home-page-inner">
        <div className="home-header">
          <h2 className="home-title">数据概览</h2>
          <button className="btn btn-secondary btn-sm" onClick={loadStats} title="刷新数据">
            <Icon name="refresh-cw" size={14} /> 刷新
          </button>
        </div>

        <div className="home-stats-row">
          {statCards.map((c, i) => (
            <div key={i} className="home-stat-card">
              <div className="stat-icon"><Icon name={c.icon} size={20} /></div>
              <div className="stat-info">
                <span className="stat-value">{c.value}</span>
                <span className="stat-label">{c.label}</span>
              </div>
            </div>
          ))}
        </div>

        <div className="home-charts">
          <div className="home-chart-panel">
            <h3 className="chart-title"><Icon name="folder-tree" size={13} /> 分类分布</h3>
            <div className="chart-bars">
              {catDist.length === 0 ? (
                <span className="chart-empty">暂无数据</span>
              ) : (
                catDist.map(c => {
                  const pct = Math.round((c.count / maxCount) * 100);
                  return (
                    <div key={c.name} className="chart-bar-row">
                      <span className="chart-bar-label" title={escapeHtml(c.name)}>{escapeHtml(c.name)}</span>
                      <div className="chart-bar-track">
                        <div className="chart-bar-fill" style={{ width: `${pct}%` }}></div>
                      </div>
                      <span className="chart-bar-count">{c.count}</span>
                    </div>
                  );
                })
              )}
            </div>
          </div>

          <div className="home-chart-panel">
            <h3 className="chart-title"><Icon name="trophy" size={13} /> 游玩时长 Top 5</h3>
            <div className="chart-ranking">
              {top5.length === 0 ? (
                <span className="chart-empty">暂无游玩记录</span>
              ) : (
                top5.map((item, i) => (
                  <div key={item.name} className="ranking-item">
                    <span className={`ranking-pos ${rankClasses[i]}`}>{i + 1}</span>
                    <div className="ranking-info">
                      <div className="ranking-name" title={escapeHtml(item.name)}>{escapeHtml(item.name)}</div>
                      <div className="ranking-category">{escapeHtml(item.category || '未分类')}</div>
                    </div>
                    <span className="ranking-time">{formatPlayTime(item.play_time)}</span>
                  </div>
                ))
              )}
            </div>
          </div>
        </div>

        <div className="home-recent">
          <h3 className="chart-title"><Icon name="clock" size={13} /> 最近游玩</h3>
          {recentPlays.length === 0 ? (
            <span className="chart-empty">还没有游玩记录,启动一款游戏后会在此显示</span>
          ) : (
            <div className="recent-list">
              {recentPlays.map(g => {
                const cover = g.cover && g.cover.startsWith('data:') ? g.cover : '';
                return (
                  <div
                    key={g.id}
                    className="recent-item"
                    onClick={() => onShowDetail && onShowDetail(g.id)}
                    title="点击查看详情"
                  >
                    {cover ? (
                      <img className="recent-cover" src={cover} alt="" loading="lazy" />
                    ) : (
                      <div className="recent-cover-fallback"><Icon name="gamepad-2" size={14} /></div>
                    )}
                    <div className="recent-info">
                      <div className="recent-name">{escapeHtml(g.name)}</div>
                      <div className="recent-meta">
                        <Icon name="folder" size={11} /> {escapeHtml(g.category || '未分类')}
                      </div>
                    </div>
                    <span className="recent-time">{formatLastPlay(g.last_play)}</span>
                  </div>
                );
              })}
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
