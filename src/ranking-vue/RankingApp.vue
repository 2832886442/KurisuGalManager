<template>
  <div class="ranking-page">
    <!-- Header -->
    <div class="ranking-header">
      <div class="ranking-title-row">
        <h2 class="ranking-title">排名管理</h2>
        <button class="btn btn-primary btn-sm" @click="showCreateModal = true">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M12 5v14M5 12h14" />
          </svg>
          新建排名
        </button>
      </div>
      <p class="ranking-subtitle">将下方游戏库中的游戏拖拽到对应等级行即可完成排名，不同等级行之间也可拖拽调整</p>
    </div>

    <div class="ranking-layout">
      <!-- Sidebar -->
      <div class="ranking-sidebar">
        <div class="ranking-list-header"><span class="ranking-list-title">我的排名</span></div>
        <div class="ranking-list">
          <div v-if="rankings.length === 0" class="ranking-empty">
            <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
              <path
                d="M6 9H4.5a2.5 2.5 0 0 1 0-5H6M18 9h1.5a2.5 2.5 0 0 0 0-5H18M4 22h16M10 14.66V17c0 .55-.47.98-.97 1.21C7.85 18.75 7 20.24 7 22M14 14.66V17c0 .55.47.98.97 1.21C16.15 18.75 17 20.24 17 22M18 2H6v7a6 6 0 0 0 12 0V2z" />
            </svg>
            <span>暂无排名</span>
            <span class="ranking-empty-hint">点击上方按钮创建</span>
          </div>
          <div v-for="r in rankings" :key="r.id" class="ranking-item" :class="{ active: selected?.id === r.id }"
            @click="selectRanking(r)">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
              <path
                d="M6 9H4.5a2.5 2.5 0 0 1 0-5H6M18 9h1.5a2.5 2.5 0 0 0 0-5H18M4 22h16M10 14.66V17c0 .55-.47.98-.97 1.21C7.85 18.75 7 20.24 7 22M14 14.66V17c0 .55.47.98.97 1.21C16.15 18.75 17 20.24 17 22M18 2H6v7a6 6 0 0 0 12 0V2z" />
            </svg>
            <span class="ranking-item-name">{{ r.name }}</span>
            <button class="ranking-delete-btn" @click.stop="deleteRanking(r.id, r.name)" title="删除">
              <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M3 6h18M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2" />
              </svg>
            </button>
          </div>
        </div>
      </div>

      <!-- Main -->
      <div v-if="!selected" class="ranking-main">
        <div class="ranking-select-hint">
          <svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
            <path d="M3 3l7.07 16.97 2.51-7.39 7.39-2.51L3 3z" />
          </svg>
          <h3>选择一个排名</h3>
          <p>从左侧列表选择或创建新排名</p>
        </div>
      </div>

      <div v-else ref="rankingMainRef" class="ranking-main">
        <div class="ranking-header-bar">
          <h3>{{ selected.name }}</h3>
          <span class="ranking-level-count">{{ selected.levels.length }}个等级</span>
        </div>

        <!-- Tiers -->
        <div class="ranking-tiers">
          <div class="ranking-tier-labels">
            <div v-for="level in selected.levels" :key="level.level" class="ranking-tier-label"
              :class="{ editing: editingLevel === level.level }"
              :style="{ backgroundColor: level.color || '#9ca3af', color: contrastColor(level.color) }"
              @dblclick="startEdit(level)" title="双击编辑名称和颜色">
              <template v-if="editingLevel === level.level">
                <div class="ranking-tier-label-edit">
                  <input v-model="editName" class="ranking-tier-label-edit-input" @keydown.enter="saveEdit"
                    @keydown.escape="editingLevel = null" @click.stop />
                  <label class="ranking-tier-label-color-wrap" title="选择颜色">
                    <input v-model="editColor" type="color" class="ranking-tier-label-color-picker" @click.stop />
                    <span class="ranking-tier-label-color-swatch" :style="{ background: editColor }"></span>
                  </label>
                  <button class="ranking-tier-label-edit-save" @click.stop="saveEdit" title="保存">
                    <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor"
                      stroke-width="2.5">
                      <path d="M20 6L9 17l-5-5" />
                    </svg>
                  </button>
                  <button class="ranking-tier-label-edit-cancel" @click.stop="editingLevel = null" title="取消">
                    <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor"
                      stroke-width="2.5">
                      <path d="M18 6L6 18M6 6l12 12" />
                    </svg>
                  </button>
                </div>
              </template>
              <template v-else>
                <span class="color-dot"
                  :style="{ background: contrastColor(level.color, 0.9), borderColor: contrastColor(level.color, 0.4) }"
                  @click.stop="setColorPicker(level.level)" title="更换颜色" />
                <input v-if="colorPickerFor === level.level" type="color" class="ranking-tier-inline-color"
                  :value="level.color || '#9ca3af'" @change="e => inlineColorChange(level, e.target.value)"
                  @click.stop />
                <span class="ranking-tier-label-text">{{ level.name }}</span>
                <span class="ranking-tier-label-hint" :style="{ color: contrastColor(level.color, 0.7) }">双击编辑</span>
              </template>
            </div>
          </div>

          <div class="ranking-tier-rows">
            <div v-for="(level, idx) in selected.levels" :key="level.level" class="ranking-tier-row"
              :data-level-color="level.color || '#9ca3af'" :data-level-level="level.level">
              <div class="ranking-tier-items" @wheel="onTierWheel">
                <div v-if="getGames(level).length === 0" class="ranking-tier-empty">
                  <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
                    <path d="M12 5v14M19 12l-7 7-7-7" />
                  </svg>
                  <span>拖拽游戏到这里</span>
                </div>
                <div v-for="game in getGames(level)" :key="game.id" class="ranking-tier-item"
                  @mousedown="onItemMouseDown($event, game, idx)" @click="previewGame = game" :title="game.name">
                  <img v-if="game.cover && game.cover.startsWith('data:')" :src="game.cover" :alt="game.name" />
                  <div v-else class="ranking-tier-item-no-cover">
                    <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor"
                      stroke-width="1.5">
                      <path
                        d="M6 11h4M8 9v4M15 12h.01M18 11h.01M17.32 5H6.68A4.68 4.68 0 0 0 2 9.68v4.64A4.68 4.68 0 0 0 6.68 19h10.64A4.68 4.68 0 0 0 22 14.32V9.68A4.68 4.68 0 0 0 17.32 5z" />
                    </svg>
                  </div>
                  <button class="ranking-tier-item-remove" @click.stop="removeGame(game.id)" title="移除">
                    <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                      <path d="M18 6L6 18M6 6l12 12" />
                    </svg>
                  </button>
                </div>
              </div>

              <!-- Row gear -->
              <div class="ranking-tier-row-actions">
                <button class="ranking-tier-row-gear"
                  @click.stop="rowMenuLevel = rowMenuLevel === level.level ? null : level.level" title="行操作">
                  <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <path
                      d="M12 15a3 3 0 1 0 0-6 3 3 0 0 0 0 6zM19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 1 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 1 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 1 1-2.83-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 1 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 1 1 2.83-2.83l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 1 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 1 1 2.83 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 1 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z" />
                  </svg>
                </button>
                <div v-if="rowMenuLevel === level.level" class="ranking-tier-row-menu" @click.stop>
                  <button @click="addLevel(level.level, 'above'); rowMenuLevel = null">
                    <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                      <path d="M12 19V5M5 12l7-7 7 7" />
                    </svg>
                    向上新建行
                  </button>
                  <button @click="addLevel(level.level, 'below'); rowMenuLevel = null">
                    <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                      <path d="M12 5v14M19 12l-7 7-7-7" />
                    </svg>
                    向下新建行
                  </button>
                  <button @click="clearLevel(level); rowMenuLevel = null">
                    <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                      <path d="M18 6L6 18M6 6l12 12" />
                    </svg>
                    清空此行
                  </button>
                  <button class="danger" @click="deleteLevel(level); rowMenuLevel = null">
                    <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                      <path d="M3 6h18M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2" />
                    </svg>
                    删除此行
                  </button>
                </div>
              </div>
            </div>
          </div>
        </div>

        <!-- Bangumi 搜索添加 -->
        <div class="ranking-game-pool ranking-bgm-pool">
          <div class="ranking-game-pool-header">
            <span class="ranking-game-pool-title">从 Bangumi 搜索添加</span>
            <span class="bgm-add-card-trigger" @click="openBgmSearch" title="搜索添加游戏">
              <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M12 5v14M5 12h14" />
              </svg>
            </span>
          </div>
          <div class="bgm-notice">
            <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z" />
              <path d="M12 9v4M12 17h.01" />
            </svg>
            <span>Bangumi 数据源需要科学上网才能访问</span>
          </div>
          <div v-if="showBgmSearch" class="bgm-add-section">
            <div class="bgm-add-search-bar">
              <input
                v-model="bgmKeyword"
                type="text"
                class="form-input"
                placeholder="在 Bangumi 搜索游戏..."
                @keydown.enter="handleBgmSearch"
                ref="bgmInputRef"
              />
              <button class="btn btn-primary btn-sm" @click="handleBgmSearch" :disabled="bgmLoading">
                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <path d="M11 19a8 8 0 1 0 0-16 8 8 0 0 0 0 16zM21 21l-4.35-4.35" />
                </svg>
                搜索
              </button>
              <button class="btn btn-secondary btn-sm" @click="cancelBgmSearch">取消</button>
            </div>
            <div v-if="bgmLoading" class="bgm-add-loading">
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" class="spin">
                <path d="M21.5 2v6h-6M2.5 22v-6h6M2 11.5a10 10 0 0 1 18.8-4.3M22 12.5a10 10 0 0 1-18.8 4.2" />
              </svg>
              正在搜索...
            </div>
            <div v-if="bgmResults.length > 0" class="bgm-add-results">
              <div
                v-for="item in bgmResults"
                :key="item.id"
                class="bgm-add-result-item"
                :class="{ 'bgm-add-selecting': bgmSelectingId === item.id }"
              >
                <img v-if="item.image" class="bgm-add-result-cover" :src="item.image" alt="" loading="lazy" />
                <div class="bgm-add-result-info">
                  <div class="bgm-add-result-title">{{ item.name_cn || item.name }}</div>
                  <div v-if="item.name !== item.name_cn" class="bgm-add-result-subtitle">{{ item.name }}</div>
                  <div class="bgm-add-result-meta">
                    <span v-if="item.date">{{ item.date }}</span>
                    <span v-if="item.score > 0" style="color:#fbbf24">★ {{ item.score.toFixed(1) }}</span>
                    <span v-if="item.rank > 0">#{{ item.rank }}</span>
                  </div>
                </div>
                <button
                  class="btn btn-primary btn-sm"
                  @click="handleBgmSelect(item)"
                  :disabled="bgmSelectingId !== null"
                >
                  {{ bgmSelectingId === item.id ? '添加中...' : '添加' }}
                </button>
              </div>
            </div>
            <div v-if="bgmResults.length === 0 && !bgmLoading && bgmSearched" class="bgm-add-empty">
              未找到相关游戏
            </div>
          </div>
        </div>

        <!-- Game Pool - 本地游戏库 -->
        <div class="ranking-game-pool">
          <div class="ranking-game-pool-header">
            <span class="ranking-game-pool-title">游戏库 ({{ filteredGames.length }})</span>
            <div class="ranking-game-pool-search">
              <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M11 19a8 8 0 1 0 0-16 8 8 0 0 0 0 16zM21 21l-4.35-4.35" />
              </svg>
              <input v-model="searchKeyword" type="text" placeholder="搜索游戏..." />
            </div>
          </div>
          <div v-if="filteredGames.length === 0" class="ranking-game-pool-empty">
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
              <path d="M6 11h4M8 9v4M15 12h.01M18 11h.01M17.32 5H6.68A4.68 4.68 0 0 0 2 9.68v4.64A4.68 4.68 0 0 0 6.68 19h10.64A4.68 4.68 0 0 0 22 14.32V9.68A4.68 4.68 0 0 0 17.32 5z" />
            </svg>
            <span>游戏库中暂无未排名的游戏</span>
          </div>
          <div class="ranking-game-pool-list">
            <div v-for="game in filteredGames" :key="game.id" class="ranking-pool-card"
              @mousedown="onPoolMouseDown($event, game)" @click="previewGame = game" :title="game.name">
              <button v-if="game._isVirtual" class="ranking-pool-delete-btn" @click.stop="deleteRankVirtualGame(game)" title="删除虚拟游戏">
                <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <path d="M18 6L6 18M6 6l12 12" />
                </svg>
              </button>
              <div class="ranking-pool-cover">
                <img v-if="game.cover && game.cover.startsWith('data:')" :src="game.cover" :alt="game.name" />
                <div v-else class="no-cover">
                  <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
                    <path
                      d="M6 11h4M8 9v4M15 12h.01M18 11h.01M17.32 5H6.68A4.68 4.68 0 0 0 2 9.68v4.64A4.68 4.68 0 0 0 6.68 19h10.64A4.68 4.68 0 0 0 22 14.32V9.68A4.68 4.68 0 0 0 17.32 5z" />
                  </svg>
                </div>
              </div>
              <div class="ranking-pool-name">{{ game.name }}</div>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Create Modal -->
    <div v-if="showCreateModal" class="modal-overlay" @click.self="showCreateModal = false">
      <div class="modal-content modal-sm">
        <div class="modal-header">
          <h3>新建排名</h3>
        </div>
        <div class="modal-body">
          <div class="form-group">
            <label>排名名称</label>
            <input v-model="newRankingName" type="text" class="form-input" placeholder="例如：纯爱排名、精彩排名"
              @keydown.enter="createRanking" />
          </div>
          <div class="hint">排名将包含5个默认等级：夯、顶级、人上人、NPC、拉完了</div>
        </div>
        <div class="form-actions">
          <button class="btn btn-secondary" @click="showCreateModal = false">取消</button>
          <button class="btn btn-primary" @click="createRanking">创建</button>
        </div>
      </div>
    </div>

    <!-- Preview -->
    <div v-if="previewGame" class="modal-overlay" @click.self="previewGame = null">
      <div class="modal-content modal-lg">
        <div class="ranking-preview">
          <div class="ranking-preview-cover">
            <img v-if="previewGame.cover && previewGame.cover.startsWith('data:')" :src="previewGame.cover"
              :alt="previewGame.name" />
            <div v-else class="ranking-preview-no-cover">
              <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
                <path
                  d="M6 11h4M8 9v4M15 12h.01M18 11h.01M17.32 5H6.68A4.68 4.68 0 0 0 2 9.68v4.64A4.68 4.68 0 0 0 6.68 19h10.64A4.68 4.68 0 0 0 22 14.32V9.68A4.68 4.68 0 0 0 17.32 5z" />
              </svg>
            </div>
          </div>
          <div class="ranking-preview-info">
            <h3>{{ previewGame.name }}</h3>
            <p v-if="previewGame.alias" class="ranking-preview-alias">{{ previewGame.alias }}</p>
            <p v-if="previewGame.description" class="ranking-preview-desc">{{ previewGame.description }}</p>
            <div class="ranking-preview-meta">
              <span>分类: {{ previewGame.category || '未分类' }}</span>
              <span>状态: {{ previewGame.status || '未游玩' }}</span>
              <span>游玩时长: {{ previewGame.play_time || 0 }} min</span>
            </div>
            <div v-if="previewGame.tags && previewGame.tags.length" class="ranking-preview-tags">
              <span v-for="t in previewGame.tags" :key="t" class="card-tag">{{ t }}</span>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Drag float -->
    <div v-if="dragState" ref="dragFloatRef" class="ranking-drag-float" :style="{
      position: 'fixed',
      left: (dragState.initX || 0) - 42 + 'px',
      top: (dragState.initY || 0) - 42 + 'px',
      width: '84px', height: '84px',
      zIndex: 9999, pointerEvents: 'none',
      opacity: 0.92,
      transform: 'rotate(-3deg) scale(1.08)',
      boxShadow: '0 12px 30px rgba(0,0,0,0.35)',
      borderRadius: '8px', overflow: 'hidden',
    }">
      <img v-if="dragState.game.cover && dragState.game.cover.startsWith('data:')" :src="dragState.game.cover" alt=""
        style="width:100%;height:100%;object-fit:cover" />
      <div v-else
        style="width:100%;height:100%;display:flex;align-items:center;justify-content:center;background:var(--bg-tertiary)">
        <svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
          <path
            d="M6 11h4M8 9v4M15 12h.01M18 11h.01M17.32 5H6.68A4.68 4.68 0 0 0 2 9.68v4.64A4.68 4.68 0 0 0 6.68 19h10.64A4.68 4.68 0 0 0 22 14.32V9.68A4.68 4.68 0 0 0 17.32 5z" />
        </svg>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, watch, onMounted, onUnmounted, nextTick, toRaw } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { bridgeState } from './bridge';

/* ---------- 游戏数据（由 React 通过 window.__kurisuGalGames 同步） ---------- */
const allGames = ref(typeof window !== 'undefined' ? (window.__kurisuGalGames || []) : []);

// React 更新游戏数据时通过轮询检测变化（简单可靠）
let _gamePollTimer = null;
onMounted(() => {
  _gamePollTimer = setInterval(() => {
    const g = window.__kurisuGalGames;
    if (g && g !== allGames.value) allGames.value = g;
  }, 200);
});
onUnmounted(() => { clearInterval(_gamePollTimer); });

/* ---------- 工具函数 ---------- */
function hexToRgba(hex, a) {
  hex = (hex || '#9ca3af').replace('#', '');
  const r = parseInt(hex.substring(0, 2), 16) || 0;
  const g = parseInt(hex.substring(2, 4), 16) || 0;
  const b = parseInt(hex.substring(4, 6), 16) || 0;
  return `rgba(${r},${g},${b},${a})`;
}
function contrastColor(hex, alpha) {
  let r, g, b;
  const raw = (hex || '#9ca3af').trim();
  if (raw.startsWith('#')) {
    const h = raw.replace('#', '');
    r = parseInt(h.substring(0, 2), 16) || 0;
    g = parseInt(h.substring(2, 4), 16) || 0;
    b = parseInt(h.substring(4, 6), 16) || 0;
  } else {
    // rgb(r, g, b) 格式
    const m = raw.match(/[\d.]+/g);
    r = m ? parseInt(m[0]) || 0 : 0;
    g = m ? parseInt(m[1]) || 0 : 0;
    b = m ? parseInt(m[2]) || 0 : 0;
  }
  const isLight = (r * 299 + g * 587 + b * 114) / 1000 >= 140;
  if (alpha !== undefined) {
    return isLight ? `rgba(0,0,0,${alpha})` : `rgba(255,255,255,${alpha})`;
  }
  return isLight ? '#000' : '#fff';
}

/* ---------- Toast ---------- */
function toast(type, msg) {
  if (window.__kurisuToast) window.__kurisuToast(type, msg);
}

/* ---------- 状态 ---------- */
const rankings = ref([]);
const selected = ref(null);
const searchKeyword = ref('');
const showCreateModal = ref(false);
const newRankingName = ref('');
const editingLevel = ref(null);
const editName = ref('');
const editColor = ref('#9ca3af');
const colorPickerFor = ref(null);
const previewGame = ref(null);
const rowMenuLevel = ref(null);
const rankingMainRef = ref(null);
const dragFloatRef = ref(null);
const bgmInputRef = ref(null);

/* ---------- Bangumi 搜索添加 ---------- */
const showBgmSearch = ref(false);
const bgmKeyword = ref('');
const bgmResults = ref([]);
const bgmLoading = ref(false);
const bgmSearchKwRef = ref('');
const bgmSelectingId = ref(null);
const bgmSearched = ref(false);

function openBgmSearch() {
  showBgmSearch.value = true;
  bgmResults.value = [];
  bgmKeyword.value = '';
  bgmSearched.value = false;
  nextTick(() => { bgmInputRef.value?.focus(); });
}

function cancelBgmSearch() {
  showBgmSearch.value = false;
  bgmResults.value = [];
  bgmKeyword.value = '';
  bgmSearched.value = false;
}

async function handleBgmSearch() {
  const kw = bgmKeyword.value.trim();
  if (!kw) { toast('warning', '请输入游戏名称'); return; }
  bgmLoading.value = true;
  bgmResults.value = [];
  bgmSearchKwRef.value = kw;
  bgmSearched.value = true;
  try {
    const list = await invoke('search_bangumi', { keyword: kw });
    bgmResults.value = list || [];
    if ((list || []).length === 0) toast('info', '未找到相关游戏');
  } catch (e) {
    toast('error', 'Bangumi 搜索失败: ' + (e?.toString() || ''));
    console.error('Bangumi search error:', e);
  } finally {
    bgmLoading.value = false;
  }
}

async function handleBgmSelect(item) {
  if (bgmSelectingId.value) return;
  bgmSelectingId.value = item.id;

  try {
    // 获取详情和封面
    let fillData = null;
    try {
      fillData = await Promise.race([
        invoke('fetch_bangumi_game', { subjectId: item.id, keyword: bgmSearchKwRef.value || null }),
        new Promise((_, reject) => setTimeout(() => reject(new Error('获取超时(15s)')), 15_000)),
      ]);
    } catch (fetchErr) {
      console.warn('[RankingApp] fetch_bangumi_game failed, using search cache:', fetchErr);
      // 回退：使用搜索结果数据
      let coverBase64 = '';
      const coverUrl = item.image_large || item.image;
      if (coverUrl) {
        try {
          coverBase64 = await Promise.race([
            invoke('download_bangumi_cover', { imageUrl: coverUrl }),
            new Promise((_, reject) => setTimeout(() => reject(new Error('封面下载超时')), 10_000)),
          ]);
        } catch (coverErr) {
          console.warn('[RankingApp] fallback cover download failed:', coverErr);
        }
      }
      fillData = {
        name: item.name || '',
        name_cn: item.name_cn || '',
        summary: item.summary || '',
        cover: coverBase64,
        tags: [],
        date: item.date || '',
      };
    }

    if (!fillData || (!fillData.name && !fillData.name_cn)) {
      toast('warning', '获取的游戏数据为空');
      return;
    }

    // 创建虚拟游戏对象（仅排名内生效，不进入全局游戏库）
    const displayName = fillData.name_cn || fillData.name;
    const alias = (fillData.name_cn && fillData.name && fillData.name_cn !== fillData.name) ? fillData.name : '';
    const timestamp = Date.now();
    const gameId = `bgm_${item.id}_${timestamp}`;

    const gameObj = {
      id: gameId,
      name: displayName,
      alias: alias,
      path: '',
      cover: fillData.cover || '',
      category: '未分类',
      tags: fillData.tags || [],
      status: '未游玩',
      description: fillData.summary || '',
      play_time: 0,
      last_play: null,
      favorite: false,
      screenshots: [],
    };

    const updatedRanking = await invoke('add_rank_virtual_game', {
      rankingId: selected.value.id,
      game: gameObj,
    });

    // 同步到本地状态
    applyUpdate(updatedRanking);
    // applyUpdate 内部会 nextTick(loadVirtualGameCovers)
    // 但我们也在当前 tick 立即加载以确保封面在渲染前准备好
    await loadVirtualGameCovers();

    toast('success', `已添加「${displayName}」（排名虚拟游戏）`);
    cancelBgmSearch();
  } catch (e) {
    const errMsg = e?.toString() || '';
    if (errMsg.includes('已存在于库中')) {
      toast('warning', '该游戏已在游戏库中');
    } else {
      toast('error', '添加失败: ' + errMsg);
    }
    console.error('Bangumi add error:', e);
  } finally {
    bgmSelectingId.value = null;
  }
}

/* ---------- 初始化 ---------- */
onMounted(async () => {
  try {
    const data = await invoke('get_rankings');
    rankings.value = data || [];
    bridgeState.rankings = data || [];
    if (data?.[0]) selectRanking(data[0]);
  } catch (e) { console.warn('加载排名失败:', e); }
});

/* ---------- 计算 ---------- */
const filteredGames = computed(() => {
  const kw = searchKeyword.value.trim().toLowerCase();
  const rankedIds = selected.value?.levels.flatMap(l => l.game_ids) || [];
  // 汇总：真实导入游戏 + 排名内虚拟游戏
  const real = allGames.value.filter(g => !rankedIds.includes(g.id));
  const virt = (selected.value?.virtual_games || [])
    .filter(g => !rankedIds.includes(g.id))
    .map(g => ({ ...g, _isVirtual: true }));
  const combined = [...real, ...virt];
  if (!kw) return combined;
  return combined.filter(g =>
    g.name.toLowerCase().includes(kw) || (g.alias && g.alias.toLowerCase().includes(kw))
  );
});

async function loadVirtualGameCovers() {
  const ranking = selected.value;
  if (!ranking?.virtual_games?.length) return;
  const needLoad = [];
  for (const g of ranking.virtual_games) {
    if (g.cover && !g.cover.startsWith('data:')) needLoad.push(g);
  }
  if (needLoad.length === 0) return;
  try {
    const ids = needLoad.map(g => g.id);
    const coversMap = await invoke('get_game_covers', { gameIds: ids });
    // 通过 selected.value（Vue 响应式代理）写入，确保模板能感知变化
    const s = selected.value;
    if (!s) return;
    for (const g of s.virtual_games) {
      if (coversMap[g.id]) g.cover = coversMap[g.id];
    }
  } catch (e) { console.warn('loadVirtualGameCovers failed:', e); }
}

function getGames(level) {
  return level.game_ids.map(id => {
    const real = allGames.value.find(g => g.id === id);
    if (real) return real;
    return (selected.value?.virtual_games || []).find(g => g.id === id);
  }).filter(Boolean);
}

/* ---------- 排名操作 ---------- */
function selectRanking(r) {
  selected.value = r;
  bridgeState.selectedRanking = r;
  nextTick(() => loadVirtualGameCovers());
}
async function createRanking() {
  const name = newRankingName.value.trim();
  if (!name) { toast('warning', '请输入排名名称'); return; }
  try {
    const ranking = await invoke('create_ranking', { name });
    rankings.value = [...rankings.value, ranking];
    bridgeState.rankings = rankings.value;
    selected.value = ranking;
    bridgeState.selectedRanking = ranking;
    newRankingName.value = '';
    showCreateModal.value = false;
    toast('success', `排名「${name}」已创建`);
  } catch (e) { toast('error', e.toString()); }
}
async function deleteRanking(id, name) {
  if (!confirm(`确定删除排名「${name}」？`)) return;
  try {
    await invoke('delete_ranking', { id });
    rankings.value = rankings.value.filter(r => r.id !== id);
    bridgeState.rankings = rankings.value;
    if (selected.value?.id === id) { selected.value = null; bridgeState.selectedRanking = null; }
    toast('success', `排名「${name}」已删除`);
  } catch (e) { toast('error', e.toString()); }
}

/* ---------- 等级操作 ---------- */
async function addLevel(insertAfter, dir) {
  if (!selected.value) return;
  try {
    const updated = await invoke('add_rank_level', {
      rankingId: selected.value.id,
      name: '新等级', color: '#9ca3af',
      insertAfterLevel: dir === 'above' ? insertAfter - 1 : insertAfter,
    });
    applyUpdate(updated);
    toast('success', '已添加新等级');
  } catch (e) { toast('error', e.toString()); }
}
async function deleteLevel(level) {
  if (!selected.value || selected.value.levels.length <= 1) { toast('warning', '至少保留一个等级'); return; }
  if (!confirm(`确定删除等级「${level.name}」？该等级中的游戏将回到游戏库。`)) return;
  try {
    const updated = await invoke('delete_rank_level', { rankingId: selected.value.id, level: level.level });
    applyUpdate(updated);
    toast('success', `已删除等级「${level.name}」`);
  } catch (e) { toast('error', e.toString()); }
}
async function clearLevel(level) {
  if (!selected.value) return;
  if (level.game_ids.length === 0) { toast('info', '该等级没有游戏'); return; }
  if (!confirm(`确定清空等级「${level.name}」中的所有游戏？游戏将回到游戏库。`)) return;
  try {
    const updated = await invoke('clear_rank_level', { rankingId: selected.value.id, level: level.level });
    applyUpdate(updated);
    toast('success', `已清空等级「${level.name}」`);
  } catch (e) { toast('error', e.toString()); }
}
function startEdit(level) {
  editingLevel.value = level.level;
  editName.value = level.name;
  editColor.value = level.color || '#9ca3af';
}
async function saveEdit() {
  if (!selected.value || editingLevel.value === null) return;
  if (!editName.value.trim()) { toast('warning', '等级名称不能为空'); return; }
  try {
    const updated = await invoke('update_rank_level', {
      rankingId: selected.value.id,
      level: editingLevel.value,
      name: editName.value.trim(),
      color: editColor.value,
    });
    applyUpdate(updated);
    toast('success', '已保存');
  } catch (e) { toast('error', e.toString()); }
  editingLevel.value = null;
}
async function inlineColorChange(level, newColor) {
  if (!selected.value) return;
  try {
    const updated = await invoke('update_rank_level', {
      rankingId: selected.value.id, level: level.level, name: level.name, color: newColor,
    });
    applyUpdate(updated);
    colorPickerFor.value = null;
  } catch (e) { toast('error', e.toString()); }
}
function setColorPicker(level) { colorPickerFor.value = level; }
async function removeGame(gameId) {
  if (!selected.value) return;
  try {
    const updated = await invoke('remove_game_from_rank', { rankingId: selected.value.id, gameId });
    applyUpdate(updated);
    toast('success', '已从排名中移除');
  } catch (e) { toast('error', e.toString()); }
}
async function deleteRankVirtualGame(game) {
  if (!selected.value) return;
  if (!confirm(`确定从排名中删除虚拟游戏「${game.name}」？`)) return;
  try {
    const updated = await invoke('remove_rank_virtual_game', {
      rankingId: selected.value.id,
      gameId: game.id,
    });
    applyUpdate(updated);
    toast('success', `已删除虚拟游戏「${game.name}」`);
  } catch (e) { toast('error', '删除失败: ' + (e?.toString() || '')); }
}

function applyUpdate(updated) {
  selected.value = updated;
  bridgeState.selectedRanking = updated;
  const idx = rankings.value.findIndex(r => r.id === updated.id);
  if (idx >= 0) {
    rankings.value[idx] = updated;
    bridgeState.rankings = [...rankings.value];
  }
  nextTick(() => loadVirtualGameCovers());
}

/* ---------- 滚轮横向滚动 ---------- */
function onTierWheel(e) {
  if (e.deltaY !== 0) {
    e.preventDefault();
    e.currentTarget.scrollLeft += e.deltaY * 0.5;
  }
}

/* ---------- DRAG & DROP ============ */
const dragState = ref(null);
const dragPendingRef = ref(null);
const autoScrollRef = ref(null);
const dropTargetRef = ref(null);

function setHighlight(row, color, on) {
  if (on) {
    row.setAttribute('data-drag-over', '');
    row.style.setProperty('--drag-bg', hexToRgba(color, 0.12));
    row.style.setProperty('--drag-border', color);
  } else {
    row.removeAttribute('data-drag-over');
    row.style.removeProperty('--drag-bg');
    row.style.removeProperty('--drag-border');
  }
}
function clearHighlights() {
  document.querySelectorAll('.ranking-tier-row[data-drag-over]').forEach(r => setHighlight(r, '', false));
}
function startAuto(dir) {
  const c = rankingMainRef.value;
  if (!c || autoScrollRef.value) return;
  const scroll = () => { c.scrollTop += dir * 10; autoScrollRef.value = requestAnimationFrame(scroll); };
  autoScrollRef.value = requestAnimationFrame(scroll);
}
function stopAuto() {
  if (autoScrollRef.value) { cancelAnimationFrame(autoScrollRef.value); autoScrollRef.value = null; }
}
function autoScroll(clientY) {
  const c = rankingMainRef.value;
  if (!c) return;
  const r = c.getBoundingClientRect();
  const t = 100;
  if (clientY < r.top + t) startAuto(-1);
  else if (clientY > r.bottom - t) startAuto(1);
  else stopAuto();
}

function onItemMouseDown(e, game, tierIdx) {
  if (e.button !== 0) return;
  e.preventDefault();
  dragPendingRef.value = { game: toRaw(game), sourceTierIdx: tierIdx, startX: e.clientX, startY: e.clientY };
}
function onPoolMouseDown(e, game) {
  if (e.button !== 0) return;
  e.preventDefault();
  dragPendingRef.value = { game: toRaw(game), sourceTierIdx: null, startX: e.clientX, startY: e.clientY };
}

function handleMouseMove(e) {
  if (!dragPendingRef.value && !dragState.value) return;
  // 待机 → 激活
  if (dragPendingRef.value && !dragState.value) {
    const p = dragPendingRef.value;
    if (Math.abs(e.clientX - p.startX) < 4 && Math.abs(e.clientY - p.startY) < 4) return;
    dragState.value = { game: p.game, sourceTierIdx: p.sourceTierIdx, initX: e.clientX, initY: e.clientY };
    dragPendingRef.value = null;
    nextTick(() => {
      if (dragFloatRef.value) {
        dragFloatRef.value.style.left = (e.clientX - 42) + 'px';
        dragFloatRef.value.style.top = (e.clientY - 42) + 'px';
      }
    });
    return;
  }
  // 活跃拖拽
  if (dragFloatRef.value) {
    dragFloatRef.value.style.left = (e.clientX - 42) + 'px';
    dragFloatRef.value.style.top = (e.clientY - 42) + 'px';
  }
  // 目标行检测
  const row = document.elementFromPoint(e.clientX, e.clientY)?.closest('.ranking-tier-row');
  const newLvl = row ? parseInt(row.dataset.levelLevel) : null;
  if (dropTargetRef.value !== newLvl) {
    clearHighlights();
    dropTargetRef.value = newLvl;
    if (newLvl != null && row) setHighlight(row, row.dataset.levelColor || '#9ca3af', true);
  }
  autoScroll(e.clientY);
}

async function handleMouseUp(e) {
  if (dragPendingRef.value) { dragPendingRef.value = null; return; }
  if (!dragState.value) return;
  const state = dragState.value;
  dragState.value = null;
  clearHighlights();
  stopAuto();
  dropTargetRef.value = null;

  const row = document.elementFromPoint(e.clientX, e.clientY)?.closest('.ranking-tier-row');
  if (!row || !selected.value) return;
  const levelLvl = parseInt(row.dataset.levelLevel);
  const levelIdx = selected.value.levels.findIndex(l => l.level === levelLvl);
  if (levelIdx === -1 || levelIdx === state.sourceTierIdx) return;
  const level = selected.value.levels[levelIdx];
  try {
    const updated = await invoke('set_game_rank', {
      rankingId: selected.value.id,
      gameId: state.game.id,
      level: level.level,
    });
    applyUpdate(updated);
    toast('success', `已将「${state.game.name}」放入「${level.name}」`);
  } catch (err) {
    console.error('set_game_rank failed:', err);
    toast('error', err.toString());
  }
}

onMounted(() => {
  document.addEventListener('mousemove', handleMouseMove);
  document.addEventListener('mouseup', handleMouseUp);
});
onUnmounted(() => {
  document.removeEventListener('mousemove', handleMouseMove);
  document.removeEventListener('mouseup', handleMouseUp);
  stopAuto();
});

/* ---------- 点击外部关闭弹窗 ---------- */
watch([colorPickerFor, rowMenuLevel], () => {
  if (colorPickerFor.value !== null || rowMenuLevel.value !== null) {
    const handler = () => { colorPickerFor.value = null; rowMenuLevel.value = null; };
    setTimeout(() => document.addEventListener('click', handler, { once: true }), 0);
  }
});
</script>
<style scoped>
/* Scoped: keep existing CSS from components.css */
</style>
