import { useState, useMemo } from 'react';
import { useGame } from '../hooks/useGameData';
import { escapeHtml } from '../utils';
import Icon from './Icon';

export default function BatchBar({ onApply }) {
  const { state, dispatch, getFilteredGames, getCategories } = useGame();
  const filtered = useMemo(() => getFilteredGames(), [getFilteredGames]);
  const categories = useMemo(() => getCategories(), [getCategories]);
  const [selectedCat, setSelectedCat] = useState('');

  const handleSelectAll = () => {
    if (state.selectedIds.length === filtered.length && state.selectedIds.length > 0) {
      dispatch({ type: 'SELECT_ALL', payload: [] });
    } else {
      dispatch({ type: 'SELECT_ALL', payload: filtered.map(g => g.id) });
    }
  };

  const allSelected = state.selectedIds.length === filtered.length && state.selectedIds.length > 0;

  return (
    <div className="batch-bar">
      <span className="batch-info">
        <Icon name="check-square" size={14} /> 已选 {state.selectedIds.length} 项
      </span>
      <button className="btn btn-sm" onClick={handleSelectAll}>
        <Icon name={allSelected ? 'x' : 'check-check'} size={13} />
        {allSelected ? '取消全选' : '全选当前'}
      </button>
      <div className="batch-actions">
        <select className="batch-select" value={selectedCat} onChange={(e) => setSelectedCat(e.target.value)}>
          <option value="">-- 移动到分类 --</option>
          {Object.keys(categories).sort().map(c => <option key={c} value={c}>{escapeHtml(c)}</option>)}
        </select>
        <button
          className="btn btn-primary btn-sm"
          onClick={() => onApply(selectedCat)}
          disabled={!selectedCat || state.selectedIds.length === 0}
        >
          <Icon name="check" size={13} /> 应用
        </button>
      </div>
      <button className="btn btn-sm" onClick={() => dispatch({ type: 'EXIT_BATCH' })}>
        <Icon name="x" size={13} /> 取消
      </button>
    </div>
  );
}
