import { useGame } from '../hooks/useGameData';
import Icon from './Icon';

export default function Pagination({ totalItems }) {
  const { state, dispatch } = useGame();
  const totalPages = Math.max(1, Math.ceil(totalItems / state.pageSize));
  const page = state.currentPage;
  if (totalPages <= 1) return null;

  const startItem = totalItems === 0 ? 0 : (page - 1) * state.pageSize + 1;
  const endItem = Math.min(page * state.pageSize, totalItems);

  return (
    <div className="pagination">
      <span className="page-info">{startItem}-{endItem} / {totalItems}</span>
      <button className="page-btn" disabled={page === 1} onClick={() => dispatch({ type: 'SET_PAGE', payload: 1 })} title="第一页">
        <Icon name="chevrons-left" size={14} />
      </button>
      <button className="page-btn" disabled={page === 1} onClick={() => dispatch({ type: 'SET_PAGE', payload: page - 1 })} title="上一页">
        <Icon name="chevron-left" size={14} />
      </button>
      <span className="page-current">{page} / {totalPages}</span>
      <button className="page-btn" disabled={page >= totalPages} onClick={() => dispatch({ type: 'SET_PAGE', payload: page + 1 })} title="下一页">
        <Icon name="chevron-right" size={14} />
      </button>
      <button className="page-btn" disabled={page >= totalPages} onClick={() => dispatch({ type: 'SET_PAGE', payload: totalPages })} title="最后一页">
        <Icon name="chevrons-right" size={14} />
      </button>
      <select className="page-size-select" value={state.pageSize} onChange={(e) => dispatch({ type: 'SET_PAGE_SIZE', payload: parseInt(e.target.value) })}>
        <option value="12">12/页</option>
        <option value="24">24/页</option>
        <option value="48">48/页</option>
        <option value="96">96/页</option>
      </select>
    </div>
  );
}
