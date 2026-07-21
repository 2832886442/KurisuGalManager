// main.js - CleanGal Galgame 管理器 (优化版)
import { debounce, showToast, escapeHtml, setLoading, formatError } from './js/utils.js';

const invoke = window.__TAURI__.core.invoke;
const listen = window.__TAURI__.event.listen;

// ======================== 状态管理 ========================

let gameData = { games: [] };
let currentFilter = {
  category: 'all',
  tag: null,
  search: '',
  view: 'grid',
};

// 运行时状态：正在运行的游戏 ID 集合
let runningGameIds = new Set();

// 批量选择模式
let batchMode = false;
let selectedGameIds = new Set();

// 分页
let currentPage = 1;
let pageSize = parseInt(localStorage.getItem('pageSize') || '24', 10);

// 自定义分类（独立于游戏数据存储）
let customCategories = new Set(JSON.parse(localStorage.getItem('customCategories') || '[]'));


// ======================== 数据层 ========================

function saveCustomCategories() {
  localStorage.setItem('customCategories', JSON.stringify([...customCategories]));
}

function syncCategoriesFromGames() {
  gameData.games.forEach(g => {
    if (g.category && g.category !== '未分类') {
      customCategories.add(g.category);
    }
  });
  saveCustomCategories();
}

function getAllCategories() {
  const cats = {};
  gameData.games.forEach(g => {
    cats[g.category] = (cats[g.category] || 0) + 1;
  });
  // 合并自定义分类（游戏数为0的分类也会显示）
  customCategories.forEach(c => {
    if (!cats[c]) cats[c] = 0;
  });
  return cats;
}

function getCategories() {
  return getAllCategories();
}

function getTags() {
  const tags = {};
  gameData.games.forEach(g => {
    g.tags.forEach(t => { tags[t] = (tags[t] || 0) + 1; });
  });
  return tags;
}

function getFilteredGames() {
  let list = gameData.games;
  if (currentFilter.category !== 'all') {
    list = list.filter(g => g.category === currentFilter.category);
  }
  if (currentFilter.tag) {
    list = list.filter(g => g.tags.includes(currentFilter.tag));
  }
  if (currentFilter.search.trim()) {
    const kw = currentFilter.search.trim().toLowerCase();
    list = list.filter(g =>
      g.name.toLowerCase().includes(kw) ||
      (g.alias && g.alias.toLowerCase().includes(kw))
    );
  }
  return list;
}

// ======================== 渲染引擎 ========================

function renderPagination(totalItems, page, size) {
  const container = document.getElementById('pagination');
  const totalPages = Math.max(1, Math.ceil(totalItems / size));
  if (totalPages <= 1) {
    container.style.display = 'none';
    return;
  }
  container.style.display = 'flex';

  const startItem = totalItems === 0 ? 0 : (page - 1) * size + 1;
  const endItem = Math.min(page * size, totalItems);

  container.innerHTML = `
    <span class="page-info">${startItem}-${endItem} / ${totalItems}</span>
    <button class="page-btn" data-page="1" ${page === 1 ? 'disabled' : ''}>«</button>
    <button class="page-btn" data-page="${page - 1}" ${page === 1 ? 'disabled' : ''}>‹</button>
    <span class="page-current">${page} / ${totalPages}</span>
    <button class="page-btn" data-page="${page + 1}" ${page >= totalPages ? 'disabled' : ''}>›</button>
    <button class="page-btn" data-page="${totalPages}" ${page >= totalPages ? 'disabled' : ''}>»</button>
    <select class="page-size-select" id="page-size-select">
      <option value="12" ${size === 12 ? 'selected' : ''}>12/页</option>
      <option value="24" ${size === 24 ? 'selected' : ''}>24/页</option>
      <option value="48" ${size === 48 ? 'selected' : ''}>48/页</option>
      <option value="96" ${size === 96 ? 'selected' : ''}>96/页</option>
    </select>
  `;

  // 绑定事件
  container.querySelectorAll('.page-btn:not([disabled])').forEach(btn => {
    btn.addEventListener('click', () => gotoPage(parseInt(btn.dataset.page)));
  });
  const psSelect = document.getElementById('page-size-select');
  if (psSelect) {
    psSelect.addEventListener('change', (e) => {
      pageSize = parseInt(e.target.value, 10);
      localStorage.setItem('pageSize', pageSize);
      currentPage = 1;
      renderGames();
    });
  }
}

function gotoPage(page) {
  const filtered = getFilteredGames();
  const totalPages = Math.max(1, Math.ceil(filtered.length / pageSize));
  currentPage = Math.max(1, Math.min(page, totalPages));
  renderGames();
  document.getElementById('game-grid').scrollTop = 0;
}

function applyFilter() {
  currentPage = 1;
  renderSidebar();
  renderGames();
}

function renderSidebar() {
  const cats = getCategories();
  const listEl = document.getElementById('category-list');
  listEl.innerHTML = '';

  const allItem = document.createElement('li');
  allItem.className = 'nav-item' + (currentFilter.category === 'all' ? ' active' : '');
  allItem.dataset.category = 'all';
  allItem.innerHTML = `
    <span class="nav-icon">📚</span>
    <span class="nav-label">全部游戏</span>
    <span class="nav-count" id="count-all">${gameData.games.length}</span>
  `;
  allItem.addEventListener('click', () => {
    currentFilter.category = 'all'; currentFilter.tag = null;
    exitBatchMode(); applyFilter();
  });
  listEl.appendChild(allItem);

  Object.keys(cats).sort().forEach(cat => {
    const li = document.createElement('li');
    li.className = 'nav-item' + (currentFilter.category === cat ? ' active' : '');
    li.dataset.category = cat;
    li.innerHTML = `
      <span class="nav-icon">📁</span>
      <span class="nav-label">${escapeHtml(cat)}</span>
      <span class="nav-count">${cats[cat]}</span>
    `;
    li.addEventListener('click', () => {
      currentFilter.category = cat; currentFilter.tag = null;
      exitBatchMode(); applyFilter();
    });
    listEl.appendChild(li);
  });
  renderTags();
}

function renderTags() {
  const tags = getTags();
  const cloud = document.getElementById('tag-cloud');
  cloud.innerHTML = '';
  const sorted = Object.keys(tags).sort();
  if (sorted.length === 0) {
    cloud.innerHTML = '<span style="font-size:0.75rem;color:var(--text-muted);padding:4px 0;">暂无标签</span>';
    return;
  }
  sorted.forEach(tag => {
    const span = document.createElement('span');
    span.className = 'tag-item' + (currentFilter.tag === tag ? ' active' : '');
    span.textContent = `${tag} (${tags[tag]})`;
    span.addEventListener('click', () => {
      if (currentFilter.tag === tag) {
        currentFilter.tag = null;
      } else {
        currentFilter.tag = tag; currentFilter.category = 'all';
      }
      exitBatchMode(); applyFilter();
    });
    cloud.appendChild(span);
  });
}

function renderGames() {
  const grid = document.getElementById('game-grid');
  const empty = document.getElementById('empty-state');
  const filtered = getFilteredGames();

  document.getElementById('page-title').textContent =
    currentFilter.category !== 'all' ? currentFilter.category :
      currentFilter.tag ? `#${currentFilter.tag}` : '全部游戏';
  document.getElementById('game-count').textContent = `${filtered.length} 款`;

  if (filtered.length === 0) {
    grid.style.display = 'none'; empty.style.display = 'flex';
    renderPagination(0, 1, pageSize);
    return;
  }
  grid.style.display = 'grid'; empty.style.display = 'none';
  grid.className = `game-grid ${currentFilter.view === 'grid' ? 'grid-view' : 'list-view'}`;

  // 分页计算
  const totalPages = Math.ceil(filtered.length / pageSize);
  if (currentPage > totalPages) currentPage = totalPages;
  const startIdx = (currentPage - 1) * pageSize;
  const pagedGames = filtered.slice(startIdx, startIdx + pageSize);

  grid.innerHTML = pagedGames.map(game => {
    const isRunning = runningGameIds.has(game.id);
    const statusMap = { '未游玩': 'status-unplayed', '游玩中': 'status-playing', '已通关': 'status-completed', '搁置': 'status-shelved' };

    // 运行时显示「游戏中」，否则显示持久化状态
    const displayStatus = isRunning ? '游戏中' : game.status;
    const statusClass = isRunning ? 'status-running' : (statusMap[game.status] || '');

    const coverHtml = game.cover && game.cover.startsWith('data:')
      ? `<img src="${game.cover}" alt="${escapeHtml(game.name)}" loading="lazy" />`
      : `<div class="no-cover">🎮</div>`;

    const checked = selectedGameIds.has(game.id) ? 'checked' : '';
    const batchClass = batchMode ? 'batch-mode' : '';
    const selClass = selectedGameIds.has(game.id) ? 'selected' : '';

    return `
      <div class="game-card ${batchClass} ${selClass}" data-id="${game.id}">
        <div class="card-cover">
          ${coverHtml}
          <input type="checkbox" class="card-checkbox" ${checked} data-id="${game.id}" />
          ${game.favorite ? '<span class="favorite-badge">⭐</span>' : ''}
          <span class="status-badge ${statusClass}">${escapeHtml(displayStatus)}</span>
          ${isRunning ? `
          <div class="card-play-overlay" data-id="${game.id}" title="游戏正在运行中">
            <span class="play-indicator"></span>
          </div>` : `
          <button class="card-launch-btn" data-id="${game.id}" title="启动游戏">▶</button>
          <div class="status-quick-switch" data-id="${game.id}">
            <button class="status-quick-btn qs-playing" data-status="游玩中">游玩中</button>
            <button class="status-quick-btn qs-completed" data-status="已通关">通关</button>
            <button class="status-quick-btn qs-shelved" data-status="搁置">搁置</button>
          </div>`}
        </div>
        <div class="card-body">
          <h4 class="card-title">${escapeHtml(game.name)}</h4>
          ${game.alias ? `<p class="card-alias">${escapeHtml(game.alias)}</p>` : ''}
          <div class="card-tags">
            ${game.tags.slice(0, 3).map(t => `<span class="card-tag">${escapeHtml(t)}</span>`).join('')}
            ${game.tags.length > 3 ? `<span class="card-tag">+${game.tags.length - 3}</span>` : ''}
          </div>
          <div class="card-footer">
            <span class="card-category">${escapeHtml(game.category)}</span>
            <span class="card-playtime">${game.play_time > 0 ? game.play_time + 'min' : ''}</span>
          </div>
        </div>
      </div>
    `;
  }).join('');

  renderPagination(filtered.length, currentPage, pageSize);

  // 事件委托（省略，原样保持...）
  grid.querySelectorAll('.card-checkbox').forEach(cb => {
    cb.addEventListener('click', (e) => {
      e.stopPropagation();
      const id = cb.dataset.id;
      if (cb.checked) selectedGameIds.add(id);
      else selectedGameIds.delete(id);
      updateBatchUI();
      renderGames(); // 刷新选中样式
    });
  });

  // 卡片上的启动按钮（单击启动，不打开详情）
  grid.querySelectorAll('.card-launch-btn').forEach(btn => {
    btn.addEventListener('click', (e) => {
      e.stopPropagation();
      launchGameFromCard(btn.dataset.id);
    });
  });

  grid.querySelectorAll('.status-quick-switch .status-quick-btn').forEach(btn => {
    btn.addEventListener('click', (e) => {
      e.stopPropagation();
      const id = btn.parentElement.dataset.id;
      const status = btn.dataset.status;
      quickChangeStatus(id, status);
    });
  });

  grid.querySelectorAll('.game-card').forEach(card => {
    card.addEventListener('click', () => {
      if (batchMode) {
        const cb = card.querySelector('.card-checkbox');
        if (cb) {
          cb.checked = !cb.checked;
          const id = card.dataset.id;
          if (cb.checked) selectedGameIds.add(id);
          else selectedGameIds.delete(id);
          updateBatchUI();
          renderGames();
        }
      } else {
        showDetail(card.dataset.id);
      }
    });
    // 双击卡片直接启动游戏
    card.addEventListener('dblclick', (e) => {
      if (batchMode) return;
      const launchBtn = card.querySelector('.card-launch-btn');
      if (launchBtn) {
        e.stopPropagation();
        launchGameFromCard(launchBtn.dataset.id);
      }
    });
  });
}

// ======================== 运行时状态 ========================

async function launchGameFromCard(gameId) {
  const game = gameData.games.find(g => g.id === gameId);
  if (!game) return;
  if (runningGameIds.has(gameId)) {
    showToast('info', '游戏已在运行中');
    return;
  }
  try {
    setLoading('正在启动...');
    await invoke('launch_game', { gameId, path: game.path });
    runningGameIds.add(gameId);
    renderGames(); // 立即刷新卡片显示「游戏中」
    showToast('success', `「${game.name}」已启动`);
  } catch (e) {
    showToast('error', formatError('启动游戏', e));
  } finally {
    setLoading(null);
  }
}

async function quickChangeStatus(gameId, status) {
  try {
    const data = await invoke('quick_update_status', { gameId, status });
    gameData.games = data.games || [];
    applyFilter();
    showToast('success', `状态已更新为「${status}」`);
  } catch (e) {
    showToast('error', formatError('更新状态', e));
  }
}

// ======================== 批量选择 ========================

function enterBatchMode() {
  batchMode = true;
  selectedGameIds.clear();
  document.getElementById('batch-toggle').classList.add('active');
  document.getElementById('batch-bar').style.display = 'flex';
  updateBatchUI();
  refreshBatchCategorySelect();
  renderGames();
}

function exitBatchMode() {
  batchMode = false;
  selectedGameIds.clear();
  document.getElementById('batch-toggle').classList.remove('active');
  document.getElementById('batch-bar').style.display = 'none';
  renderGames();
}

function toggleBatchMode() {
  batchMode ? exitBatchMode() : enterBatchMode();
}

function updateBatchUI() {
  document.getElementById('batch-info').textContent = `已选 ${selectedGameIds.size} 项`;
}

function refreshBatchCategorySelect() {
  const select = document.getElementById('batch-category-select');
  select.innerHTML = '<option value="">-- 移动到分类 --</option>';
  Object.keys(getCategories()).sort().forEach(c => {
    select.innerHTML += `<option value="${escapeHtml(c)}">${escapeHtml(c)}</option>`;
  });
}

async function batchApplyCategory() {
  const select = document.getElementById('batch-category-select');
  const category = select.value;
  if (!category || selectedGameIds.size === 0) {
    showToast('warning', '请选择目标分类并至少勾选一个游戏');
    return;
  }
  try {
    setLoading('批量移动中...');
    const data = await invoke('batch_update_category', {
      gameIds: Array.from(selectedGameIds),
      category,
    });
    gameData.games = data.games || [];
    exitBatchMode();
    applyFilter();
    showToast('success', `已将 ${selectedGameIds.size} 个游戏移动到「${category}」`);
  } catch (e) {
    showToast('error', formatError('批量操作', e));
  } finally {
    setLoading(null);
  }
}

function toggleSelectAll() {
  const filtered = getFilteredGames();
  const btn = document.getElementById('batch-select-all');
  if (selectedGameIds.size === filtered.length && selectedGameIds.size > 0) {
    // 全部取消
    selectedGameIds.clear();
    btn.textContent = '全选当前';
  } else {
    // 全部选中
    filtered.forEach(g => selectedGameIds.add(g.id));
    btn.textContent = '取消全选';
  }
  updateBatchUI();
  renderGames();
}

function initBatchSystem() {
  document.getElementById('batch-toggle').addEventListener('click', toggleBatchMode);
  document.getElementById('batch-cancel').addEventListener('click', exitBatchMode);
  document.getElementById('batch-apply').addEventListener('click', batchApplyCategory);
  document.getElementById('batch-select-all').addEventListener('click', toggleSelectAll);
}

// ======================== 后端操作 ========================

async function loadGames() {
  try {
    setLoading('加载游戏库...');
    const data = await invoke('get_games');
    gameData.games = data.games || [];
    syncCategoriesFromGames();
    // 先加载封面再渲染
    await loadCovers();
    applyFilter();
  } catch (e) {
    showToast('error', formatError('加载游戏数据', e));
  } finally {
    setLoading(null);
  }
}

async function loadCovers() {
  const ids = gameData.games
    .filter(g => g.cover && !g.cover.startsWith('data:'))
    .map(g => g.id);
  if (ids.length === 0) return;
  try {
    const covers = await invoke('get_game_covers', { gameIds: ids });
    for (const game of gameData.games) {
      if (covers[game.id]) {
        game.cover = covers[game.id];
      }
    }
  } catch (e) {
    console.warn('封面加载失败:', e);
  }
}

async function addGameToBackend(game) {
  try {
    setLoading('添加游戏...');
    const data = await invoke('add_game', { game });
    gameData.games = data.games || [];
    applyFilter();
    showToast('success', '游戏添加成功');
  } catch (e) {
    showToast('error', formatError('添加游戏', e));
    throw e;
  } finally {
    setLoading(null);
  }
}

async function updateGameToBackend(game) {
  try {
    setLoading('更新游戏...');
    const data = await invoke('update_game', { game });
    gameData.games = data.games || [];
    applyFilter();
    showToast('success', '游戏更新成功');
  } catch (e) {
    showToast('error', formatError('更新游戏', e));
    throw e;
  } finally {
    setLoading(null);
  }
}

async function deleteGameFromBackend(id) {
  try {
    setLoading('删除游戏...');
    const data = await invoke('delete_game', { id });
    gameData.games = data.games || [];
    applyFilter();
    showToast('success', '游戏已删除');
  } catch (e) {
    showToast('error', formatError('删除游戏', e));
    throw e;
  } finally {
    setLoading(null);
  }
}

// ======================== 详情弹窗 ========================

function showDetail(id) {
  const game = gameData.games.find(g => g.id === id);
  if (!game) return;
  const overlay = document.getElementById('detail-overlay');
  const coverSrc = game.cover && game.cover.startsWith('data:') ? game.cover : '';
  const isRunning = runningGameIds.has(game.id);

  document.getElementById('detail-container').innerHTML = `
    <div class="detail-header">
      <div class="detail-cover">
        ${coverSrc ? `<img src="${coverSrc}" alt="${escapeHtml(game.name)}" />` : '<div class="no-cover">🎮</div>'}
      </div>
      <div class="detail-info">
        <h2>${escapeHtml(game.name)}${isRunning ? ' <span style="color:#00a86b;font-size:0.7em;">(游戏中)</span>' : ''}</h2>
        ${game.alias ? `<p class="detail-alias">别名：${escapeHtml(game.alias)}</p>` : ''}
        <p class="detail-category">分类：${escapeHtml(game.category)}</p>
        <div class="detail-tags">${game.tags.map(t => `<span class="tag-item">${escapeHtml(t)}</span>`).join('')}</div>
        <p class="detail-status">状态：<span class="${game.status}">${isRunning ? '游戏中' : escapeHtml(game.status)}</span></p>
        <p class="detail-playtime">游玩时长：${game.play_time} 分钟</p>
        <p class="detail-lastplay">上次游玩：${game.last_play || '从未'}</p>
        <div class="detail-actions">
          ${isRunning
      ? '<button class="btn-primary" disabled>游戏运行中...</button>'
      : '<button class="btn-primary" id="detail-launch">🚀 启动游戏</button>'}
          <button class="btn-edit" id="detail-edit">✏️ 编辑</button>
          <button class="btn-danger" id="detail-delete">🗑️ 删除</button>
        </div>
        ${!isRunning ? `
        <div class="detail-status-quick" style="margin-top:10px;display:flex;gap:6px;">
          <span style="font-size:0.8rem;color:var(--text-muted);line-height:2;">快速切换状态：</span>
          <button class="status-quick-btn qs-playing" data-status="游玩中">游玩中</button>
          <button class="status-quick-btn qs-completed" data-status="已通关">已通关</button>
          <button class="status-quick-btn qs-shelved" data-status="搁置">搁置</button>
        </div>` : ''}
      </div>
    </div>
    <div class="detail-body">
      <h4>简介</h4>
      <p>${escapeHtml(game.description || '暂无简介')}</p>
      <h4>启动路径</h4>
      <code>${escapeHtml(game.path)}</code>
    </div>
  `;

  overlay.style.display = 'flex';

  const launchBtn = document.getElementById('detail-launch');
  if (launchBtn) {
    launchBtn.addEventListener('click', async () => {
      try {
        setLoading('正在启动游戏...');
        await invoke('launch_game', { gameId: game.id, path: game.path });
        runningGameIds.add(game.id);
        applyFilter();
        showToast('success', '游戏已启动（关闭后将自动记录游玩时间）');
        closeDetail();
      } catch (err) {
        showToast('error', formatError('启动游戏', err));
      } finally {
        setLoading(null);
      }
    });
  }

  document.getElementById('detail-edit').addEventListener('click', () => {
    closeDetail(); openEditGame(game.id);
  });
  document.getElementById('detail-delete').addEventListener('click', async () => {
    if (confirm(`确定要删除游戏"${game.name}"吗？`)) {
      await deleteGameFromBackend(game.id); closeDetail();
    }
  });

  // 详情页快速状态切换
  document.querySelectorAll('.detail-status-quick .status-quick-btn').forEach(btn => {
    btn.addEventListener('click', async () => {
      await quickChangeStatus(game.id, btn.dataset.status);
      closeDetail();
    });
  });
}

function closeDetail() {
  document.getElementById('detail-overlay').style.display = 'none';
}

// ======================== Bangumi 查询 ========================

let bgmResultsCache = [];
let bgmSearchAborted = false;
let bgmLastKeyword = '';

function initBangumiSearch() {
  const input = document.getElementById('bgm-search-input');
  const btn = document.getElementById('bgm-search-btn');
  const cancelBtn = document.getElementById('bgm-search-cancel');
  const directIdBtn = document.getElementById('bgm-direct-id-btn');
  const directIdInput = document.getElementById('bgm-direct-id-input');

  btn.addEventListener('click', () => searchBangumi());
  cancelBtn.addEventListener('click', () => cancelBangumiSearch());
  input.addEventListener('keydown', (e) => {
    if (e.key === 'Enter') {
      e.preventDefault();
      searchBangumi();
    }
  });

  // 直接输入 ID 获取
  directIdBtn.addEventListener('click', () => {
    const idVal = directIdInput.value.trim();
    if (!idVal) { showToast('warning', '请输入 Bangumi 条目 ID'); return; }
    const id = parseInt(idVal, 10);
    if (isNaN(id) || id <= 0) { showToast('warning', '请输入有效的数字 ID'); return; }
    selectBangumiResult(id);
  });
  directIdInput.addEventListener('keydown', (e) => {
    if (e.key === 'Enter') { e.preventDefault(); directIdBtn.click(); }
  });
}

function cancelBangumiSearch() {
  bgmSearchAborted = true;
  document.getElementById('bgm-search-cancel').style.display = 'none';
  document.getElementById('bgm-results').style.display = 'none';
  document.getElementById('bgm-direct-id').style.display = 'none';
  setLoading(null);
  showToast('info', '已取消搜索');
}

async function searchBangumi() {
  const keyword = document.getElementById('bgm-search-input').value.trim();
  if (!keyword) {
    showToast('warning', '请输入游戏名称');
    return;
  }

  const resultsEl = document.getElementById('bgm-results');
  const cancelBtn = document.getElementById('bgm-search-cancel');
  const directIdEl = document.getElementById('bgm-direct-id');

  bgmSearchAborted = false;
  bgmLastKeyword = keyword;
  cancelBtn.style.display = '';
  directIdEl.style.display = 'none';
  resultsEl.style.display = 'block';
  resultsEl.innerHTML = '<div class="bgm-loading">🔍 正在搜索「' + escapeHtml(keyword) + '」...</div>';

  try {
    const list = await invoke('search_bangumi', { keyword });

    // 如果用户点了取消，不更新结果
    if (bgmSearchAborted) return;

    bgmResultsCache = list || [];

    if (bgmResultsCache.length === 0) {
      resultsEl.innerHTML = '<div class="bgm-empty">未找到匹配的游戏。您可以尝试：<br>1. 更换关键词重试<br>2. 在 Bangumi 网站找到目标游戏后，直接输入条目 ID 获取数据</div>';
      directIdEl.style.display = '';
      return;
    }

    resultsEl.innerHTML = `
      <div class="bgm-result-count" style="font-size:0.8rem;color:var(--text-muted);padding:4px 8px;">
        找到 ${bgmResultsCache.length} 条游戏结果（显示前 50 条，请使用精确关键词缩小范围）
      </div>
      ${bgmResultsCache.map((item, idx) => `
      <div class="bgm-result-item" data-index="${idx}">
        ${item.image ? `<img class="bgm-result-cover" src="${escapeHtml(item.image)}" alt="" loading="lazy" />` : ''}
        <div class="bgm-result-info">
          <div class="bgm-result-title">${escapeHtml(item.name_cn || item.name)}</div>
          <div class="bgm-result-subtitle">${escapeHtml(item.name !== item.name_cn ? item.name : '')}</div>
          <div class="bgm-result-meta">
            ${item.date ? `<span>📅 ${escapeHtml(item.date)}</span>` : ''}
            ${item.score > 0 ? `<span>⭐ ${item.score.toFixed(1)}</span>` : ''}
            ${item.rank > 0 ? `<span>🏆 #${item.rank}</span>` : ''}
          </div>
        </div>
        <button class="btn-primary btn-sm bgm-select-btn" data-id="${item.id}">选择</button>
      </div>
    `).join('')}`;

    // 绑定选择按钮
    resultsEl.querySelectorAll('.bgm-select-btn').forEach(btn => {
      btn.addEventListener('click', () => selectBangumiResult(parseInt(btn.dataset.id)));
    });
  } catch (e) {
    if (bgmSearchAborted) return;
    resultsEl.innerHTML = '<div class="bgm-error">搜索失败：' + escapeHtml(formatError('Bangumi搜索', e)) + '</div>';
    directIdEl.style.display = '';
  } finally {
    cancelBtn.style.display = 'none';
  }
}

async function selectBangumiResult(subjectId) {
  const resultsEl = document.getElementById('bgm-results');

  bgmSearchAborted = false;
  try {
    resultsEl.style.display = 'block';
    resultsEl.innerHTML = '<div class="bgm-loading">📥 正在获取游戏详情（最多等待 15 秒）...</div>';

    const FETCH_TIMEOUT_MS = 15_000;
    const keyword = bgmLastKeyword || null;
    const data = await Promise.race([
      invoke('fetch_bangumi_game', { subjectId, keyword }),
      new Promise((_, reject) =>
        setTimeout(() => reject(new Error('获取超时(15s)：请检查网络后重试')), FETCH_TIMEOUT_MS)
      ),
    ]);

    if (bgmSearchAborted) return;

    // 检查返回数据有效性
    if (!data || (!data.name && !data.name_cn && !data.summary)) {
      resultsEl.innerHTML = '<div class="bgm-error">获取的游戏数据为空，请尝试其他结果</div>';
      return;
    }

    // 填充表单
    applyBangumiFill(data, resultsEl);

  } catch (e) {
    if (bgmSearchAborted) return;

    // === 回退方案：使用搜索结果缓存数据 ===
    const cached = bgmResultsCache.find(item => item.id === subjectId);
    if (cached && (cached.summary || cached.image_large)) {
      resultsEl.innerHTML = '<div class="bgm-loading">📥 详情API不可用，使用搜索结果+下载封面...</div>';

      // 下载封面
      let coverBase64 = '';
      if (cached.image_large) {
        try {
          coverBase64 = await Promise.race([
            invoke('download_bangumi_cover', { imageUrl: cached.image_large }),
            new Promise((_, reject) => setTimeout(() => reject(new Error('封面下载超时')), 10_000)),
          ]);
        } catch (coverErr) {
          console.warn('封面下载失败:', coverErr);
        }
      }

      if (bgmSearchAborted) return;

      const fallbackData = {
        name: cached.name || '',
        name_cn: cached.name_cn || '',
        summary: cached.summary || '',
        cover: coverBase64,
        tags: [],
        date: cached.date || '',
      };
      applyBangumiFill(fallbackData, resultsEl);

    } else {
      resultsEl.innerHTML = '<div class="bgm-error">获取详情失败：' + escapeHtml(formatError('Bangumi', e)) +
        '<br><br>提示：可在 Bangumi 网站找到目标游戏后，直接输入条目 ID 获取数据</div>';
    }
  } finally {
    setLoading(null);
  }
}

/** 将 Bangumi 数据填充到添加表单 */
function applyBangumiFill(data, resultsEl) {
  const fillName = data.name_cn || data.name || '';
  const fillAlias = (data.name_cn && data.name && data.name_cn !== data.name)
    ? data.name : '';
  const summary = data.summary || '';
  const tagsStr = (data.tags || []).join(', ');
  const hasCover = !!data.cover;

  setField('add-name', fillName);
  setField('add-alias', fillAlias);
  setField('add-desc', summary);
  setField('add-tags', tagsStr);
  setField('add-cover', hasCover ? '[Bangumi]' : '');

  if (hasCover) {
    let hidden = document.getElementById('bgm-cover-data');
    if (!hidden) {
      hidden = document.createElement('input');
      hidden.type = 'hidden';
      hidden.id = 'bgm-cover-data';
      document.getElementById('add-form').appendChild(hidden);
    }
    hidden.value = data.cover;
  }

  resultsEl.style.display = 'none';

  const filled = [];
  if (fillName) filled.push('名称');
  if (summary) filled.push('简介');
  if (tagsStr) filled.push('标签(' + data.tags.length + '个)');
  if (hasCover) filled.push('封面');

  showToast('success', filled.length > 0
    ? '已填充: ' + filled.join('、')
    : '未获取到有效数据');
}

/** 安全设置表单字段值，字段不存在时仅警告不崩溃 */
function setField(id, value) {
  const el = document.getElementById(id);
  if (el) {
    el.value = value;
  } else {
    console.warn('Bangumi 填充：找不到字段 #' + id);
  }
}

// ======================== 添加 / 编辑游戏 ========================

function openAddGame() {
  const overlay = document.getElementById('add-overlay');
  const form = document.getElementById('add-form');
  form.reset();
  form.dataset.mode = 'add';
  delete form.dataset.editId;
  populateCategoryDatalist();
  overlay.style.display = 'flex';
  form.onsubmit = createAddSubmitHandler();

  document.getElementById('add-cancel').onclick = closeAdd;
  document.getElementById('add-close').onclick = closeAdd;

  document.getElementById('add-browse').onclick = async () => {
    try {
      const selected = await invoke('open_file_dialog', { title: '选择游戏启动程序', extensions: ['exe'] });
      if (selected) document.getElementById('add-path').value = selected;
    } catch (e) { showToast('error', '选择文件失败：' + e); }
  };
  document.getElementById('add-cover-browse').onclick = async () => {
    try {
      const selected = await invoke('open_file_dialog', { title: '选择封面图片', extensions: ['png', 'jpg', 'jpeg', 'gif', 'bmp'] });
      if (selected) document.getElementById('add-cover').value = selected;
    } catch (e) { showToast('error', '选择图片失败：' + e); }
  };

  // 「新建分类」快捷按钮：打开分类管理弹窗
  document.getElementById('add-cat-new').onclick = () => {
    openCategoryManager();
  };
}

function closeAdd() {
  document.getElementById('add-overlay').style.display = 'none';
  // 重置 Bangumi 查询状态
  document.getElementById('bgm-search-input').value = '';
  document.getElementById('bgm-results').style.display = 'none';
  document.getElementById('bgm-results').innerHTML = '';
  document.getElementById('bgm-search-cancel').style.display = 'none';
  document.getElementById('bgm-direct-id').style.display = 'none';
  document.getElementById('bgm-direct-id-input').value = '';
  bgmResultsCache = [];
  bgmSearchAborted = false;
  bgmLastKeyword = '';
  const hiddenCover = document.getElementById('bgm-cover-data');
  if (hiddenCover) hiddenCover.value = '';
}

function populateCategoryDatalist() {
  const datalist = document.getElementById('category-datalist');
  const cats = getCategories();
  datalist.innerHTML = '';
  if (!Object.keys(cats).includes('未分类')) {
    datalist.innerHTML += '<option value="未分类">';
  }
  Object.keys(cats).sort().forEach(c => {
    datalist.innerHTML += `<option value="${escapeHtml(c)}">`;
  });
}

function createAddSubmitHandler() {
  return async (e) => {
    e.preventDefault();
    const name = document.getElementById('add-name').value.trim();
    if (!name) { showToast('warning', '请输入游戏名称'); return; }
    const path = document.getElementById('add-path').value.trim();
    if (!path) { showToast('warning', '请输入启动路径'); return; }

    const category = document.getElementById('add-category').value || '未分类';
    const tags = document.getElementById('add-tags').value.split(',').map(s => s.trim()).filter(Boolean);
    const status = document.getElementById('add-status').value;
    const coverInput = document.getElementById('add-cover').value.trim();
    const desc = document.getElementById('add-desc').value.trim();
    const gameId = Date.now().toString();

    let coverData = '';
    if (coverInput === '[Bangumi]') {
      // 使用 Bangumi 下载的封面 Base64
      const bgmCover = document.getElementById('bgm-cover-data');
      coverData = bgmCover ? bgmCover.value : '';
    } else if (coverInput) {
      try {
        setLoading('处理封面图片...');
        coverData = await invoke('copy_cover', { sourcePath: coverInput, gameId });
      } catch (e) {
        showToast('error', '复制封面失败：' + e); setLoading(null); return;
      } finally { setLoading(null); }
    }

    const newGame = {
      id: gameId, name,
      alias: document.getElementById('add-alias').value.trim(),
      path, cover: coverData, category, tags, status,
      description: desc, play_time: 0, last_play: null, favorite: false,
    };
    try { await addGameToBackend(newGame); closeAdd(); } catch (e) { }
  };
}

function openEditGame(id) {
  const game = gameData.games.find(g => g.id === id);
  if (!game) return;
  openAddGame();
  const form = document.getElementById('add-form');
  form.dataset.mode = 'edit';
  form.dataset.editId = id;
  document.getElementById('add-name').value = game.name;
  document.getElementById('add-alias').value = game.alias || '';
  document.getElementById('add-path').value = game.path;
  document.getElementById('add-category').value = game.category;
  document.getElementById('add-tags').value = game.tags.join(', ');
  document.getElementById('add-status').value = game.status;
  document.getElementById('add-cover').value = game.cover || '';
  document.getElementById('add-desc').value = game.description || '';

  form.onsubmit = async (ev) => {
    ev.preventDefault();
    const name = document.getElementById('add-name').value.trim();
    if (!name) { showToast('warning', '请输入游戏名称'); return; }
    const path = document.getElementById('add-path').value.trim();
    if (!path) { showToast('warning', '请输入启动路径'); return; }
    const category = document.getElementById('add-category').value || '未分类';
    const tags = document.getElementById('add-tags').value.split(',').map(s => s.trim()).filter(Boolean);
    const status = document.getElementById('add-status').value;
    const coverInput = document.getElementById('add-cover').value.trim();
    const desc = document.getElementById('add-desc').value.trim();

    let coverData = game.cover || '';
    if (coverInput === '[Bangumi]') {
      const bgmCover = document.getElementById('bgm-cover-data');
      coverData = bgmCover ? bgmCover.value : game.cover || '';
    } else if (coverInput && coverInput !== game.cover) {
      try {
        setLoading('更新封面图片...');
        coverData = await invoke('copy_cover', { sourcePath: coverInput, gameId: game.id });
      } catch (e) {
        showToast('error', '更新封面失败：' + e); setLoading(null); return;
      } finally { setLoading(null); }
    } else if (!coverInput) { coverData = ''; }

    const updatedGame = {
      id: game.id, name,
      alias: document.getElementById('add-alias').value.trim(),
      path, cover: coverData, category, tags, status,
      description: desc, play_time: game.play_time, last_play: game.last_play, favorite: game.favorite,
    };
    try { await updateGameToBackend(updatedGame); closeAdd(); } catch (e) { }
  };

  document.getElementById('add-cancel').onclick = () => {
    closeAdd();
    const f = document.getElementById('add-form');
    f.dataset.mode = 'add'; delete f.dataset.editId;
    f.onsubmit = createAddSubmitHandler();
  };
  document.getElementById('add-close').onclick = document.getElementById('add-cancel').onclick;
}

// ======================== 分类管理 ========================

function openCategoryManager() {
  document.getElementById('cats-overlay').style.display = 'flex';
  renderCategoryList();
}

function closeCategoryManager() {
  document.getElementById('cats-overlay').style.display = 'none';
}

function renderCategoryList() {
  const listEl = document.getElementById('cat-manage-list');
  const cats = getCategories();
  const keys = Object.keys(cats).sort();
  listEl.innerHTML = keys.map(cat => `
    <div class="cat-manage-item" data-cat="${escapeHtml(cat)}">
      <span class="cat-name">${escapeHtml(cat)}</span>
      <span class="cat-count">${cats[cat]} 个游戏</span>
      <button class="btn-rename" data-action="rename">重命名</button>
      <button class="btn-del" data-action="delete">删除</button>
    </div>
  `).join('') || '<div style="color:var(--text-muted);padding:16px;text-align:center;">暂无分类</div>';

  listEl.querySelectorAll('.btn-rename').forEach(btn => {
    btn.addEventListener('click', () => {
      const item = btn.closest('.cat-manage-item');
      startRenameCategory(item);
    });
  });
  listEl.querySelectorAll('.btn-del').forEach(btn => {
    btn.addEventListener('click', () => {
      const item = btn.closest('.cat-manage-item');
      deleteCategory(item.dataset.cat);
    });
  });
}

function startRenameCategory(item) {
  const oldName = item.dataset.cat;
  const nameSpan = item.querySelector('.cat-name');
  const renameBtn = item.querySelector('.btn-rename');
  const delBtn = item.querySelector('.btn-del');

  const input = document.createElement('input');
  input.type = 'text';
  input.className = 'cat-rename-input';
  input.value = oldName;
  nameSpan.replaceWith(input);
  renameBtn.textContent = '确认';
  renameBtn.dataset.action = 'confirm-rename';
  delBtn.textContent = '取消';
  delBtn.dataset.action = 'cancel-rename';
  input.focus();
  input.select();

  renameBtn.onclick = () => confirmRename(oldName, input.value.trim());
  delBtn.onclick = () => renderCategoryList(); // 取消：重新渲染

  input.addEventListener('keydown', (e) => {
    if (e.key === 'Enter') confirmRename(oldName, input.value.trim());
    if (e.key === 'Escape') renderCategoryList();
  });
}

async function confirmRename(oldName, newName) {
  if (!newName || newName === oldName) { renderCategoryList(); return; }
  if (getAllCategories()[newName] !== undefined && newName !== oldName) {
    showToast('warning', '目标分类名已存在');
    renderCategoryList();
    return;
  }
  try {
    setLoading('重命名分类...');
    // 更新所有属于该分类的游戏
    const ids = gameData.games.filter(g => g.category === oldName).map(g => g.id);
    if (ids.length > 0) {
      const data = await invoke('batch_update_category', { gameIds: ids, category: newName });
      gameData.games = data.games || [];
    }
    // 更新自定义分类列表
    if (customCategories.has(oldName)) {
      customCategories.delete(oldName);
      customCategories.add(newName);
      saveCustomCategories();
    }
    applyFilter();
    renderCategoryList();
    showToast('success', `分类「${oldName}」已重命名为「${newName}」`);
  } catch (e) {
    showToast('error', formatError('重命名分类', e));
  } finally {
    setLoading(null);
  }
}

async function deleteCategory(cat) {
  if (cat === '未分类') {
    showToast('warning', '「未分类」是默认分类，不能删除');
    return;
  }
  const count = gameData.games.filter(g => g.category === cat).length;
  if (!confirm(`确定删除分类「${cat}」？\n\n该分类下有 ${count} 个游戏将被移到「未分类」。`)) return;
  try {
    setLoading('删除分类...');
    const ids = gameData.games.filter(g => g.category === cat).map(g => g.id);
    if (ids.length > 0) {
      const data = await invoke('batch_update_category', { gameIds: ids, category: '未分类' });
      gameData.games = data.games || [];
    }
    customCategories.delete(cat);
    saveCustomCategories();
    applyFilter();
    renderCategoryList();
    showToast('success', `分类「${cat}」已删除`);
  } catch (e) {
    showToast('error', formatError('删除分类', e));
  } finally {
    setLoading(null);
  }
}

async function createCategory() {
  const input = document.getElementById('new-cat-input');
  const name = input.value.trim();
  if (!name) { showToast('warning', '请输入分类名称'); return; }
  if (getAllCategories()[name] !== undefined) {
    showToast('warning', '该分类已存在');
    return;
  }
  customCategories.add(name);
  saveCustomCategories();
  input.value = '';
  renderCategoryList();
  populateCategoryDatalist();
  refreshBatchCategorySelect();
  // 也刷新侧边栏
  renderSidebar();
  showToast('success', `分类「${name}」已创建`);
}

function initCategoryManager() {
  document.getElementById('btn-manage-cats').addEventListener('click', openCategoryManager);
  document.getElementById('cats-close').addEventListener('click', closeCategoryManager);
  document.getElementById('cats-cancel').addEventListener('click', closeCategoryManager);
  initModalOverlayClose('cats-overlay', closeCategoryManager);
  document.getElementById('btn-create-cat').addEventListener('click', createCategory);
  document.getElementById('new-cat-input').addEventListener('keydown', (e) => {
    if (e.key === 'Enter') createCategory();
  });
}

// ======================== 主题管理 ========================

function getSystemTheme() {
  return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
}

function applyTheme(theme) {
  // theme: 'light' | 'dark' | 'system'
  const effective = theme === 'system' ? getSystemTheme() : theme;
  const isDark = effective === 'dark';
  document.documentElement.classList.toggle('dark', isDark);
  const icon = document.querySelector('.theme-icon');
  if (icon) {
    icon.textContent = isDark ? '☀️' : '🌙';
  }
}

function toggleTheme() {
  const isDark = document.documentElement.classList.contains('dark');
  const newTheme = isDark ? 'light' : 'dark';
  localStorage.setItem('theme', newTheme);
  applyTheme(newTheme);
}

function loadTheme() {
  const stored = localStorage.getItem('theme'); // 'light' | 'dark' | 'system' | null
  const theme = stored || 'system';
  applyTheme(theme);

  // 监听系统主题变化（仅在 "跟随系统" 时响应）
  window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', () => {
    const current = localStorage.getItem('theme');
    if (!current || current === 'system') {
      applyTheme('system');
    }
  });
}

// ======================== 设置界面 ========================

function openSettings() { document.getElementById('settings-overlay').style.display = 'flex'; loadSettingsUI(); }
function closeSettings() { document.getElementById('settings-overlay').style.display = 'none'; }

function loadSettingsUI() {
  const theme = localStorage.getItem('theme') || 'system';
  document.querySelectorAll('.opt-btn[data-theme]').forEach(btn => {
    btn.classList.toggle('active', btn.dataset.theme === theme);
  });
  document.getElementById('radius-slider').value = localStorage.getItem('window-radius') || '14';
  document.getElementById('radius-value').textContent = (localStorage.getItem('window-radius') || '14') + 'px';
  document.getElementById('zoom-select').value = localStorage.getItem('zoom') || '1.0';
  document.getElementById('startup-toggle').checked = localStorage.getItem('startup') === 'true';
  document.getElementById('close-action').value = localStorage.getItem('close-action') || 'tray';
  document.getElementById('default-view').value = localStorage.getItem('default-view') || 'grid';
  document.getElementById('page-size-setting').value = localStorage.getItem('pageSize') || '24';

  // 加载数据路径
  loadDataRootInfo();
}

async function loadDataRootInfo() {
  try {
    const info = await invoke('get_data_size_info');
    document.getElementById('data-path-text').textContent = info.data_root;
    document.getElementById('data-dir-info').innerHTML =
      `文件数: <strong>${info.file_count}</strong> | 封面: <strong>${info.cover_count}</strong> | 大小: <strong>${parseFloat(info.total_mb).toFixed(1)} MB</strong><br>配置文件: ${info.config_dir}`;
  } catch (e) {
    document.getElementById('data-path-text').textContent = '加载失败';
    document.getElementById('data-dir-info').textContent = '无法获取数据目录信息';
  }
}

async function saveSettings() {
  const theme = document.querySelector('.opt-btn[data-theme].active')?.dataset.theme || 'system';
  if (theme === 'system') {
    localStorage.removeItem('theme');
  } else {
    localStorage.setItem('theme', theme);
  }
  applyTheme(theme);

  const radius = document.getElementById('radius-slider').value;
  localStorage.setItem('window-radius', radius);
  document.documentElement.style.setProperty('--radius', radius + 'px');

  const zoom = document.getElementById('zoom-select').value;
  localStorage.setItem('zoom', zoom);
  document.querySelector('.app').style.transform = `scale(${zoom})`;
  document.querySelector('.app').style.transformOrigin = 'top left';

  const startup = document.getElementById('startup-toggle').checked;
  localStorage.setItem('startup', startup);
  try { await invoke('set_startup', { enabled: startup }); } catch (e) { showToast('warning', '开机自启设置失败'); }

  const closeAction = document.getElementById('close-action').value;
  localStorage.setItem('close-action', closeAction);

  const defaultView = document.getElementById('default-view').value;
  localStorage.setItem('default-view', defaultView);
  if (currentFilter.view !== defaultView) {
    currentFilter.view = defaultView;
    document.querySelectorAll('.view-btn').forEach(b => b.classList.toggle('active', b.dataset.view === defaultView));
    renderGames();
  }

  const newPageSize = parseInt(document.getElementById('page-size-setting').value, 10);
  if (newPageSize !== pageSize) {
    pageSize = newPageSize;
    localStorage.setItem('pageSize', pageSize);
    currentPage = 1;
    renderGames();
  }

  closeSettings();
  showToast('success', '设置已保存');
}

async function backupData() {
  try {
    setLoading('正在备份数据...');
    const path = await invoke('backup_data');
    showToast('success', `备份成功：${path}`);
  } catch (e) { showToast('error', formatError('备份', e)); } finally { setLoading(null); }
}

function restoreData() {
  invoke('open_file_dialog', { title: '选择备份文件', extensions: ['json'] }).then(async (filePath) => {
    if (!filePath) return;
    try { setLoading('正在恢复数据...'); await invoke('restore_data', { filePath }); await loadGames(); showToast('success', '数据恢复成功'); }
    catch (e) { showToast('error', formatError('恢复', e)); } finally { setLoading(null); }
  }).catch(e => showToast('error', '选择文件失败：' + e));
}

async function cleanUp() {
  if (!confirm('此操作将删除所有路径无效的游戏条目，确定继续？')) return;
  try { setLoading('正在清理无效数据...'); const data = await invoke('cleanup_invalid'); gameData.games = data.games || []; applyFilter(); showToast('success', `清理完成，当前共有 ${gameData.games.length} 款游戏`); }
  catch (e) { showToast('error', formatError('清理', e)); } finally { setLoading(null); }
}

// ======================== 数据路径迁移 ========================

let pendingDataPath = '';

async function openBrowsePath() {
  try {
    const folder = await invoke('pick_folder', { title: '选择新的数据存储目录' });
    if (!folder) return;

    // 获取当前信息
    const oldInfo = await invoke('get_data_size_info');
    pendingDataPath = folder;

    // 填充确认对话框
    document.getElementById('confirm-old-path').textContent = oldInfo.data_root;
    document.getElementById('confirm-new-path').textContent = folder;
    document.getElementById('confirm-stat-files').textContent = `文件: ${oldInfo.file_count}`;
    document.getElementById('confirm-stat-covers').textContent = `封面: ${oldInfo.cover_count}`;
    document.getElementById('confirm-stat-size').textContent = `大小: ${parseFloat(oldInfo.total_mb).toFixed(1)} MB`;
    document.getElementById('confirm-error-section').style.display = 'none';
    document.getElementById('confirm-error-text').textContent = '';
    document.getElementById('migrate-confirm-ok').disabled = false;

    // 显示对话框
    document.getElementById('migrate-confirm-overlay').style.display = 'flex';
  } catch (e) {
    showToast('error', '获取路径信息失败: ' + formatError('', e));
  }
}

function closeMigrateConfirm() {
  document.getElementById('migrate-confirm-overlay').style.display = 'none';
  pendingDataPath = '';
}

async function executeMigrate() {
  if (!pendingDataPath) return;

  const okBtn = document.getElementById('migrate-confirm-ok');
  okBtn.disabled = true;
  okBtn.textContent = '迁移中...';
  document.getElementById('confirm-error-section').style.display = 'none';

  try {
    setLoading('正在迁移数据...');
    await invoke('set_data_root', { path: pendingDataPath, migrate: true });
    // 刷新数据
    await loadGames();
    showToast('success', '数据迁移完成，路径已更新');
    closeMigrateConfirm();
    // 刷新设置页显示
    loadDataRootInfo();
  } catch (e) {
    const errMsg = formatError('迁移', e);
    document.getElementById('confirm-error-text').textContent = '迁移失败: ' + errMsg;
    document.getElementById('confirm-error-section').style.display = 'block';
    okBtn.disabled = false;
    okBtn.textContent = '重试迁移';
    showToast('error', errMsg);
  } finally {
    setLoading(null);
    if (document.getElementById('migrate-confirm-overlay').style.display !== 'none') {
      okBtn.textContent = '确认迁移';
    }
  }
}

function initSettings() {
  document.getElementById('btn-settings').addEventListener('click', openSettings);
  document.getElementById('settings-close').addEventListener('click', closeSettings);
  document.getElementById('settings-cancel').addEventListener('click', closeSettings);
  initModalOverlayClose('settings-overlay', closeSettings);
  document.querySelectorAll('.opt-btn[data-theme]').forEach(btn => {
    btn.addEventListener('click', () => {
      document.querySelectorAll('.opt-btn[data-theme]').forEach(b => b.classList.remove('active'));
      btn.classList.add('active');
    });
  });
  document.getElementById('radius-slider').addEventListener('input', (e) => {
    document.getElementById('radius-value').textContent = e.target.value + 'px';
  });
  document.getElementById('settings-save').addEventListener('click', saveSettings);
  document.getElementById('btn-backup').addEventListener('click', backupData);
  document.getElementById('btn-restore').addEventListener('click', restoreData);
  document.getElementById('btn-cleanup').addEventListener('click', cleanUp);
  document.getElementById('btn-data-root-browse').addEventListener('click', openBrowsePath);
  // 迁移确认对话框
  document.getElementById('migrate-confirm-cancel').addEventListener('click', closeMigrateConfirm);
  document.getElementById('migrate-confirm-close').addEventListener('click', closeMigrateConfirm);
  document.getElementById('migrate-confirm-ok').addEventListener('click', executeMigrate);
  initModalOverlayClose('migrate-confirm-overlay', closeMigrateConfirm);
}

// ======================== Modal Overlay 关闭辅助 ========================

/**
 * 为 modal overlay 绑定安全的关闭逻辑
 * 仅在以下情况关闭：
 *   1. mousedown 和 click 都发生在 overlay 上（非子窗口内容区）
 * 这样可防止用户在子窗口内按下鼠标并拖到窗口外松开时误关。
 */
function initModalOverlayClose(overlayId, closeFn) {
  const overlay = document.getElementById(overlayId);
  if (!overlay) return;
  let mouseDownTarget = null;
  overlay.addEventListener('mousedown', (e) => {
    mouseDownTarget = e.target;
  });
  overlay.addEventListener('click', (e) => {
    // 只有当 mousedown 和 click 都在 overlay 本身上时才关闭
    if (mouseDownTarget === e.currentTarget && e.target === e.currentTarget) {
      closeFn();
    }
    mouseDownTarget = null;
  });
}

// ======================== 初始化 ========================

function init() {
  loadTheme();
  const zoom = localStorage.getItem('zoom') || '1.0';
  document.querySelector('.app').style.transform = `scale(${zoom})`;
  document.querySelector('.app').style.transformOrigin = 'top left';
  document.documentElement.style.setProperty('--radius', (localStorage.getItem('window-radius') || '14') + 'px');

  initSettings();
  initCategoryManager();
  initBatchSystem();
  initBangumiSearch();
  loadGames();

  const debouncedSearch = debounce((value) => { currentFilter.search = value; applyFilter(); }, 300);
  document.getElementById('search-input').addEventListener('input', (e) => {
    e.target.classList.toggle('has-value', e.target.value.trim().length > 0);
    debouncedSearch(e.target.value);
  });

  document.querySelectorAll('.view-btn').forEach(btn => {
    btn.addEventListener('click', () => {
      document.querySelectorAll('.view-btn').forEach(b => b.classList.remove('active'));
      btn.classList.add('active');
      currentFilter.view = btn.dataset.view;
      renderGames();
    });
  });

  document.getElementById('btn-add-game').addEventListener('click', openAddGame);
  document.getElementById('btn-empty-add').addEventListener('click', openAddGame);
  document.getElementById('theme-toggle').addEventListener('click', toggleTheme);

  // 弹窗 overlay：通过 mousedown 追踪防止拖拽误关
  initModalOverlayClose('detail-overlay', closeDetail);
  document.getElementById('detail-close').addEventListener('click', (e) => { e.stopPropagation(); closeDetail(); });

  initModalOverlayClose('add-overlay', closeAdd);
  document.querySelector('#add-overlay .add-modal').addEventListener('click', (e) => { e.stopPropagation(); });

  document.addEventListener('keydown', (e) => {
    if (e.key === 'Escape') {
      if (batchMode) { exitBatchMode(); return; }
      closeDetail(); closeAdd(); closeSettings(); closeCategoryManager();
    }
    if ((e.ctrlKey || e.metaKey) && e.key === 'k') {
      e.preventDefault(); document.getElementById('search-input').focus();
    }
  });

  // 游戏退出事件：移除运行时标记 + 刷新数据
  listen('game-exited', (event) => {
    const { game_id, play_time_added, updated } = event.payload;
    runningGameIds.delete(game_id);
    if (updated) {
      const game = gameData.games.find(g => g.id === game_id);
      const name = game ? game.name : game_id;
      showToast('info', `「${escapeHtml(name)}」已退出，本次游玩 +${play_time_added} 分钟`);
      invoke('get_games').then(data => {
        gameData.games = data.games || [];
        applyFilter();
      }).catch(() => { });
    }
  });

}

document.addEventListener('DOMContentLoaded', init);
window.__app = { gameData, currentFilter, applyFilter, renderGames, showToast, runningGameIds };
