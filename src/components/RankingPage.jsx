import { useEffect, useRef, useState, useCallback } from 'react';
import { createApp } from 'vue';
import { useGame } from '../hooks/useGameData';
import { showToast } from './Toast';
import { bridgeState, onBridgeChange } from '../ranking-vue/bridge';
import RankingApp from '../ranking-vue/RankingApp.vue';

/**
 * RankingPage — React×Vue3 混合容器
 * 排名管理 UI 完全由 Vue3 渲染，React 仅负责挂载和数据桥接。
 */
export default function RankingPage() {
  const { state } = useGame();
  const containerRef = useRef(null);
  const vueApp = useRef(null);
  const [, tick] = useState(0);

  // 暴露全局桥接
  useEffect(() => {
    window.__kurisuToast = showToast;
    window.__kurisuGalGames = state.games;
    return () => {
      window.__kurisuToast = undefined;
      window.__kurisuGalGames = undefined;
    };
  }, []);

  // 同步游戏数据到 Vue
  useEffect(() => {
    window.__kurisuGalGames = state.games;
  }, [state.games]);

  // 监听 Vue 数据变更，刷新 UI
  const handleBridgeChange = useCallback(() => {
    tick(n => n + 1);
  }, []);
  useEffect(() => {
    onBridgeChange(handleBridgeChange);
  }, [handleBridgeChange]);

  // 挂载 Vue 应用
  useEffect(() => {
    if (!containerRef.current) return;
    const app = createApp(RankingApp);
    app.mount(containerRef.current);
    vueApp.current = app;
    return () => {
      app.unmount();
      vueApp.current = null;
    };
  }, []);

  return <div ref={containerRef} className="ranking-vue-root" />;
}
