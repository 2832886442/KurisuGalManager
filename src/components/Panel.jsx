import { useState, useMemo } from 'react';
import { useGame } from '../hooks/useGameData';
import { escapeHtml, debounce } from '../utils';
import Icon from './Icon';

export default function Panel() {
  const { state, dispatch, getCategories, getTags } = useGame();
  const [tagSearch, setTagSearch] = useState('');

  const categories = useMemo(() => getCategories(), [getCategories]);
  const tags = useMemo(() => getTags(), [getTags]);

  const filteredTags = useMemo(() => {
    let sorted = Object.keys(tags).sort();
    if (tagSearch.trim()) {
      const kw = tagSearch.trim().toLowerCase();
      sorted = sorted.filter(t => t.toLowerCase().includes(kw));
    }
    return sorted;
  }, [tags, tagSearch]);

  const handleSearch = useMemo(
    () => debounce((value) => dispatch({ type: 'SET_FILTER', payload: { search: value } }), 300),
    [dispatch]
  );

  const handleCategoryClick = (cat) => {
    dispatch({ type: 'SET_ACTIVE_NAV', payload: 'games' });
    dispatch({ type: 'SET_FILTER', payload: { category: cat, tag: null, favorites: false } });
    dispatch({ type: 'EXIT_BATCH' });
  };

  const handleTagClick = (tag) => {
    dispatch({ type: 'SET_ACTIVE_NAV', payload: 'games' });
    if (state.filters.tag === tag) {
      dispatch({ type: 'SET_FILTER', payload: { tag: null, category: 'all', favorites: false } });
    } else {
      dispatch({ type: 'SET_FILTER', payload: { tag, category: 'all', favorites: false } });
    }
    dispatch({ type: 'EXIT_BATCH' });
  };

  const isGames = state.activeNav === 'games' || state.activeNav === 'favorites';

  return (
    <aside className="panel">
      <div className="panel-search">
        <div className="panel-search-wrap">
          <span className="panel-search-icon"><Icon name="search" size={14} /></span>
          <input type="text" className="panel-search-input" placeholder="搜索游戏名称..."
            onChange={(e) => handleSearch(e.target.value)} />
        </div>
      </div>
      <div className="panel-section panel-section-cats">
        <div className="panel-section-title">分类</div>
        <div className="panel-section-scroll">
          {Object.keys(categories).sort().map(cat => (
            <div key={cat} className={`panel-cat-item${state.filters.category === cat && !state.filters.favorites && isGames ? ' active' : ''}`}
              onClick={() => handleCategoryClick(cat)}>
              {escapeHtml(cat)}
              <span className="panel-cat-count">{categories[cat]}</span>
            </div>
          ))}
        </div>
      </div>
      <div className="panel-section panel-section-tags">
        <div className="panel-section-title">标签</div>
        <input type="text" className="panel-tag-search" placeholder="搜索标签..." value={tagSearch} onChange={(e) => setTagSearch(e.target.value)} />
        <div className="panel-section-scroll">
          {filteredTags.length === 0 ? (
            <span style={{ fontSize: '0.7rem', color: 'var(--text-muted)', padding: '4px 14px', display: 'block' }}>
              {tagSearch.trim() ? '无匹配标签' : '暂无标签'}
            </span>
          ) : (
            filteredTags.map(tag => (
              <span key={tag} className={`panel-tag-item${state.filters.tag === tag ? ' active' : ''}`} onClick={() => handleTagClick(tag)}>
                {tag} ({tags[tag]})
              </span>
            ))
          )}
        </div>
      </div>
      <div className="panel-footer">
        <span className="panel-footer-dot"></span>
        <span className="panel-footer-text">本地运行</span>
        <span className="panel-footer-count">{state.games.length} 款 · 收藏 {state.games.filter(g => g.favorite).length}</span>
      </div>
    </aside>
  );
}
