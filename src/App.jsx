import { useEffect, useCallback, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { GameProvider, useGame } from './hooks/useGameData';
import Sidebar from './components/Sidebar';
import Panel from './components/Panel';
import Toolbar from './components/Toolbar';
import GameGrid from './components/GameGrid';
import HomePage from './components/HomePage';
import WelcomePage from './components/WelcomePage';
import AboutPage from './components/AboutPage';
import RankingPage from './components/RankingPage';
import SettingsPage from './components/SettingsPage';
import DetailModal from './components/DetailModal';
import AddGameModal from './components/AddGameModal';
import CategoryManager from './components/CategoryManager';
import FloatToolbar from './components/FloatToolbar';
import BatchBar from './components/BatchBar';
import ToastContainer, { showToast } from './components/Toast';
import GlobalLoader from './components/GlobalLoader';
import { formatError } from './utils';

function AppInner() {
    const { state, dispatch } = useGame();
    const [showAdd, setShowAdd] = useState(false);
    const [showCats, setShowCats] = useState(false);
    const [showDetail, setShowDetail] = useState(null);
    const [editGameId, setEditGameId] = useState(null);

    const loadGames = useCallback(async () => {
        try {
            dispatch({ type: 'SET_LOADING', payload: '加载游戏库...' });
            const data = await invoke('get_games');
            dispatch({ type: 'SET_GAMES', payload: data.games || [] });
            const ids = (data.games || []).filter(g => g.cover && !g.cover.startsWith('data:')).map(g => g.id);
            if (ids.length > 0) {
                try {
                    const covers = await invoke('get_game_covers', { gameIds: ids });
                    const gamesWithCovers = (data.games || []).map(g => {
                        if (covers[g.id]) return { ...g, cover: covers[g.id] };
                        return g;
                    });
                    dispatch({ type: 'SET_GAMES', payload: gamesWithCovers });
                } catch (e) { console.warn('封面加载失败:', e); }
            }
            const cats = [];
            (data.games || []).forEach(g => { if (g.category && g.category !== '未分类' && !cats.includes(g.category)) cats.push(g.category); });
            const stored = JSON.parse(localStorage.getItem('customCategories') || '[]');
            const merged = [...new Set([...stored, ...cats])];
            dispatch({ type: 'SET_CUSTOM_CATEGORIES', payload: merged });
        } catch (e) {
            showToast('error', formatError('加载游戏数据', e));
        } finally {
            dispatch({ type: 'SET_LOADING', payload: null });
        }
    }, [dispatch]);

    const loadCovers = useCallback(async () => {
        const ids = state.games.filter(g => g.cover && !g.cover.startsWith('data:')).map(g => g.id);
        if (ids.length === 0) return;
        try {
            const covers = await invoke('get_game_covers', { gameIds: ids });
            const updated = state.games.map(g => { if (covers[g.id]) return { ...g, cover: covers[g.id] }; return g; });
            dispatch({ type: 'SET_GAMES', payload: updated });
        } catch (e) { console.warn('封面加载失败:', e); }
    }, [state.games, dispatch]);

    useEffect(() => { loadGames(); }, []);

    useEffect(() => {
        const unlisten = listen('game-exited', (event) => {
            const { game_id, play_time_added, updated } = event.payload;
            dispatch({ type: 'REMOVE_RUNNING', payload: game_id });
            if (updated) {
                const game = state.games.find(g => g.id === game_id);
                showToast('info', `「${game?.name || game_id}」已退出，本次游玩 +${play_time_added} 分钟`);
                loadGames();
            }
        });
        return () => { unlisten.then(fn => fn()); };
    }, [state.games]);

    useEffect(() => {
        const unlisten = listen('screenshot-captured', (event) => {
            const { success, path, error } = event.payload;
            if (success) {
                showToast('success', `截图成功，保存到 ${path}`);
                loadGames();
            } else {
                showToast('error', `截图失败: ${error}`);
            }
        });
        return () => { unlisten.then(fn => fn()); };
    }, []);

    const handleAddGame = () => { setEditGameId(null); setShowAdd(true); };
    const handleEditGame = (id) => { setEditGameId(id); setShowAdd(true); };
    const handleGameSaved = async () => { setShowAdd(false); setEditGameId(null); await loadGames(); await loadCovers(); };

    const handleDeleteGame = async (id) => {
        try {
            dispatch({ type: 'SET_LOADING', payload: '删除游戏...' });
            const data = await invoke('delete_game', { id });
            const updatedGames = data.games || [];
            dispatch({ type: 'SET_GAMES', payload: updatedGames });
            const ids = updatedGames.filter(g => g.cover && !g.cover.startsWith('data:')).map(g => g.id);
            if (ids.length > 0) {
                try {
                    const covers = await invoke('get_game_covers', { gameIds: ids });
                    const gamesWithCovers = updatedGames.map(g => { if (covers[g.id]) return { ...g, cover: covers[g.id] }; return g; });
                    dispatch({ type: 'SET_GAMES', payload: gamesWithCovers });
                } catch (e) { console.warn('封面加载失败:', e); }
            }
            setShowDetail(null);
            showToast('success', '游戏已删除');
        } catch (e) { showToast('error', formatError('删除游戏', e)); }
        finally { dispatch({ type: 'SET_LOADING', payload: null }); }
    };

    const handleBatchApply = async (category) => {
        if (!category || state.selectedIds.length === 0) { showToast('warning', '请选择目标分类'); return; }
        try {
            dispatch({ type: 'SET_LOADING', payload: '批量移动中...' });
            const data = await invoke('batch_update_category', { gameIds: state.selectedIds, category });
            const updatedGames = data.games || [];
            dispatch({ type: 'SET_GAMES', payload: updatedGames });
            const ids = updatedGames.filter(g => g.cover && !g.cover.startsWith('data:')).map(g => g.id);
            if (ids.length > 0) {
                try {
                    const covers = await invoke('get_game_covers', { gameIds: ids });
                    const gamesWithCovers = updatedGames.map(g => { if (covers[g.id]) return { ...g, cover: covers[g.id] }; return g; });
                    dispatch({ type: 'SET_GAMES', payload: gamesWithCovers });
                } catch (e) { console.warn('封面加载失败:', e); }
            }
            dispatch({ type: 'EXIT_BATCH' });
            showToast('success', `已将游戏移动到「${category}」`);
        } catch (e) { showToast('error', formatError('批量操作', e)); }
        finally { dispatch({ type: 'SET_LOADING', payload: null }); }
    };

    const isGamesView = state.activeNav !== 'welcome' && state.activeNav !== 'overview'
        && state.activeNav !== 'about' && state.activeNav !== 'ranking'
        && state.activeNav !== 'settings';
    const isRankingView = state.activeNav === 'ranking';
    const isFullscreenView = state.activeNav === 'welcome' || state.activeNav === 'overview' || state.activeNav === 'about';

    return (
        <div className="app">
            <Sidebar
                onAddGame={handleAddGame}
                onManageCats={() => setShowCats(true)}
                onRefresh={loadGames}
            />
            {!isFullscreenView && !isRankingView && state.activeNav !== 'settings' && <Panel />}
            <main className="main-content">
                {isGamesView && <Toolbar />}
                {isGamesView && (
                    <>
                        <FloatToolbar onAddGame={handleAddGame} onRefresh={loadGames} />
                        {state.batchMode && <BatchBar onApply={handleBatchApply} />}
                    </>
                )}
                {state.activeNav === 'welcome' ? (
                    <WelcomePage />
                ) : state.activeNav === 'overview' ? (
                    <HomePage onShowDetail={setShowDetail} />
                ) : state.activeNav === 'about' ? (
                    <AboutPage />
                ) : state.activeNav === 'ranking' ? (
                    <RankingPage />
                ) : state.activeNav === 'settings' ? (
                    <SettingsPage />
                ) : (
                    <GameGrid onShowDetail={setShowDetail} />
                )}
            </main>

            {showDetail && <DetailModal gameId={showDetail} onClose={() => setShowDetail(null)} onEdit={handleEditGame} onDelete={handleDeleteGame} onRefresh={loadGames} />}
            {showAdd && <AddGameModal editGameId={editGameId} onClose={() => { setShowAdd(false); setEditGameId(null); }} onSaved={handleGameSaved} />}
            {showCats && <CategoryManager onClose={() => setShowCats(false)} onRefresh={loadGames} />}

            <ToastContainer />
            <GlobalLoader />
        </div>
    );
}

export default function App() {
    return <GameProvider><AppInner /></GameProvider>;
}
