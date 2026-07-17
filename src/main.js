// main.js - Galgame 管理器 (纯全局对象，无 Vite)
const invoke = window.__TAURI__.core.invoke;

let gameData = { games: [] };
let currentFilter = {
  category: 'all',
  tag: null,
  search: '',
  view: 'grid',
};

// ======================== 渲染引擎 ========================

function getCategories() {
  const cats = {};
  gameData.games.forEach(g => {
    cats[g.category] = (cats[g.category] || 0) + 1;
  });
  return cats;
}

function getTags() {
  const tags = {};
  gameData.games.forEach(g => {
    g.tags.forEach(t => {
      tags[t] = (tags[t] || 0) + 1;
    });
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
    const keyword = currentFilter.search.trim().toLowerCase();
    list = list.filter(g =>
      g.name.toLowerCase().includes(keyword) ||
      (g.alias && g.alias.toLowerCase().includes(keyword))
    );
  }
  return list;
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
    currentFilter.category = 'all';
    currentFilter.tag = null;
    applyFilter();
  });
  listEl.appendChild(allItem);

  const sorted = Object.keys(cats).sort();
  sorted.forEach(cat => {
    const li = document.createElement('li');
    li.className = 'nav-item' + (currentFilter.category === cat ? ' active' : '');
    li.dataset.category = cat;
    li.innerHTML = `
      <span class="nav-icon">📁</span>
      <span class="nav-label">${cat}</span>
      <span class="nav-count">${cats[cat]}</span>
    `;
    li.addEventListener('click', () => {
      currentFilter.category = cat;
      currentFilter.tag = null;
      applyFilter();
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
  sorted.forEach(tag => {
    const span = document.createElement('span');
    span.className = 'tag-item' + (currentFilter.tag === tag ? ' active' : '');
    span.textContent = `${tag} (${tags[tag]})`;
    span.addEventListener('click', () => {
      if (currentFilter.tag === tag) {
        currentFilter.tag = null;
      } else {
        currentFilter.tag = tag;
        currentFilter.category = 'all';
      }
      applyFilter();
    });
    cloud.appendChild(span);
  });
}

function renderGames() {
  const grid = document.getElementById('game-grid');
  const empty = document.getElementById('empty-state');
  const filtered = getFilteredGames();

  const title = document.getElementById('page-title');
  let titleText = '全部游戏';
  if (currentFilter.category !== 'all') titleText = currentFilter.category;
  else if (currentFilter.tag) titleText = `#${currentFilter.tag}`;
  title.textContent = titleText;
  document.getElementById('game-count').textContent = `${filtered.length} 款`;

  if (filtered.length === 0) {
    grid.style.display = 'none';
    empty.style.display = 'flex';
    return;
  }
  grid.style.display = 'grid';
  empty.style.display = 'none';

  const isGrid = currentFilter.view === 'grid';
  grid.className = isGrid ? 'game-grid grid-view' : 'game-grid list-view';

  grid.innerHTML = filtered.map(game => {
    const statusMap = {
      '未游玩': 'status-unplayed',
      '游玩中': 'status-playing',
      '已通关': 'status-completed',
      '搁置': 'status-shelved',
    };
    const statusClass = statusMap[game.status] || '';

    // 封面直接使用 Base64 数据 URI
    const coverHtml = game.cover && game.cover.startsWith('data:')
      ? `<img src="${game.cover}" alt="${game.name}" loading="lazy" />`
      : `<div class="no-cover">🎮</div>`;

    return `
      <div class="game-card" data-id="${game.id}">
        <div class="card-cover">
          ${coverHtml}
          ${game.favorite ? '<span class="favorite-badge">⭐</span>' : ''}
          <span class="status-badge ${statusClass}">${game.status}</span>
        </div>
        <div class="card-body">
          <h4 class="card-title">${game.name}</h4>
          ${game.alias ? `<p class="card-alias">${game.alias}</p>` : ''}
          <div class="card-tags">
            ${game.tags.slice(0, 3).map(t => `<span class="card-tag">${t}</span>`).join('')}
            ${game.tags.length > 3 ? `<span class="card-tag">+${game.tags.length - 3}</span>` : ''}
          </div>
          <div class="card-footer">
            <span class="card-category">${game.category}</span>
            <span class="card-playtime">${game.play_time > 0 ? game.play_time + 'min' : ''}</span>
          </div>
        </div>
      </div>
    `;
  }).join('');

  grid.querySelectorAll('.game-card').forEach(card => {
    card.addEventListener('click', () => {
      const id = card.dataset.id;
      showDetail(id);
    });
  });
}

function applyFilter() {
  renderSidebar();
  renderGames();
}

// ======================== 后端操作 ========================

async function loadGames() {
  try {
    const data = await invoke('get_games');
    gameData.games = data.games || [];
    applyFilter();
  } catch (e) {
    console.error('加载游戏失败', e);
    alert('加载游戏数据失败，请检查后端');
  }
}

async function addGameToBackend(game) {
  try {
    const data = await invoke('add_game', { game });
    gameData.games = data.games || [];
    applyFilter();
  } catch (e) {
    alert('添加失败：' + e);
    throw e;
  }
}

async function updateGameToBackend(game) {
  try {
    const data = await invoke('update_game', { game });
    gameData.games = data.games || [];
    applyFilter();
  } catch (e) {
    alert('更新失败：' + e);
    throw e;
  }
}

async function deleteGameFromBackend(id) {
  try {
    const data = await invoke('delete_game', { id });
    gameData.games = data.games || [];
    applyFilter();
  } catch (e) {
    alert('删除失败：' + e);
    throw e;
  }
}

// ======================== 详情弹窗 ========================

function showDetail(id) {
  const game = gameData.games.find(g => g.id === id);
  if (!game) return;
  const overlay = document.getElementById('detail-overlay');

  const coverSrc = game.cover && game.cover.startsWith('data:') ? game.cover : '';

  const container = document.getElementById('detail-container');
  container.innerHTML = `
    <div class="detail-header">
      <div class="detail-cover">
        ${coverSrc ? `<img src="${coverSrc}" alt="${game.name}" />` : '<div class="no-cover">🎮</div>'}
      </div>
      <div class="detail-info">
        <h2>${game.name}</h2>
        ${game.alias ? `<p class="detail-alias">别名：${game.alias}</p>` : ''}
        <p class="detail-category">分类：${game.category}</p>
        <div class="detail-tags">
          ${game.tags.map(t => `<span class="tag-item">${t}</span>`).join('')}
        </div>
        <p class="detail-status">状态：<span class="${game.status}">${game.status}</span></p>
        <p class="detail-playtime">游玩时长：${game.play_time} 分钟</p>
        <p class="detail-lastplay">上次游玩：${game.last_play || '从未'}</p>
        <div class="detail-actions">
          <button class="btn-primary" id="detail-launch">🚀 启动游戏</button>
          <button class="btn-edit" id="detail-edit">✏️ 编辑</button>
          <button class="btn-danger" id="detail-delete">🗑️ 删除</button>
        </div>
      </div>
    </div>
    <div class="detail-body">
      <h4>简介</h4>
      <p>${game.description || '暂无简介'}</p>
      <h4>启动路径</h4>
      <code>${game.path}</code>
    </div>
  `;

  overlay.style.display = 'flex';

  document.getElementById('detail-launch').addEventListener('click', () => {
    invoke('launch_game', { path: game.path }).catch(err => alert('启动失败：' + err));
  });

  document.getElementById('detail-edit').addEventListener('click', () => {
    closeDetail();
    openEditGame(game.id);
  });

  document.getElementById('detail-delete').addEventListener('click', async () => {
    if (confirm(`确定要删除游戏“${game.name}”吗？`)) {
      await deleteGameFromBackend(game.id);
      closeDetail();
    }
  });
}

function closeDetail() {
  document.getElementById('detail-overlay').style.display = 'none';
}

// ======================== 添加 / 编辑游戏 ========================

function openAddGame() {
  const overlay = document.getElementById('add-overlay');
  const form = document.getElementById('add-form');
  form.reset();
  populateCategorySelect();
  overlay.style.display = 'flex';

  form.onsubmit = async (e) => {
    e.preventDefault();
    const name = document.getElementById('add-name').value.trim();
    if (!name) return alert('请输入游戏名称');
    const path = document.getElementById('add-path').value.trim();
    if (!path) return alert('请输入启动路径');
    const category = document.getElementById('add-category').value || '未分类';
    const tags = document.getElementById('add-tags').value.split(',').map(s => s.trim()).filter(Boolean);
    const status = document.getElementById('add-status').value;
    const coverInput = document.getElementById('add-cover').value.trim();
    const desc = document.getElementById('add-desc').value.trim();

    const gameId = Date.now().toString();

    let coverData = '';
    if (coverInput) {
      try {
        coverData = await invoke('copy_cover', { sourcePath: coverInput, gameId: gameId });
      } catch (e) {
        alert('复制封面失败：' + e);
        return;
      }
    }

    const newGame = {
      id: gameId,
      name,
      alias: document.getElementById('add-alias').value.trim(),
      path,
      cover: coverData,
      category,
      tags,
      status,
      description: desc,
      play_time: 0,
      last_play: null,
      favorite: false,
    };

    try {
      await addGameToBackend(newGame);
      closeAdd();
    } catch (e) { }
  };

  document.getElementById('add-cancel').onclick = closeAdd;
  document.getElementById('add-close').onclick = closeAdd;

  document.getElementById('add-browse').onclick = async () => {
    try {
      const selected = await invoke('open_file_dialog', {
        title: '选择游戏启动程序',
        extensions: ['exe']
      });
      if (selected) {
        document.getElementById('add-path').value = selected;
      }
    } catch (e) {
      alert('选择文件失败：' + e);
    }
  };

  document.getElementById('add-cover-browse').onclick = async () => {
    try {
      const selected = await invoke('open_file_dialog', {
        title: '选择封面图片',
        extensions: ['png', 'jpg', 'jpeg', 'gif', 'bmp']
      });
      if (selected) {
        document.getElementById('add-cover').value = selected;
      }
    } catch (e) {
      alert('选择图片失败：' + e);
    }
  };
}

function closeAdd() {
  document.getElementById('add-overlay').style.display = 'none';
}

function populateCategorySelect() {
  const select = document.getElementById('add-category');
  const cats = getCategories();
  select.innerHTML = '<option value="未分类">未分类</option>';
  Object.keys(cats).sort().forEach(c => {
    select.innerHTML += `<option value="${c}">${c}</option>`;
  });
}

function openEditGame(id) {
  const game = gameData.games.find(g => g.id === id);
  if (!game) return;
  openAddGame();
  const form = document.getElementById('add-form');
  document.getElementById('add-name').value = game.name;
  document.getElementById('add-alias').value = game.alias || '';
  document.getElementById('add-path').value = game.path;
  document.getElementById('add-category').value = game.category;
  document.getElementById('add-tags').value = game.tags.join(', ');
  document.getElementById('add-status').value = game.status;
  document.getElementById('add-cover').value = game.cover || '';
  document.getElementById('add-desc').value = game.description || '';

  form.onsubmit = async (e) => {
    e.preventDefault();
    const name = document.getElementById('add-name').value.trim();
    if (!name) return alert('请输入游戏名称');
    const path = document.getElementById('add-path').value.trim();
    if (!path) return alert('请输入启动路径');
    const category = document.getElementById('add-category').value || '未分类';
    const tags = document.getElementById('add-tags').value.split(',').map(s => s.trim()).filter(Boolean);
    const status = document.getElementById('add-status').value;
    const coverInput = document.getElementById('add-cover').value.trim();
    const desc = document.getElementById('add-desc').value.trim();

    let coverData = game.cover || '';
    // 如果用户更换了封面，且输入不为空，复制新封面
    if (coverInput && coverInput !== game.cover) {
      try {
        coverData = await invoke('copy_cover', { sourcePath: coverInput, gameId: game.id });
      } catch (e) {
        alert('更新封面失败：' + e);
        return;
      }
    } else if (!coverInput) {
      // 用户清空了封面
      coverData = '';
    }

    const updatedGame = {
      id: game.id,
      name,
      alias: document.getElementById('add-alias').value.trim(),
      path,
      cover: coverData,
      category,
      tags,
      status,
      description: desc,
      play_time: game.play_time,
      last_play: game.last_play,
      favorite: game.favorite,
    };

    try {
      await updateGameToBackend(updatedGame);
      closeAdd();
    } catch (e) { }
  };

  document.getElementById('add-cancel').onclick = () => {
    closeAdd();
    // 恢复默认提交（重新绑定添加逻辑）
    document.getElementById('add-form').onsubmit = async (e) => {
      e.preventDefault();
      const name = document.getElementById('add-name').value.trim();
      if (!name) return alert('请输入游戏名称');
      const path = document.getElementById('add-path').value.trim();
      if (!path) return alert('请输入启动路径');
      const category = document.getElementById('add-category').value || '未分类';
      const tags = document.getElementById('add-tags').value.split(',').map(s => s.trim()).filter(Boolean);
      const status = document.getElementById('add-status').value;
      const coverInput = document.getElementById('add-cover').value.trim();
      const desc = document.getElementById('add-desc').value.trim();
      const gameId = Date.now().toString();
      let coverData = '';
      if (coverInput) {
        try {
          coverData = await invoke('copy_cover', { sourcePath: coverInput, gameId: gameId });
        } catch (e) {
          alert('复制封面失败：' + e);
          return;
        }
      }
      const newGame = {
        id: gameId,
        name,
        alias: document.getElementById('add-alias').value.trim(),
        path,
        cover: coverData,
        category,
        tags,
        status,
        description: desc,
        play_time: 0,
        last_play: null,
        favorite: false,
      };
      try {
        await addGameToBackend(newGame);
        closeAdd();
      } catch (e) { }
    };
  };
  document.getElementById('add-close').onclick = document.getElementById('add-cancel').onclick;
}

// ======================== 主题切换 ========================

function toggleTheme() {
  document.documentElement.classList.toggle('dark');
  const icon = document.querySelector('.theme-icon');
  if (document.documentElement.classList.contains('dark')) {
    icon.textContent = '☀️';
    localStorage.setItem('theme', 'dark');
  } else {
    icon.textContent = '🌙';
    localStorage.setItem('theme', 'light');
  }
}

function loadTheme() {
  const theme = localStorage.getItem('theme') || 'light';
  if (theme === 'dark') {
    document.documentElement.classList.add('dark');
    document.querySelector('.theme-icon').textContent = '☀️';
  } else {
    document.documentElement.classList.remove('dark');
    document.querySelector('.theme-icon').textContent = '🌙';
  }
}

// ======================== 设置界面 ========================

function openSettings() {
  document.getElementById('settings-overlay').style.display = 'flex';
  loadSettingsUI();
}

function closeSettings() {
  document.getElementById('settings-overlay').style.display = 'none';
}

function loadSettingsUI() {
  const theme = localStorage.getItem('theme') || 'system';
  document.querySelectorAll('.opt-btn[data-theme]').forEach(btn => {
    btn.classList.toggle('active', btn.dataset.theme === theme);
  });
  const radius = localStorage.getItem('window-radius') || '14';
  document.getElementById('radius-slider').value = radius;
  document.getElementById('radius-value').textContent = radius + 'px';
  document.getElementById('zoom-select').value = localStorage.getItem('zoom') || '1.0';
  document.getElementById('startup-toggle').checked = localStorage.getItem('startup') === 'true';
  document.getElementById('close-action').value = localStorage.getItem('close-action') || 'tray';
  document.getElementById('default-view').value = localStorage.getItem('default-view') || 'grid';
}

function saveSettings() {
  const theme = document.querySelector('.opt-btn[data-theme].active')?.dataset.theme || 'system';
  if (theme === 'system') {
    localStorage.removeItem('theme');
    const prefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches;
    document.documentElement.classList.toggle('dark', prefersDark);
  } else {
    localStorage.setItem('theme', theme);
    document.documentElement.classList.toggle('dark', theme === 'dark');
  }
  const icon = document.querySelector('.theme-icon');
  if (theme === 'dark') icon.textContent = '☀️';
  else if (theme === 'light') icon.textContent = '🌙';
  else icon.textContent = '💻';

  const radius = document.getElementById('radius-slider').value;
  localStorage.setItem('window-radius', radius);
  document.documentElement.style.setProperty('--radius', radius + 'px');

  const zoom = document.getElementById('zoom-select').value;
  localStorage.setItem('zoom', zoom);
  document.querySelector('.app').style.transform = `scale(${zoom})`;
  document.querySelector('.app').style.transformOrigin = 'top left';

  const startup = document.getElementById('startup-toggle').checked;
  localStorage.setItem('startup', startup);
  invoke('set_startup', { enabled: startup }).catch(console.error);

  const closeAction = document.getElementById('close-action').value;
  localStorage.setItem('close-action', closeAction);

  const defaultView = document.getElementById('default-view').value;
  localStorage.setItem('default-view', defaultView);
  if (currentFilter.view !== defaultView) {
    currentFilter.view = defaultView;
    document.querySelectorAll('.view-btn').forEach(b => b.classList.toggle('active', b.dataset.view === defaultView));
    renderGames();
  }

  closeSettings();
}

function backupData() {
  alert('📤 备份功能：将导出所有游戏数据到本地文件 (game_backup.json)');
}

function restoreData() {
  alert('📥 恢复功能：请选择备份文件，将覆盖当前数据');
}

function cleanUp() {
  if (confirm('此操作将删除所有路径无效的游戏和空分类，确定继续？')) {
    invoke('cleanup_invalid')
      .then(data => {
        gameData.games = data.games || [];
        applyFilter();
        alert(`清理完成，当前共有 ${gameData.games.length} 款游戏。`);
      })
      .catch(err => alert('清理失败：' + err));
  }
}

function initSettings() {
  document.getElementById('btn-settings').addEventListener('click', openSettings);
  document.getElementById('settings-close').addEventListener('click', closeSettings);
  document.getElementById('settings-cancel').addEventListener('click', closeSettings);
  document.getElementById('settings-overlay').addEventListener('click', (e) => {
    if (e.target === e.currentTarget) closeSettings();
  });

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
}

// ======================== 初始化 ========================

function init() {
  loadTheme();

  const zoom = localStorage.getItem('zoom') || '1.0';
  document.querySelector('.app').style.transform = `scale(${zoom})`;
  document.querySelector('.app').style.transformOrigin = 'top left';
  const radius = localStorage.getItem('window-radius') || '14';
  document.documentElement.style.setProperty('--radius', radius + 'px');

  initSettings();
  loadGames();

  document.getElementById('search-input').addEventListener('input', (e) => {
    currentFilter.search = e.target.value;
    applyFilter();
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

  document.getElementById('detail-overlay').addEventListener('click', (e) => {
    if (e.target === e.currentTarget) closeDetail();
  });
  document.getElementById('detail-close').addEventListener('click', closeDetail);

  document.getElementById('add-overlay').addEventListener('click', (e) => {
    if (e.target === e.currentTarget) closeAdd();
  });
}

document.addEventListener('DOMContentLoaded', init);

window.__app = { gameData, currentFilter, applyFilter, renderGames };