import { createContext, useContext, useReducer, useCallback } from 'react';

const GameContext = createContext(null);

const initialState = {
  games: [],
  activeNav: 'welcome',
  filters: { category: 'all', tag: null, search: '', view: 'grid', favorites: false },
  sort: localStorage.getItem('sort') || 'name-asc',
  batchMode: false,
  selectedIds: [],
  runningIds: [],
  currentPage: 1,
  pageSize: parseInt(localStorage.getItem('pageSize') || '24', 10),
  customCategories: JSON.parse(localStorage.getItem('customCategories') || '[]'),
  tagSearchKeyword: '',
  loading: null,
};

function gameReducer(state, action) {
  switch (action.type) {
    case 'SET_GAMES':
      return { ...state, games: action.payload };
    case 'SET_ACTIVE_NAV':
      return { ...state, activeNav: action.payload };
    case 'SET_FILTER':
      return { ...state, filters: { ...state.filters, ...action.payload }, currentPage: 1 };
    case 'SET_VIEW':
      return { ...state, filters: { ...state.filters, view: action.payload } };
    case 'SET_SORT':
      localStorage.setItem('sort', action.payload);
      return { ...state, sort: action.payload };
    case 'SET_PAGE':
      return { ...state, currentPage: action.payload };
    case 'SET_PAGE_SIZE':
      localStorage.setItem('pageSize', String(action.payload));
      return { ...state, pageSize: action.payload, currentPage: 1 };
    case 'TOGGLE_BATCH':
      return { ...state, batchMode: !state.batchMode, selectedIds: [] };
    case 'EXIT_BATCH':
      return { ...state, batchMode: false, selectedIds: [] };
    case 'TOGGLE_SELECT':
      const id = action.payload;
      const selected = state.selectedIds.includes(id)
        ? state.selectedIds.filter(s => s !== id)
        : [...state.selectedIds, id];
      return { ...state, selectedIds: selected };
    case 'SELECT_ALL':
      return { ...state, selectedIds: action.payload };
    case 'SET_RUNNING':
      return { ...state, runningIds: action.payload };
    case 'ADD_RUNNING':
      return { ...state, runningIds: [...state.runningIds, action.payload] };
    case 'REMOVE_RUNNING':
      return { ...state, runningIds: state.runningIds.filter(id => id !== action.payload) };
    case 'SET_CUSTOM_CATEGORIES':
      localStorage.setItem('customCategories', JSON.stringify(action.payload));
      return { ...state, customCategories: action.payload };
    case 'SET_TAG_SEARCH':
      return { ...state, tagSearchKeyword: action.payload };
    case 'SET_LOADING':
      return { ...state, loading: action.payload, loadingAborted: false };
    case 'CANCEL_LOADING':
      return { ...state, loading: null, loadingAborted: true };
    case 'UPDATE_GAME':
      return {
        ...state,
        games: state.games.map(g =>
          g.id === action.payload.id ? { ...g, ...action.payload } : g
        ),
      };
    case 'REMOVE_GAME':
      return {
        ...state,
        games: state.games.filter(g => g.id !== action.payload),
      };
    case 'BATCH_UPDATE_CATEGORY':
      return {
        ...state,
        games: state.games.map(g =>
          action.payload.gameIds.includes(g.id)
            ? { ...g, category: action.payload.category }
            : g
        ),
      };
    default:
      return state;
  }
}

export function GameProvider({ children }) {
  const [state, dispatch] = useReducer(gameReducer, initialState);

  const getFilteredGames = useCallback(() => {
    let list = state.games;
    if (state.filters.favorites) {
      list = list.filter(g => g.favorite);
    }
    if (state.filters.category !== 'all') {
      list = list.filter(g => g.category === state.filters.category);
    }
    if (state.filters.tag) {
      list = list.filter(g => g.tags.includes(state.filters.tag));
    }
    if (state.filters.search.trim()) {
      const kw = state.filters.search.trim().toLowerCase();
      list = list.filter(g =>
        g.name.toLowerCase().includes(kw) ||
        (g.alias && g.alias.toLowerCase().includes(kw))
      );
    }
    // 排序
    const sorted = [...list];
    switch (state.sort) {
      case 'name-asc':  sorted.sort((a, b) => (a.name || '').localeCompare(b.name || '', 'zh-CN')); break;
      case 'name-desc': sorted.sort((a, b) => (b.name || '').localeCompare(a.name || '', 'zh-CN')); break;
      case 'play-desc': sorted.sort((a, b) => (b.play_time || 0) - (a.play_time || 0)); break;
      case 'play-asc':  sorted.sort((a, b) => (a.play_time || 0) - (b.play_time || 0)); break;
      case 'last-desc': sorted.sort((a, b) => (b.last_played || 0) - (a.last_played || 0)); break;
      case 'added-desc':sorted.sort((a, b) => (b.added_at || b.id || 0) - (a.added_at || a.id || 0)); break;
      default: break;
    }
    return sorted;
  }, [state.games, state.filters, state.sort]);

  const getCategories = useCallback(() => {
    const cats = {};
    state.games.forEach(g => { cats[g.category] = (cats[g.category] || 0) + 1; });
    state.customCategories.forEach(c => { if (!cats[c]) cats[c] = 0; });
    return cats;
  }, [state.games, state.customCategories]);

  const getTags = useCallback(() => {
    const tags = {};
    state.games.forEach(g => {
      g.tags.forEach(t => { tags[t] = (tags[t] || 0) + 1; });
    });
    return tags;
  }, [state.games]);

  return (
    <GameContext.Provider
      value={{ state, dispatch, getFilteredGames, getCategories, getTags }}
    >
      {children}
    </GameContext.Provider>
  );
}

export function useGame() {
  const ctx = useContext(GameContext);
  if (!ctx) throw new Error('useGame must be used within GameProvider');
  return ctx;
}
