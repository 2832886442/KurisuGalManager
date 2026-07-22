import { useGame } from '../hooks/useGameData';
import Icon from './Icon';

/**
 * 顶部固定工具栏(主操作区)
 * 显示当前视图标题、游戏计数、排序下拉、视图切换、批量按钮。
 * 仅在「游戏列表/收藏夹」视图渲染(主页不显示)。
 */
const SORT_OPTIONS = [
  { value: 'name-asc',  label: '名称 A → Z' },
  { value: 'name-desc', label: '名称 Z → A' },
  { value: 'play-desc', label: '游玩时长 ↓' },
  { value: 'play-asc',  label: '游玩时长 ↑' },
  { value: 'last-desc', label: '最近游玩 ↓' },
  { value: 'added-desc',label: '添加时间 ↓' },
];

function getViewTitle(activeNav, filters) {
  if (activeNav === 'favorites') return '收藏夹';
  if (filters.favorites) return '收藏夹';
  if (filters.tag) return `标签:${filters.tag}`;
  if (filters.category && filters.category !== 'all') return `分类:${filters.category}`;
  return '所有游戏';
}

export default function Toolbar() {
  const { state, dispatch } = useGame();

  const totalCount = state.games.length;
  const filteredCount = (() => {
    let n = state.games.length;
    if (state.filters.favorites) n = state.games.filter(g => g.favorite).length;
    if (state.filters.category !== 'all') n = state.games.filter(g => g.category === state.filters.category).length;
    if (state.filters.tag) n = state.games.filter(g => g.tags.includes(state.filters.tag)).length;
    if (state.filters.search.trim()) {
      const kw = state.filters.search.trim().toLowerCase();
      n = state.games.filter(g => g.name.toLowerCase().includes(kw) || (g.alias && g.alias.toLowerCase().includes(kw))).length;
    }
    return n;
  })();

  return (
    <header className="top-toolbar">
      <span className="toolbar-title">{getViewTitle(state.activeNav, state.filters)}</span>
      <span className="toolbar-count">{filteredCount} / {totalCount}</span>

      <span className="toolbar-spacer" />

      <select
        className="sort-select"
        value={state.sort}
        onChange={(e) => dispatch({ type: 'SET_SORT', payload: e.target.value })}
        title="排序方式"
      >
        {SORT_OPTIONS.map(o => <option key={o.value} value={o.value}>{o.label}</option>)}
      </select>

      <div className="toolbar-group" role="group" aria-label="视图切换">
        <button
          className={`toolbar-btn${state.filters.view === 'grid' ? ' active' : ''}`}
          title="网格视图"
          onClick={() => dispatch({ type: 'SET_VIEW', payload: 'grid' })}
        >
          <Icon name="layout-grid" size={14} />
        </button>
        <button
          className={`toolbar-btn${state.filters.view === 'list' ? ' active' : ''}`}
          title="列表视图"
          onClick={() => dispatch({ type: 'SET_VIEW', payload: 'list' })}
        >
          <Icon name="list" size={14} />
        </button>
      </div>

      <button
        className={`toolbar-btn${state.batchMode ? ' active' : ''}`}
        title={state.batchMode ? '退出批量' : '批量选择'}
        onClick={() => dispatch({ type: 'TOGGLE_BATCH' })}
      >
        <Icon name="check-square" size={14} />
        <span>{state.batchMode ? '退出' : '批量'}</span>
      </button>
    </header>
  );
}
