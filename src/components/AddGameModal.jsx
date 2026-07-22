import { useState, useEffect, useMemo, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { useGame } from '../hooks/useGameData';
import { escapeHtml, formatError } from '../utils';
import { showToast } from './Toast';
import Modal from './Modal';
import Icon from './Icon';

export default function AddGameModal({ editGameId, onClose, onSaved }) {
  const { state, dispatch, getCategories } = useGame();
  const isEdit = !!editGameId;
  const editGame = isEdit ? state.games.find(g => g.id === editGameId) : null;

  const categories = useMemo(() => getCategories(), [getCategories]);

  const [name, setName] = useState('');
  const [alias, setAlias] = useState('');
  const [path, setPath] = useState('');
  const [category, setCategory] = useState('未分类');
  const [tags, setTags] = useState('');
  const [status, setStatus] = useState('未游玩');
  const [cover, setCover] = useState('');
  const [desc, setDesc] = useState('');

  // Dirty tracking for confirm-before-close
  const [dirty, setDirty] = useState(false);

  // Bangumi search state
  const [bgmKeyword, setBgmKeyword] = useState('');
  const [bgmResults, setBgmResults] = useState([]);
  const [bgmLoading, setBgmLoading] = useState(false);
  const [bgmCoverData, setBgmCoverData] = useState('');

  // 用 ref 追踪取消状态,避免闭包陈旧值问题
  const abortRef = useRef(false);
  // 保存搜索时的关键词,供 fetch 回退使用
  const searchKwRef = useRef('');
  // 追踪正在选择的 subjectId,防止重复点击
  const [selectingId, setSelectingId] = useState(null);
  const selectingRef = useRef(null);

  useEffect(() => {
    if (isEdit && editGame) {
      setName(editGame.name || '');
      setAlias(editGame.alias || '');
      setPath(editGame.path || '');
      setCategory(editGame.category || '未分类');
      setTags((editGame.tags || []).join(', '));
      setStatus(editGame.status || '未游玩');
      setCover(editGame.cover || '');
      setDesc(editGame.description || '');
    }
  }, [isEdit, editGame]);

  // 监听全局取消事件
  useEffect(() => {
    const onCancel = () => { abortRef.current = true; };
    window.addEventListener('loading-cancel', onCancel);
    return () => window.removeEventListener('loading-cancel', onCancel);
  }, []);

  const handleBrowse = async () => {
    try {
      const selected = await invoke('open_file_dialog', { title: '选择游戏启动程序', extensions: ['exe'] });
      if (selected) { setPath(selected); setDirty(true); }
    } catch (e) { showToast('error', '选择文件失败:' + e); }
  };

  const handleCoverBrowse = async () => {
    try {
      const selected = await invoke('open_file_dialog', { title: '选择封面图片', extensions: ['png', 'jpg', 'jpeg', 'gif', 'bmp'] });
      if (selected) { setCover(selected); setDirty(true); }
    } catch (e) { showToast('error', '选择图片失败:' + e); }
  };

  const handleBangumiSearch = async () => {
    const kw = bgmKeyword.trim();
    if (!kw) { showToast('warning', '请输入游戏名称'); return; }
    setBgmLoading(true);
    setBgmResults([]);
    searchKwRef.current = kw;  // 保存搜索关键词
    try {
      const list = await invoke('search_bangumi', { keyword: kw });
      setBgmResults(list || []);
    } catch (e) {
      showToast('error', formatError('Bangumi 搜索', e));
    } finally {
      setBgmLoading(false);
    }
  };

  const handleBangumiSelect = async (subjectId) => {
    // 防止重复点击: 如果已有请求在进行中则忽略
    if (selectingRef.current) return;
    selectingRef.current = subjectId;
    setSelectingId(subjectId);
    abortRef.current = false;
    dispatch({ type: 'SET_LOADING', payload: `获取 Bangumi 数据 (${subjectId})...` });
    // 使用 ref 中的搜索关键词,避免闭包中 bgmKeyword 已被清空
    const kw = searchKwRef.current || null;

    // 保存当前搜索结果的快照,用于回退
    const cachedResults = bgmResults;

    try {
      // === 第1层: fetch_bangumi_game (含15秒超时) ===
      const FETCH_TIMEOUT_MS = 15_000;
      let data = null;
      try {
        data = await Promise.race([
          invoke('fetch_bangumi_game', { subjectId, keyword: kw }),
          new Promise((_, reject) =>
            setTimeout(() => reject(new Error('获取超时(15s)：请检查网络后重试')), FETCH_TIMEOUT_MS)
          ),
        ]);
      } catch (fetchErr) {
        // fetch 失败或超时,进入回退逻辑
        console.warn('[AddGameModal] fetch_bangumi_game failed, trying fallback:', fetchErr);

        if (abortRef.current) { showToast('info', '已取消数据填充'); return; }

        // === 第2层: 使用搜索缓存 + 独立封面下载 ===
        const cached = cachedResults.find(item => item.id === subjectId);
        if (cached && (cached.summary || cached.image_large || cached.image)) {
          dispatch({ type: 'SET_LOADING', payload: '使用搜索结果获取数据...' });

          // 独立下载封面 (10秒超时)
          let coverBase64 = '';
          const coverUrl = cached.image_large || cached.image;
          if (coverUrl) {
            try {
              coverBase64 = await Promise.race([
                invoke('download_bangumi_cover', { imageUrl: coverUrl }),
                new Promise((_, reject) => setTimeout(() => reject(new Error('封面下载超时')), 10_000)),
              ]);
            } catch (coverErr) {
              console.warn('[AddGameModal] fallback cover download failed:', coverErr);
            }
          }

          if (abortRef.current) { showToast('info', '已取消数据填充'); return; }

          data = {
            name: cached.name || '',
            name_cn: cached.name_cn || '',
            summary: cached.summary || '',
            cover: coverBase64,
            tags: [],
            date: cached.date || '',
          };
          console.log('[AddGameModal] using fallback data from search cache');
        } else {
          // 搜索缓存也没有匹配项,报错
          throw fetchErr;
        }
      }

      if (abortRef.current) { showToast('info', '已取消数据填充'); return; }

      // === 第3层: 应用数据到表单 ===
      if (data && (data.name || data.name_cn || data.summary)) {
        if (data.name_cn) setName(data.name_cn);
        else if (data.name) setName(data.name);
        if (data.name_cn && data.name && data.name_cn !== data.name) setAlias(data.name);
        if (data.summary) setDesc(data.summary);
        if (data.tags && data.tags.length) setTags(data.tags.join(', '));
        if (data.cover) {
          setCover('[Bangumi]');
          setBgmCoverData(data.cover);
        }
        setBgmResults([]);
        setBgmKeyword('');
        searchKwRef.current = '';
        setDirty(true);

        // 根据数据完整度给出不同提示
        const filled = [];
        if (data.name_cn || data.name) filled.push('名称');
        if (data.summary) filled.push('简介');
        if (data.tags && data.tags.length) filled.push(`标签(${data.tags.length}个)`);
        if (data.cover) filled.push('封面');
        showToast('success', filled.length > 0 ? '已填充: ' + filled.join('、') : '未获取到有效数据');
      } else {
        showToast('warning', '获取的游戏数据为空，请尝试其他结果');
      }
    } catch (e) {
      console.error('Bangumi fetch failed:', e);
      if (!abortRef.current) {
        showToast('error', formatError('Bangumi 获取', e));
      }
    } finally {
      dispatch({ type: 'SET_LOADING', payload: null });
      selectingRef.current = null;
      setSelectingId(null);
    }
  };

  const handleSubmit = async (e) => {
    e.preventDefault();
    const trimmedName = name.trim();
    if (!trimmedName) { showToast('warning', '请输入游戏名称'); return; }
    const trimmedPath = path.trim();
    if (!trimmedPath) { showToast('warning', '请输入启动路径'); return; }

    let coverData = '';
    if (cover === '[Bangumi]') {
      coverData = bgmCoverData;
    } else if (cover && (!isEdit || cover !== editGame?.cover)) {
      try {
        dispatch({ type: 'SET_LOADING', payload: '处理封面图片...' });
        coverData = await invoke('copy_cover', { sourcePath: cover, gameId: isEdit ? editGameId : Date.now().toString() });
      } catch (e) {
        showToast('error', formatError('封面', e));
        dispatch({ type: 'SET_LOADING', payload: null });
        return;
      }
    } else {
      coverData = isEdit ? (editGame?.cover || '') : '';
    }

    const gameObj = {
      id: isEdit ? editGameId : Date.now().toString(),
      name: trimmedName,
      alias: alias.trim(),
      path: trimmedPath,
      cover: coverData,
      category: category || '未分类',
      tags: tags.split(',').map(s => s.trim()).filter(Boolean),
      status,
      description: desc.trim(),
      play_time: isEdit ? (editGame?.play_time || 0) : 0,
      last_play: isEdit ? (editGame?.last_play || null) : null,
      favorite: isEdit ? (editGame?.favorite || false) : false,
    };

    try {
      dispatch({ type: 'SET_LOADING', payload: isEdit ? '更新游戏...' : '添加游戏...' });
      if (isEdit) {
        await invoke('update_game', { game: gameObj });
      } else {
        await invoke('add_game', { game: gameObj });
      }
      await onSaved();
      showToast('success', isEdit ? '游戏更新成功' : '游戏添加成功');
    } catch (e) {
      showToast('error', formatError(isEdit ? '更新游戏' : '添加游戏', e));
      dispatch({ type: 'SET_LOADING', payload: null });
    }
  };

  return (
    <Modal onClose={onClose} title={isEdit ? '编辑游戏' : '添加新游戏'} icon="plus-circle" size="lg" confirmBeforeClose={dirty}>
      <div className="bgm-section">
        <div className="bgm-notice">
          <span className="notice-icon"><Icon name="info" size={14} /></span>
          <span>Bangumi 为境外网站,请确保已开启科学上网工具,否则搜索功能可能无法正常使用。</span>
        </div>
        <div className="bgm-search-bar">
          <input type="text" className="form-input" placeholder="在 Bangumi 搜索游戏,自动填充封面与简介..." value={bgmKeyword} onChange={(e) => { setBgmKeyword(e.target.value); setDirty(true); }} onKeyDown={(e) => { if (e.key === 'Enter') handleBangumiSearch(); }} />
          <button type="button" className="btn btn-primary btn-sm" onClick={handleBangumiSearch}>
            <Icon name="search" size={14} /> 搜索
          </button>
        </div>
        {bgmLoading && (
          <div className="bgm-loading">
            <Icon name="refresh-cw" size={14} className="spin" /> 正在搜索...
          </div>
        )}
        {bgmResults.length > 0 && (
          <div className="bgm-results">
            {bgmResults.map((item, idx) => {
              const isSelecting = selectingId === item.id;
              const isDisabled = selectingId !== null;
              return (
                <div key={idx} className={`bgm-result-item${isSelecting ? ' bgm-result-selecting' : ''}`}>
                  {item.image && <img className="bgm-result-cover" src={item.image} alt="" loading="lazy" />}
                  <div className="bgm-result-info">
                    <div className="bgm-result-title">{escapeHtml(item.name_cn || item.name)}</div>
                    <div className="bgm-result-subtitle">{escapeHtml(item.name !== item.name_cn ? item.name : '')}</div>
                    <div className="bgm-result-meta">
                      {item.date && (
                        <span><Icon name="clock" size={12} /> {escapeHtml(item.date)}</span>
                      )}
                      {item.score > 0 && (
                        <span><Icon name="star" size={12} color="#fbbf24" /> {item.score.toFixed(1)}</span>
                      )}
                      {item.rank > 0 && (
                        <span><Icon name="trophy" size={12} /> #{item.rank}</span>
                      )}
                    </div>
                  </div>
                  <button
                    className="btn btn-primary btn-sm"
                    onClick={() => handleBangumiSelect(item.id)}
                    disabled={isDisabled}
                  >
                    {isSelecting ? <><Icon name="refresh-cw" size={12} className="spin" /> 获取中</> : '选择'}
                  </button>
                </div>
              );
            })}
          </div>
        )}
      </div>

      <form onSubmit={handleSubmit} autoComplete="off">
        <div className="form-group">
          <label>游戏名称 *</label>
          <input type="text" className="form-input" value={name} onChange={(e) => { setName(e.target.value); setDirty(true); }} placeholder="输入游戏名称" required />
        </div>
        <div className="form-group">
          <label>游戏别名</label>
          <input type="text" className="form-input" value={alias} onChange={(e) => { setAlias(e.target.value); setDirty(true); }} placeholder="别名(可选)" />
        </div>
        <div className="form-group">
          <label>启动程序路径 *</label>
          <div className="form-path-input">
            <input type="text" className="form-input" value={path} onChange={(e) => { setPath(e.target.value); setDirty(true); }} placeholder="选择 .exe 文件路径" required />
            <button type="button" className="btn btn-secondary btn-sm" onClick={handleBrowse}>
              <Icon name="folder-open" size={14} /> 浏览
            </button>
          </div>
        </div>
        <div className="form-group">
          <label>分类</label>
          <div className="cat-input-row">
            <input
              type="text"
              className="form-input"
              value={category}
              onChange={(e) => { setCategory(e.target.value); setDirty(true); }}
              list="category-datalist-modal"
              placeholder="选择或输入新分类"
              autoComplete="off"
              style={{ flex: 1 }}
            />
            <datalist id="category-datalist-modal">
              {!Object.keys(categories).includes('未分类') && <option value="未分类" />}
              {Object.keys(categories).sort().map(c => <option key={c} value={c} />)}
            </datalist>
          </div>
        </div>
        <div className="form-group">
          <label>标签(逗号分隔)</label>
          <input type="text" className="form-input" value={tags} onChange={(e) => { setTags(e.target.value); setDirty(true); }} placeholder="例如:纯爱, 汉化, 短篇" />
        </div>
        <div className="form-group">
          <label>游玩状态</label>
          <select className="form-select" value={status} onChange={(e) => { setStatus(e.target.value); setDirty(true); }}>
            <option value="未游玩">未游玩</option>
            <option value="游玩中">游玩中</option>
            <option value="已通关">已通关</option>
            <option value="搁置">搁置</option>
          </select>
        </div>
        <div className="form-group">
          <label>封面图(可选)</label>
          <div className="form-path-input">
            <input type="text" className="form-input" value={cover} onChange={(e) => { setCover(e.target.value); setDirty(true); }} placeholder="图片路径或留空" />
            <button type="button" className="btn btn-secondary btn-sm" onClick={handleCoverBrowse}>
              <Icon name="image" size={14} /> 选择
            </button>
          </div>
        </div>
        <div className="form-group">
          <label>简介</label>
          <textarea className="form-textarea" value={desc} onChange={(e) => { setDesc(e.target.value); setDirty(true); }} rows="3" placeholder="输入游戏简介..."></textarea>
        </div>
        <div className="form-actions">
          <button type="button" className="btn btn-secondary" onClick={onClose}>取消</button>
          <button type="submit" className="btn btn-primary">
            <Icon name="check" size={14} /> {isEdit ? '确认更新' : '确认添加'}
          </button>
        </div>
      </form>
    </Modal>
  );
}
