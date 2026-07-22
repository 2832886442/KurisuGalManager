import { useState, useEffect, useCallback } from 'react';
import { invoke, convertFileSrc } from '@tauri-apps/api/core';
import { useGame } from '../hooks/useGameData';
import { escapeHtml, formatError } from '../utils';
import { showToast } from './Toast';
import Modal from './Modal';
import Icon from './Icon';

export default function DetailModal({ gameId, onClose, onEdit, onDelete, onRefresh }) {
  const { state, dispatch } = useGame();
  const game = state.games.find(g => g.id === gameId);
  const [screenshots, setScreenshots] = useState([]);
  const [previewScreenshot, setPreviewScreenshot] = useState(null);
  const isRunning = state.runningIds.includes(gameId);

  const openPreview = async (filename) => {
    setPreviewScreenshot({ filename, loading: true });
    try {
      const path = await invoke('get_screenshot_path', { gameId, filename });
      if (import.meta.env.DEV) {
        const isThumb = path.includes('_thumb');
        console.log(
          `[截图诊断-前端] openPreview 原图 | filename=${filename} | path=${path} | 是缩略图=${isThumb}`,
        );
      }
      setPreviewScreenshot({ filename, url: convertFileSrc(path), loading: false });
    } catch { setPreviewScreenshot(null); }
  };

  const loadScreenshots = useCallback(async () => {
    try {
      const items = await invoke('list_screenshots_with_thumbs', { gameId });
      const mapped = items.map(item => ({
        filename: item.filename,
        thumbUrl: item.thumb_data_uri,
      }));
      if (import.meta.env.DEV) {
        console.log(
          `[截图诊断-前端] loadScreenshots(Base64) | gameId=${gameId} | 共 ${mapped.length} 张截图`,
        );
        mapped.forEach((s, i) => {
          const isDataUri = s.thumbUrl.startsWith('data:image/');
          console.log(
            `[截图诊断-前端] [${i + 1}/${mapped.length}] filename=${s.filename} | dataUri预览=${s.thumbUrl.substring(0, 50)}... | 是DataUri=${isDataUri}`,
          );
        });
      }
      setScreenshots(mapped);
    } catch (e) { console.warn('截图加载失败:', e); }
  }, [gameId]);

  useEffect(() => { if (game) loadScreenshots(); }, [game, loadScreenshots]);

  if (!game) return null;

  const handleLaunch = async () => {
    try {
      dispatch({ type: 'SET_LOADING', payload: '正在启动游戏...' });
      await invoke('launch_game', { gameId: game.id, path: game.path });
      dispatch({ type: 'ADD_RUNNING', payload: game.id });
      onClose();
    } catch (err) {
      showToast('error', formatError('启动游戏', err));
    } finally {
      dispatch({ type: 'SET_LOADING', payload: null });
    }
  };

  const handleQuickStatus = async (status) => {
    try {
      await invoke('quick_update_status', { gameId: game.id, status });
      await onRefresh();
      onClose();
    } catch (err) {
      showToast('error', formatError('更新状态', err));
    }
  };

  const handleToggleFav = async () => {
    try {
      const newState = await invoke('toggle_favorite', { gameId: game.id });
      const updated = state.games.map(g => g.id === game.id ? { ...g, favorite: newState } : g);
      dispatch({ type: 'SET_GAMES', payload: updated });
    } catch (e) {
      showToast('error', formatError('收藏操作', e));
    }
  };

  const handleAddScreenshot = async () => {
    try {
      const selected = await invoke('open_file_dialog', {
        title: '选择截图图片',
        extensions: ['png', 'jpg', 'jpeg', 'gif', 'webp', 'bmp']
      });
      if (!selected) return;
      dispatch({ type: 'SET_LOADING', payload: '添加截图...' });
      await invoke('add_screenshot', { gameId: game.id, sourcePath: selected });
      await onRefresh();
      loadScreenshots();
      showToast('success', '截图已添加');
    } catch (err) {
      showToast('error', formatError('添加截图', err));
    } finally {
      dispatch({ type: 'SET_LOADING', payload: null });
    }
  };

  const handleDeleteScreenshot = async (filename) => {
    if (!confirm('确定删除这张截图?')) return;
    try {
      await invoke('delete_screenshot', { gameId, filename });
      await onRefresh();
      loadScreenshots();
    } catch (e) {
      showToast('error', formatError('删除截图', e));
    }
  };

  const coverSrc = game.cover && game.cover.startsWith('data:') ? game.cover : '';

  return (
    <>
      <Modal onClose={onClose} size="lg" className="detail-modal">
        <div className="detail-container">
          <div className="detail-header">
            <div className="detail-cover">
              {coverSrc ? (
                <img src={coverSrc} alt={escapeHtml(game.name)} />
              ) : (
                <Icon name="gamepad-2" size={36} />
              )}
            </div>
            <div className="detail-info">
              <h2>
                <span>{escapeHtml(game.name)}</span>
                {isRunning && <span className="running-tag">(游戏中)</span>}
                <button
                  onClick={handleToggleFav}
                  title={game.favorite ? '取消收藏' : '添加收藏'}
                  className={`fav-btn${game.favorite ? ' active' : ''}`}
                  aria-label={game.favorite ? '取消收藏' : '添加收藏'}
                >
                  <Icon name="star" size={18} />
                </button>
              </h2>
              {game.alias && <p className="detail-alias">别名:{escapeHtml(game.alias)}</p>}
              <p className="detail-category">分类:{escapeHtml(game.category)}</p>
              <div className="detail-tags">
                {game.tags.map(t => <span key={t} className="detail-tag">{escapeHtml(t)}</span>)}
              </div>
              <p className="detail-status">状态:<span>{isRunning ? '游戏中' : escapeHtml(game.status)}</span></p>
              <p className="detail-playtime">游玩时长:{game.play_time} 分钟</p>
              <p className="detail-lastplay">上次游玩:{game.last_play || '从未'}</p>
              <div className="detail-actions">
                {isRunning ? (
                  <button className="btn btn-primary" disabled>
                    <Icon name="play" size={14} /> 游戏运行中
                  </button>
                ) : (
                  <button className="btn btn-primary" onClick={handleLaunch}>
                    <Icon name="rocket" size={14} /> 启动游戏
                  </button>
                )}
                <button className="btn btn-secondary" onClick={() => onEdit(game.id)}>
                  <Icon name="edit" size={14} /> 编辑
                </button>
                <button className="btn btn-danger" onClick={() => { if (confirm(`确定要删除游戏"${game.name}"吗?`)) onDelete(game.id); }}>
                  <Icon name="trash-2" size={14} /> 删除
                </button>
              </div>
              {!isRunning && (
                <div className="detail-status-quick">
                  <span>快速切换状态:</span>
                  <button className="status-quick-btn qs-playing" onClick={() => handleQuickStatus('游玩中')}>游玩中</button>
                  <button className="status-quick-btn qs-completed" onClick={() => handleQuickStatus('已通关')}>已通关</button>
                  <button className="status-quick-btn qs-shelved" onClick={() => handleQuickStatus('搁置')}>搁置</button>
                </div>
              )}
            </div>
          </div>
          <div className="detail-body">
            <h4>简介</h4>
            <p>{escapeHtml(game.description || '暂无简介')}</p>
            <h4>启动路径</h4>
            <code>{escapeHtml(game.path)}</code>
            <h4>游戏截图</h4>
            <div className="screenshots-section">
              <div className="screenshots-grid">
                {screenshots.length === 0 ? (
                  <span className="hint" style={{ padding: '12px 0', display: 'block' }}>暂无截图,点击下方按钮添加</span>
                ) : (
                  screenshots.map(s => (
                    <div
                      key={s.filename}
                      className="screenshot-thumb"
                      onClick={() => openPreview(s.filename)}
                      title="点击查看大图"
                    >
                      <img src={s.thumbUrl} alt="截图" loading="lazy" decoding="async" />
                      <button className="screenshot-del-btn" onClick={(e) => { e.stopPropagation(); handleDeleteScreenshot(s.filename); }} title="删除">
                        <Icon name="x" size={12} color="#fff" />
                      </button>
                    </div>
                  ))
                )}
              </div>
              <button className="btn btn-secondary btn-sm" onClick={handleAddScreenshot}>
                <Icon name="image" size={14} /> 添加截图
              </button>
            </div>
          </div>
        </div>
      </Modal>
      {previewScreenshot && (
        <div className="modal-overlay screenshot-preview-overlay" onClick={() => setPreviewScreenshot(null)}>
          <div className="screenshot-preview-container" onClick={(e) => e.stopPropagation()}>
            <button className="screenshot-preview-close" onClick={() => setPreviewScreenshot(null)} title="关闭">
              <Icon name="x" size={20} />
            </button>
            <img
              className="screenshot-preview-img"
              src={previewScreenshot.url || ''}
              alt="截图预览"
            />
            {previewScreenshot.loading && <span className="screenshot-preview-loading">加载原图...</span>}
            <span className="screenshot-preview-name">{previewScreenshot.filename}</span>
          </div>
        </div>
      )}
    </>
  );
}
