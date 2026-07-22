import { useMemo } from 'react';
import { useGame } from '../hooks/useGameData';
import GameCard from './GameCard';
import Pagination from './Pagination';
import Icon from './Icon';

export default function GameGrid({ onShowDetail }) {
  const { state, getFilteredGames } = useGame();
  const filtered = useMemo(() => getFilteredGames(), [getFilteredGames]);
  const totalPages = Math.max(1, Math.ceil(filtered.length / state.pageSize));
  const currentPage = Math.min(state.currentPage, totalPages);
  const startIdx = (currentPage - 1) * state.pageSize;
  const pagedGames = filtered.slice(startIdx, startIdx + state.pageSize);

  if (filtered.length === 0) {
    return (
      <>
        <div className="empty-state">
          <div className="empty-icon"><Icon name="library" size={48} strokeWidth={1.2} /></div>
          <h3>还没有游戏</h3>
          <p>点击侧边栏的「+」按钮或上方胶囊栏的添加图标开始管理你的 Galgame 收藏</p>
        </div>
        <div style={{ display: 'none' }}></div>
      </>
    );
  }

  return (
    <>
      <div className={`game-grid ${state.filters.view === 'grid' ? 'grid-view' : 'list-view'}`}>
        {pagedGames.map(game => (
          <div key={game.id} onClick={() => !state.batchMode && onShowDetail(game.id)}>
            <GameCard game={game} />
          </div>
        ))}
      </div>
      <Pagination totalItems={filtered.length} />
    </>
  );
}
