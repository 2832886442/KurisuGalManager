import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { useGame } from '../hooks/useGameData';
import { showToast } from './Toast';
import { formatError } from '../utils';
import Icon from './Icon';

export default function SettingsPage() {
  const { state, dispatch } = useGame();

  const [theme, setTheme] = useState(localStorage.getItem('theme') || 'system');
  const [radius, setRadius] = useState(localStorage.getItem('window-radius') || '14');
  const [zoom, setZoom] = useState(localStorage.getItem('zoom') || '1.0');
  const [startup, setStartup] = useState(localStorage.getItem('startup') === 'true');
  const [closeAction, setCloseAction] = useState(localStorage.getItem('close-action') || 'tray');
  const [defaultView, setDefaultView] = useState(localStorage.getItem('default-view') || 'grid');
  const [pageSize, setPageSizeState] = useState(localStorage.getItem('pageSize') || '24');
  const [dataRoot, setDataRoot] = useState('');
  const [dataInfo, setDataInfo] = useState('加载中...');
  const [showMigrate, setShowMigrate] = useState(false);
  const [pendingPath, setPendingPath] = useState('');
  const [oldPath, setOldPath] = useState('');
  const [migrateStats, setMigrateStats] = useState(null);

  useEffect(() => {
    loadDataRootInfo();
  }, []);

  const loadDataRootInfo = async () => {
    try {
      const info = await invoke('get_data_size_info');
      setDataRoot(info.data_root);
      setDataInfo(`文件数: ${info.file_count} | 封面: ${info.cover_count} | 大小: ${parseFloat(info.total_mb).toFixed(1)} MB`);
    } catch (e) {
      setDataRoot('加载失败');
      setDataInfo('无法获取数据目录信息');
    }
  };

  const applyTheme = (newTheme) => {
    document.documentElement.classList.remove('dark', 'light', 'neon', 'glass', 'nexus', 'system');
    if (newTheme !== 'system') {
      document.documentElement.classList.add(newTheme);
      localStorage.setItem('theme', newTheme);
    } else {
      localStorage.removeItem('theme');
      const isDark = window.matchMedia('(prefers-color-scheme: dark)').matches;
      document.documentElement.classList.add(isDark ? 'dark' : 'light');
    }
    setTheme(newTheme);
  };

  const applyRadius = (newRadius) => {
    setRadius(newRadius);
    localStorage.setItem('window-radius', newRadius);
    document.documentElement.style.setProperty('--radius', newRadius + 'px');
  };

  const applyZoom = (newZoom) => {
    setZoom(newZoom);
    localStorage.setItem('zoom', newZoom);
  };

  const applyStartup = (newStartup) => {
    setStartup(newStartup);
    localStorage.setItem('startup', newStartup ? 'true' : 'false');
    invoke('set_startup', { enabled: newStartup }).catch(() => { });
  };

  const applyCloseAction = (newAction) => {
    setCloseAction(newAction);
    localStorage.setItem('close-action', newAction);
    invoke('save_settings', { settings: { closeAction: newAction } }).catch(() => { });
  };

  const applyDefaultView = (newView) => {
    setDefaultView(newView);
    localStorage.setItem('default-view', newView);
    invoke('save_settings', { settings: { defaultView: newView } }).catch(() => { });
  };

  const applyPageSize = (newSize) => {
    setPageSizeState(newSize);
    localStorage.setItem('pageSize', newSize);
    if (String(state.pageSize) !== newSize) {
      dispatch({ type: 'SET_PAGE_SIZE', payload: parseInt(newSize) });
    }
  };

  const handleBackup = async () => {
    try {
      dispatch({ type: 'SET_LOADING', payload: '正在备份数据...' });
      const path = await invoke('backup_data');
      showToast('success', `备份成功:${path}`);
    } catch (e) { showToast('error', formatError('备份', e)); }
    finally { dispatch({ type: 'SET_LOADING', payload: null }); }
  };

  const handleRestore = async () => {
    try {
      const filePath = await invoke('open_file_dialog', { title: '选择备份文件', extensions: ['json'] });
      if (!filePath) return;
      dispatch({ type: 'SET_LOADING', payload: '正在恢复数据...' });
      await invoke('restore_data', { filePath });
      showToast('success', '数据恢复成功');
      loadDataRootInfo();
    } catch (e) { showToast('error', formatError('恢复', e)); }
    finally { dispatch({ type: 'SET_LOADING', payload: null }); }
  };

  const handleCleanup = async () => {
    if (!confirm('此操作将删除所有路径无效的游戏条目,确定继续?')) return;
    try {
      dispatch({ type: 'SET_LOADING', payload: '正在清理无效数据...' });
      const data = await invoke('cleanup_invalid');
      dispatch({ type: 'SET_GAMES', payload: data.games || [] });
      showToast('success', `清理完成,当前共有 ${data.games.length} 款游戏`);
    } catch (e) { showToast('error', formatError('清理', e)); }
    finally { dispatch({ type: 'SET_LOADING', payload: null }); }
  };

  const handleBrowseDataPath = async () => {
    try {
      const folder = await invoke('pick_folder', { title: '选择新的数据存储目录' });
      if (!folder) return;
      const info = await invoke('get_data_size_info');
      setOldPath(info.data_root);
      setPendingPath(folder);
      setMigrateStats(info);
      setShowMigrate(true);
    } catch (e) {
      showToast('error', '获取路径信息失败: ' + formatError('', e));
    }
  };

  const handleExecuteMigrate = async () => {
    if (!pendingPath) return;
    try {
      dispatch({ type: 'SET_LOADING', payload: '正在迁移数据...' });
      await invoke('set_data_root', { path: pendingPath, migrate: true });
      showToast('success', '数据迁移完成,路径已更新');
      setShowMigrate(false);
      loadDataRootInfo();
    } catch (e) {
      showToast('error', formatError('迁移', e));
    } finally {
      dispatch({ type: 'SET_LOADING', payload: null });
    }
  };

  const themeOptions = [
    { value: 'dark', label: '暗色', icon: 'moon' },
    { value: 'light', label: '亮色', icon: 'sun' },
    { value: 'neon', label: 'Neon', icon: 'rocket' },
    { value: 'glass', label: 'Glass', icon: 'image' },
    { value: 'nexus', label: 'Nexus', icon: 'sparkles' },
    { value: 'system', label: '跟随系统', icon: 'monitor' },
  ];

  return (
    <div className="settings-page">
      <div className="settings-header">
        <h1 className="settings-title">设置</h1>
        <p className="settings-subtitle">配置应用的各项参数</p>
      </div>

      <div className="settings-content">
        <div className="settings-section">
          <h2 className="settings-section-title">
            <Icon name="palette" size={16} /> 外观
          </h2>

          <div className="settings-item">
            <label className="settings-label">主题</label>
            <div className="settings-options">
              {themeOptions.map(t => (
                <button
                  key={t.value}
                  className={`settings-option-btn${theme === t.value ? ' active' : ''}`}
                  onClick={() => applyTheme(t.value)}
                >
                  <Icon name={t.icon} size={14} />
                  <span>{t.label}</span>
                </button>
              ))}
            </div>
          </div>

          <div className="settings-item">
            <label className="settings-label">窗口圆角</label>
            <div className="settings-control-row">
              <input
                type="range"
                min="0"
                max="20"
                value={radius}
                onChange={(e) => applyRadius(e.target.value)}
                className="settings-range"
              />
              <span className="settings-hint">{radius}px</span>
            </div>
          </div>

          <div className="settings-item">
            <label className="settings-label">界面缩放</label>
            <select
              value={zoom}
              onChange={(e) => applyZoom(e.target.value)}
              className="settings-select"
            >
              <option value="0.9">90%</option>
              <option value="1.0">100%</option>
              <option value="1.1">110%</option>
              <option value="1.2">120%</option>
            </select>
          </div>

          <div className="settings-item">
            <label className="settings-label">每页显示</label>
            <select
              value={pageSize}
              onChange={(e) => applyPageSize(e.target.value)}
              className="settings-select"
            >
              <option value="12">12 个</option>
              <option value="24">24 个</option>
              <option value="48">48 个</option>
              <option value="96">96 个</option>
            </select>
          </div>
        </div>

        <div className="settings-section">
          <h2 className="settings-section-title">
            <Icon name="database" size={16} /> 数据管理
          </h2>

          <div className="settings-item">
            <label className="settings-label">数据存储路径</label>
            <div className="settings-path-row">
              <span className="settings-path-text">{dataRoot}</span>
              <button
                type="button"
                className="btn btn-secondary btn-sm"
                onClick={handleBrowseDataPath}
                title="选择新路径"
              >
                <Icon name="folder-open" size={14} /> 浏览
              </button>
            </div>
          </div>

          <div className="settings-item">
            <span className="settings-hint">{dataInfo}</span>
          </div>

          <div className="settings-item">
            <button className="btn btn-secondary" onClick={handleBackup}>
              <Icon name="upload" size={14} /> 备份数据
            </button>
            <button className="btn btn-secondary" onClick={handleRestore}>
              <Icon name="download" size={14} /> 恢复数据
            </button>
          </div>

          <div className="settings-item">
            <button className="btn btn-danger" onClick={handleCleanup}>
              <Icon name="trash-2" size={14} /> 清理无效数据
            </button>
            <span className="settings-hint">扫描失效路径、空分类</span>
          </div>
        </div>

        <div className="settings-section">
          <h2 className="settings-section-title">
            <Icon name="rocket" size={16} /> 启动选项
          </h2>

          <div className="settings-item">
            <label className="settings-label">开机自启</label>
            <label className="toggle">
              <input
                type="checkbox"
                checked={startup}
                onChange={(e) => applyStartup(e.target.checked)}
              />
              <span className="toggle-track"><span className="toggle-thumb"></span></span>
            </label>
          </div>

          <div className="settings-item">
            <label className="settings-label">关闭窗口时</label>
            <select
              value={closeAction}
              onChange={(e) => applyCloseAction(e.target.value)}
              className="settings-select"
            >
              <option value="exit">退出程序</option>
              <option value="tray">最小化到托盘</option>
            </select>
          </div>

          <div className="settings-item">
            <label className="settings-label">默认启动视图</label>
            <select
              value={defaultView}
              onChange={(e) => applyDefaultView(e.target.value)}
              className="settings-select"
            >
              <option value="grid">网格视图</option>
              <option value="list">列表视图</option>
            </select>
          </div>
        </div>

        <div className="settings-section settings-about-link">
          <h2 className="settings-section-title">
            <Icon name="info" size={16} /> 关于
          </h2>
          <div className="settings-about-hint">
            <Icon name="info" size={14} />
            <span>项目信息已移至「关于我们」页面,可从左侧导航栏进入。</span>
          </div>
        </div>
      </div>

      {showMigrate && (
        <div className="modal-overlay" onClick={() => setShowMigrate(false)}>
          <div className="modal" onClick={(e) => e.stopPropagation()}>
            <div className="modal-header">
              <Icon name="alert-triangle" size={18} />
              <h3>更改数据存储路径</h3>
              <button className="modal-close" onClick={() => setShowMigrate(false)}>
                <Icon name="x" size={16} />
              </button>
            </div>
            <div className="confirm-body">
              <div className="confirm-section">
                <h4>风险提示</h4>
                <ul>
                  <li>更改路径后,系统将自动迁移所有游戏数据和封面文件</li>
                  <li>迁移过程中请勿关闭程序,否则可能导致数据丢失</li>
                  <li>建议在迁移前先进行一次数据备份</li>
                </ul>
              </div>
              <div className="confirm-section confirm-paths">
                <div className="confirm-path-row">
                  <span className="confirm-path-label">当前路径</span>
                  <span className="confirm-path-value">{oldPath}</span>
                </div>
                <div className="confirm-path-arrow">
                  <Icon name="arrow-up-down" size={14} />
                </div>
                <div className="confirm-path-row">
                  <span className="confirm-path-label">新路径</span>
                  <span className="confirm-path-value">{pendingPath}</span>
                </div>
              </div>
              {migrateStats && (
                <div className="confirm-section confirm-stats">
                  <span className="confirm-stat">文件: {migrateStats.file_count}</span>
                  <span className="confirm-stat">封面: {migrateStats.cover_count}</span>
                  <span className="confirm-stat">大小: {parseFloat(migrateStats.total_mb).toFixed(1)} MB</span>
                </div>
              )}
            </div>
            <div className="confirm-footer">
              <button className="btn btn-secondary" onClick={() => setShowMigrate(false)}>取消</button>
              <button className="btn btn-primary" onClick={handleExecuteMigrate}>
                <Icon name="check" size={14} /> 确认迁移
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
