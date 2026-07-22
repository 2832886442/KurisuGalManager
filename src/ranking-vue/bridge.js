import { reactive, watch } from 'vue';

// 与 React 共享的状态
export const bridgeState = reactive({
  rankings: [],
  selectedRanking: null,
});

// React 侧设置的回调，当 Vue 修改数据时通知 React
let reactListener = null;

export function onBridgeChange(fn) {
  reactListener = fn;
}

watch(() => bridgeState.rankings, () => {
  if (reactListener) reactListener({ rankings: [...bridgeState.rankings], selectedRanking: bridgeState.selectedRanking });
});

watch(() => bridgeState.selectedRanking, () => {
  if (reactListener) reactListener({ rankings: [...bridgeState.rankings], selectedRanking: bridgeState.selectedRanking });
});

// React 调用此函数同步数据到 Vue
export function syncFromReact(rankings, selectedRanking) {
  bridgeState.rankings = rankings;
  bridgeState.selectedRanking = selectedRanking;
}
