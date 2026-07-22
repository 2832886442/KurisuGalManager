import { useGame } from '../hooks/useGameData';
import { escapeHtml } from '../utils';
import { invoke } from '@tauri-apps/api/core';
import Icon from './Icon';
import { useState, useEffect, useRef } from 'react';

const STATUS_CLASS = {
  '未游玩': 'status-unplayed',
  '游玩中': 'status-playing',
  '已通关': 'status-completed',
  '搁置': 'status-shelved',
};

function useTagTruncation(tags) {
  const [visibleCount, setVisibleCount] = useState(tags.length);
  const tagsRef = useRef(null);

  useEffect(() => {
    if (!tagsRef.current || tags.length === 0) {
      setVisibleCount(tags.length);
      return;
    }

    const container = tagsRef.current;
    const gap = 4;
    const maxHeight = 46; // 2 lines max

    // Helper: create a styled tag span
    const makeTag = (text) => {
      const s = document.createElement('span');
      s.textContent = text;
      s.style.display = 'inline-block';
      s.style.fontSize = '11px';
      s.style.lineHeight = '1.5';
      s.style.padding = '1px 6px';
      s.style.border = '1px solid transparent';
      s.style.borderRadius = '4px';
      s.style.boxSizing = 'border-box';
      s.style.whiteSpace = 'nowrap';
      return s;
    };

    // Helper: measure temp container height
    const measure = (children) => {
      const tc = document.createElement('div');
      tc.style.visibility = 'hidden';
      tc.style.position = 'absolute';
      tc.style.display = 'flex';
      tc.style.flexWrap = 'wrap';
      tc.style.gap = gap + 'px';
      tc.style.fontSize = '11px';
      tc.style.lineHeight = '1.5';
      tc.style.width = container.offsetWidth + 'px';
      children.forEach(c => tc.appendChild(c.cloneNode(true)));
      document.body.appendChild(tc);
      const h = tc.getBoundingClientRect().height;
      document.body.removeChild(tc);
      return h;
    };

    const checkOverflow = () => {
      // Step 1: how many full tags fit?
      let fitCount = tags.length;
      for (let i = 0; i < tags.length; i++) {
        const el = makeTag(tags[i]);
        const testTags = tags.slice(0, i + 1).map(t => makeTag(t));
        if (measure(testTags) > maxHeight) {
          fitCount = i;
          break;
        }
      }

      // Step 2: if not all tags fit, make room for +N badge
      if (fitCount < tags.length && fitCount > 0) {
        const hidden = tags.length - fitCount;
        // Try removing tags one by one and replacing with +N badge
        for (let attempt = fitCount; attempt >= 1; attempt--) {
          const children = tags.slice(0, attempt).map(t => makeTag(t));
          children.push(makeTag(`+${tags.length - attempt}`));
          if (measure(children) <= maxHeight) {
            fitCount = attempt;
            break;
          }
        }
        // if even 1 tag + +N doesn't fit, show just +N
        if (fitCount >= tags.length) {
          const plusOnly = [makeTag(`+${tags.length}`)];
          if (measure(plusOnly) <= maxHeight) {
            fitCount = 0;
          }
        }
      }

      setVisibleCount(fitCount);
    };

    checkOverflow();

    const observer = new ResizeObserver(checkOverflow);
    observer.observe(container);
    return () => observer.disconnect();
  }, [tags]);

  return { visibleCount, tagsRef };
}

export default function GameCard({ game }) {
  const { state, dispatch } = useGame();
  const isRunning = state.runningIds.includes(game.id);
  const isSelected = state.selectedIds.includes(game.id);
  const isBatch = state.batchMode;

  const displayStatus = isRunning ? '游戏中' : game.status;
  const statusClass = isRunning ? 'status-running' : (STATUS_CLASS[game.status] || '');
  const hasCover = game.cover && game.cover.startsWith('data:');
  const { visibleCount, tagsRef } = useTagTruncation(game.tags);

  const handleClick = () => { if (isBatch) dispatch({ type: 'TOGGLE_SELECT', payload: game.id }); };

  const handleLaunch = async (e) => {
    e.stopPropagation();
    if (isBatch || isRunning) return;
    try {
      dispatch({ type: 'SET_LOADING', payload: '正在启动...' });
      await invoke('launch_game', { gameId: game.id, path: game.path });
      dispatch({ type: 'ADD_RUNNING', payload: game.id });
    } catch (err) { console.warn('启动失败:', err); }
    finally { dispatch({ type: 'SET_LOADING', payload: null }); }
  };

  const handleQuickStatus = async (e, status) => {
    e.stopPropagation();
    if (isBatch || isRunning) return;
    try {
      const data = await invoke('quick_update_status', { gameId: game.id, status });
      const updatedGames = data.games || [];
      dispatch({ type: 'SET_GAMES', payload: updatedGames });
      const ids = updatedGames.filter(g => g.cover && !g.cover.startsWith('data:')).map(g => g.id);
      if (ids.length > 0) {
        try {
          const covers = await invoke('get_game_covers', { gameIds: ids });
          const gamesWithCovers = updatedGames.map(g => { if (covers[g.id]) return { ...g, cover: covers[g.id] }; return g; });
          dispatch({ type: 'SET_GAMES', payload: gamesWithCovers });
        } catch (e) { console.warn('封面加载失败:', e); }
      }
    } catch (err) { console.warn('状态更新失败:', err); }
  };

  const handleDblClick = (e) => {
    if (isBatch || isRunning) return;
    e.stopPropagation();
    invoke('launch_game', { gameId: game.id, path: game.path }).then(() => {
      dispatch({ type: 'ADD_RUNNING', payload: game.id });
    }).catch(() => { });
  };

  return (
    <div
      className={`game-card${isBatch ? ' batch-mode' : ''}${isSelected ? ' selected' : ''}`}
      onClick={handleClick}
      onDoubleClick={handleDblClick}
    >
      <div className="card-cover">
        {hasCover ? (
          <img src={game.cover} alt={escapeHtml(game.name)} loading="lazy" />
        ) : (
          <div className="no-cover"><Icon name="gamepad-2" size={28} /></div>
        )}

        <input
          type="checkbox"
          className="card-checkbox"
          checked={isSelected}
          onChange={(e) => { e.stopPropagation(); dispatch({ type: 'TOGGLE_SELECT', payload: game.id }); }}
        />

        {game.favorite && !isBatch && (
          <span className="card-fav-badge" title="收藏">
            <Icon name="star" size={14} color="#fbbf24" />
          </span>
        )}

        <span className={`card-status-badge ${statusClass}`}>{escapeHtml(displayStatus)}</span>

        {isRunning ? (
          <div className="card-play-overlay"><span className="play-indicator"></span></div>
        ) : !isBatch && (
          <>
            <button className="card-launch-btn" onClick={handleLaunch} title="启动游戏">
              <Icon name="play" size={20} color="#fff" />
            </button>
            <div className="status-quick-switch">
              <button className="status-quick-btn qs-playing" onClick={(e) => handleQuickStatus(e, '游玩中')}>游玩中</button>
              <button className="status-quick-btn qs-completed" onClick={(e) => handleQuickStatus(e, '已通关')}>通关</button>
              <button className="status-quick-btn qs-shelved" onClick={(e) => handleQuickStatus(e, '搁置')}>搁置</button>
            </div>
          </>
        )}
      </div>

      <div className="card-body">
        <h4 className="card-title">{escapeHtml(game.name)}</h4>
        {game.alias && <p className="card-alias">{escapeHtml(game.alias)}</p>}
        <div className="card-tags" ref={tagsRef}>
          {game.tags.slice(0, visibleCount).map(t => <span key={t} className="card-tag">{escapeHtml(t)}</span>)}
          {game.tags.length > visibleCount && <span className="card-tag">+{game.tags.length - visibleCount}</span>}
        </div>
        {game.description && <p className="card-intro">{escapeHtml(game.description)}</p>}
        <span className="card-category">{escapeHtml(game.category)}</span>
      </div>
      <div className="card-meta">
        <span className="card-meta-last">{game.last_play ? escapeHtml(game.last_play) : '从未'}</span>
        <span className="card-meta-time">{game.play_time > 0 ? game.play_time + 'min' : '0min'}</span>
      </div>
    </div>
  );
}
