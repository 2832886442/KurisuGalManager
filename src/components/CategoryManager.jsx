import { useState, useMemo } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { useGame } from '../hooks/useGameData';
import { escapeHtml, formatError } from '../utils';
import { showToast } from './Toast';
import Modal from './Modal';
import Icon from './Icon';

export default function CategoryManager({ onClose, onRefresh }) {
  const { state, dispatch, getCategories } = useGame();
  const categories = useMemo(() => getCategories(), [getCategories]);
  const [newCatName, setNewCatName] = useState('');
  const [renaming, setRenaming] = useState(null);
  const [renameValue, setRenameValue] = useState('');

  const handleCreate = async () => {
    const name = newCatName.trim();
    if (!name) { showToast('warning', '请输入分类名称'); return; }
    if (categories[name] !== undefined) { showToast('warning', '该分类已存在'); return; }
    const updated = [...state.customCategories, name];
    dispatch({ type: 'SET_CUSTOM_CATEGORIES', payload: updated });
    setNewCatName('');
    showToast('success', `分类「${name}」已创建`);
  };

  const handleStartRename = (cat) => {
    setRenaming(cat);
    setRenameValue(cat);
  };

  const handleConfirmRename = async () => {
    const oldName = renaming;
    const newName = renameValue.trim();
    setRenaming(null);
    if (!newName || newName === oldName) return;
    if (categories[newName] !== undefined && newName !== oldName) {
      showToast('warning', '目标分类名已存在');
      return;
    }
    try {
      dispatch({ type: 'SET_LOADING', payload: '重命名分类...' });
      const ids = state.games.filter(g => g.category === oldName).map(g => g.id);
      if (ids.length > 0) {
        const data = await invoke('batch_update_category', { gameIds: ids, category: newName });
        dispatch({ type: 'SET_GAMES', payload: data.games || [] });
      }
      const updated = state.customCategories.filter(c => c !== oldName);
      updated.push(newName);
      dispatch({ type: 'SET_CUSTOM_CATEGORIES', payload: updated });
      await onRefresh();
      showToast('success', `分类「${oldName}」已重命名为「${newName}」`);
    } catch (e) {
      showToast('error', formatError('重命名分类', e));
    } finally {
      dispatch({ type: 'SET_LOADING', payload: null });
    }
  };

  const handleDelete = async (cat) => {
    if (cat === '未分类') { showToast('warning', '「未分类」是默认分类,不能删除'); return; }
    const count = state.games.filter(g => g.category === cat).length;
    if (!confirm(`确定删除分类「${cat}」?\n\n该分类下有 ${count} 个游戏将被移到「未分类」。`)) return;
    try {
      dispatch({ type: 'SET_LOADING', payload: '删除分类...' });
      const ids = state.games.filter(g => g.category === cat).map(g => g.id);
      if (ids.length > 0) {
        const data = await invoke('batch_update_category', { gameIds: ids, category: '未分类' });
        dispatch({ type: 'SET_GAMES', payload: data.games || [] });
      }
      const updated = state.customCategories.filter(c => c !== cat);
      dispatch({ type: 'SET_CUSTOM_CATEGORIES', payload: updated });
      await onRefresh();
      showToast('success', `分类「${cat}」已删除`);
    } catch (e) {
      showToast('error', formatError('删除分类', e));
    } finally {
      dispatch({ type: 'SET_LOADING', payload: null });
    }
  };

  const keys = Object.keys(categories).filter(c => c !== '未分类').sort();

  return (
    <Modal onClose={onClose} title="管理分类" icon="folder-tree" size="sm">
      <div className="form-group" style={{ display: 'flex', gap: 8, marginBottom: 16 }}>
        <input
          type="text"
          className="form-input"
          placeholder="输入新分类名称"
          value={newCatName}
          onChange={(e) => setNewCatName(e.target.value)}
          onKeyDown={(e) => { if (e.key === 'Enter') handleCreate(); }}
          style={{ flex: 1 }}
        />
        <button className="btn btn-primary btn-sm" onClick={handleCreate}>
          <Icon name="plus-circle" size={14} /> 新建
        </button>
      </div>
      <div style={{ maxHeight: 300, overflowY: 'auto' }}>
        {keys.length === 0 ? (
          <div className="hint" style={{ padding: 16, textAlign: 'center' }}>暂无分类</div>
        ) : (
          keys.map(cat => (
            <div key={cat} className="cat-manager-item">
              {renaming === cat ? (
                <>
                  <input
                    type="text"
                    className="form-input cat-rename-input"
                    value={renameValue}
                    onChange={(e) => setRenameValue(e.target.value)}
                    onKeyDown={(e) => {
                      if (e.key === 'Enter') handleConfirmRename();
                      if (e.key === 'Escape') setRenaming(null);
                    }}
                    autoFocus
                    style={{ flex: 1 }}
                  />
                  <button className="btn btn-primary btn-sm" onClick={handleConfirmRename}>
                    <Icon name="check" size={14} />
                  </button>
                  <button className="btn btn-secondary btn-sm" onClick={() => setRenaming(null)}>
                    <Icon name="x" size={14} />
                  </button>
                </>
              ) : (
                <>
                  <span className="cat-manager-name">{escapeHtml(cat)}</span>
                  <span className="cat-manager-count">{categories[cat]} 个游戏</span>
                  <button className="btn btn-secondary btn-sm" onClick={() => handleStartRename(cat)}>
                    <Icon name="edit" size={13} /> 重命名
                  </button>
                  <button className="btn btn-danger btn-sm" onClick={() => handleDelete(cat)}>
                    <Icon name="trash-2" size={13} /> 删除
                  </button>
                </>
              )}
            </div>
          ))
        )}
      </div>
      <div className="settings-footer" style={{ marginTop: 12 }}>
        <button className="btn btn-secondary" onClick={onClose}>关闭</button>
      </div>
    </Modal>
  );
}
