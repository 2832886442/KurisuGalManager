import Icon from './Icon';
import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import useAppVersion from '../hooks/useAppVersion';

/**
 * AboutPage - 关于我们页面
 * 全面介绍项目:技术栈、框架、插件、联系方式
 */
export default function AboutPage() {
  const [appIcon, setAppIcon] = useState('');
  const appVersion = useAppVersion();

  useEffect(() => {
    invoke('get_app_icon').then(setAppIcon).catch(() => { });
  }, []);
  const techStack = [
    {
      group: '前端框架',
      icon: 'palette',
      items: [
        { name: 'React 18', desc: '主 UI 框架,负责应用主体界面与状态管理' },
        { name: 'Vue 3', desc: '混合架构,用于渲染排名管理界面(RankingApp.vue)' },
        { name: 'Vite 5', desc: '前端构建工具,提供快速 HMR 与打包能力' },
      ],
    },
    {
      group: '桌面运行时',
      icon: 'rocket',
      items: [
        { name: 'Tauri 2', desc: '跨平台桌面应用框架,基于 Rust + WebView' },
        { name: 'Rust', desc: '后端核心语言,负责数据持久化、命令调用与系统交互' },
        { name: 'Tokio', desc: '异步运行时,处理 Bangumi API 请求与文件 I/O' },
      ],
    },
    {
      group: '插件与依赖',
      icon: 'database',
      items: [
        { name: '@tauri-apps/plugin-opener', desc: 'Tauri 官方插件,用于打开外部链接与文件' },
        { name: 'Serde / Serde JSON', desc: 'Rust 序列化库,处理游戏数据与配置读写' },
        { name: 'Reqwest', desc: 'HTTP 客户端,用于 Bangumi API 数据获取' },
        { name: 'Lucide Icons', desc: '线性 SVG 图标集,统一应用视觉风格' },
      ],
    },
    {
      group: '数据来源',
      icon: 'image',
      items: [
        { name: 'Bangumi API', desc: '开放的番剧/视觉小说数据库,提供封面、简介与标签' },
        { name: '本地文件系统', desc: '游戏封面、截图与数据均存储于本地数据目录' },
      ],
    },
  ];

  const features = [
    '游戏库管理:网格/列表视图、分类、标签、收藏',
    'Bangumi 元数据自动填充:封面、简介、标签',
    '分级排名:拖拽式 H2L 风格榜单,自定义等级与颜色',
    '数据统计:游玩时长 Top5、分类分布、最近游玩',
    '多主题支持:暗色、亮色、Fusion、Glass、跟随系统',
    '数据迁移:可自定义数据存储路径,支持备份与恢复',
    '批量操作:多选移动分类、清理无效条目',
    '快速启动:一键启动游戏并记录游玩时长',
  ];

  return (
    <div className="about-page">
      <div className="about-page-header">
        <div className="about-page-hero">
          <div className="about-page-logo">
            {appIcon ? (
              <img src={appIcon} alt="KurisuGal" width="32" height="32" />
            ) : (
              <Icon name="logo-gamepad" size={32} />
            )}
          </div>
          <div className="about-page-hero-text">
            <h1 className="about-page-title">关于 KurisuGal</h1>
            <p className="about-page-tagline">
              一款基于 Tauri + React + Vue3 + Rust 构建的现代化 Galgame 管理器
            </p>
          </div>
        </div>
      </div>

      <div className="about-page-content">
        {/* 项目简介 */}
        <section className="about-page-section">
          <h2 className="about-page-section-title">
            <Icon name="info" size={16} /> 项目简介
          </h2>
          <div className="about-page-card">
            <p className="about-page-intro">
              KurisuGal 是一款专为视觉小说(Galgame)玩家设计的桌面端游戏库管理器。
              项目采用 Tauri 2 + Rust 后端,结合 React 18 与 Vue 3 混合前端架构,
              在保持轻量体积的同时提供流畅的本地管理体验。支持从 Bangumi 自动抓取
              游戏元数据,并通过拖拽式排名系统帮助玩家整理自己的游戏榜单。
            </p>
            <div className="about-page-meta-grid">
              <div className="about-page-meta-item">
                <span className="about-page-meta-label">软件名称</span>
                <span className="about-page-meta-value">KurisuGal Galgame 管理器</span>
              </div>
              <div className="about-page-meta-item">
                <span className="about-page-meta-label">当前版本</span>
                <span className="about-page-meta-value">v{appVersion || '...'}</span>
              </div>
              <div className="about-page-meta-item">
                <span className="about-page-meta-label">作者</span>
                <span className="about-page-meta-value">CoolSomeBody</span>
              </div>
              <div className="about-page-meta-item">
                <span className="about-page-meta-label">开源协议</span>
                <span className="about-page-meta-value">MIT License</span>
              </div>
              <div className="about-page-meta-item">
                <span className="about-page-meta-label">项目仓库</span>
                <a
                  className="about-page-meta-link"
                  href="https://github.com/2832886442/KurisuGalManager"
                  target="_blank"
                  rel="noopener noreferrer"
                >
                  <Icon name="arrow-right" size={12} /> github.com/2832886442/KurisuGalManager
                </a>
              </div>
              <div className="about-page-meta-item">
                <span className="about-page-meta-label">联系邮箱</span>
                <a className="about-page-meta-link" href="mailto:1904483299@qq.com">
                  <Icon name="arrow-right" size={12} /> 1904483299@qq.com
                </a>
              </div>
            </div>
          </div>
        </section>

        {/* 技术栈 */}
        <section className="about-page-section">
          <h2 className="about-page-section-title">
            <Icon name="palette" size={16} /> 技术栈
          </h2>
          <div className="about-page-tech-grid">
            {techStack.map((g, i) => (
              <div key={i} className="about-page-tech-card">
                <div className="about-page-tech-header">
                  <Icon name={g.icon} size={14} />
                  <span>{g.group}</span>
                </div>
                <ul className="about-page-tech-list">
                  {g.items.map((it, j) => (
                    <li key={j} className="about-page-tech-item">
                      <span className="about-page-tech-name">{it.name}</span>
                      <span className="about-page-tech-desc">{it.desc}</span>
                    </li>
                  ))}
                </ul>
              </div>
            ))}
          </div>
        </section>

        {/* 功能列表 */}
        <section className="about-page-section">
          <h2 className="about-page-section-title">
            <Icon name="sparkles" size={16} /> 功能特性
          </h2>
          <div className="about-page-card">
            <ul className="about-page-feature-list">
              {features.map((f, i) => (
                <li key={i} className="about-page-feature-item">
                  <span className="about-page-feature-bullet">
                    <Icon name="check" size={10} />
                  </span>
                  <span>{f}</span>
                </li>
              ))}
            </ul>
          </div>
        </section>

        {/* 致谢 */}
        <section className="about-page-section">
          <h2 className="about-page-section-title">
            <Icon name="heart" size={16} /> 致谢
          </h2>
          <div className="about-page-card">
            <p className="about-page-intro">
              感谢以下开源项目与服务,没有它们就没有 KurisuGal:
            </p>
            <div className="about-page-thanks">
              <span className="about-page-thanks-chip">Tauri</span>
              <span className="about-page-thanks-chip">React</span>
              <span className="about-page-thanks-chip">Vue</span>
              <span className="about-page-thanks-chip">Rust</span>
              <span className="about-page-thanks-chip">Vite</span>
              <span className="about-page-thanks-chip">Bangumi</span>
              <span className="about-page-thanks-chip">Lucide</span>
            </div>
          </div>
        </section>

        <footer className="about-page-footer">
          <span>© 2026 KurisuGal · CoolSomeBody</span>
          <span className="about-page-footer-sep">·</span>
          <span>反馈邮箱: 1904483299@qq.com</span>
        </footer>
      </div>
    </div>
  );
}
