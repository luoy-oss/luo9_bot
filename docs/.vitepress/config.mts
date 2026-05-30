import { defineConfig } from 'vitepress'

export default defineConfig({
  title: '洛玖机器人',
  description: '基于 Napcat OneBot v11 协议的 QQ 机器人框架',
  lang: 'zh-CN',

  head: [
    ['link', { rel: 'icon', href: '/favicon.ico' }],
  ],

  themeConfig: {
    logo: '/favicon.ico',
    siteTitle: 'luo9_bot',

    nav: [
      { text: '指南', link: '/guide/' },
      { text: '插件开发', link: '/sdk/' },
      { text: 'API', link: '/api/' },
      {
        text: '相关链接',
        items: [
          { text: 'GitHub', link: 'https://github.com/luoy-oss/luo9_bot' },
          { text: 'Napcat', link: 'https://napneko.github.io/' },
        ]
      }
    ],

    sidebar: {
      '/guide/': [
        {
          text: '开始洛玖之旅',
          items: [
            { text: '介绍', link: '/guide/' },
            { text: '快速开始', link: '/guide/getting-started' },
            { text: '配置说明', link: '/guide/configuration' },
          ]
        },
        {
          text: '深入了解',
          items: [
            { text: '插件系统', link: '/guide/plugin-system' },
            { text: 'WebUI', link: '/guide/webui' },
            { text: '部署', link: '/guide/deployment' },
          ]
        }
      ],
      '/sdk/': [
        {
          text: '开发插件',
          items: [
            { text: '概述', link: '/sdk/' },
          ]
        },
        {
          text: 'Rust',
          items: [
            { text: '入门', link: '/sdk/rust/' },
            { text: '消息处理', link: '/sdk/rust/messages' },
            { text: '命令解析', link: '/sdk/rust/commands' },
            { text: '定时任务', link: '/sdk/rust/tasks' },
            { text: '技巧与常见问题', link: '/sdk/rust/tips' },
            { text: 'API 速查', link: '/sdk/rust/api' },
          ]
        },
        {
          text: 'C++ (即将推出)',
          items: [
            { text: '入门', link: '/sdk/cpp/' },
          ]
        },
        {
          text: 'Python (即将推出)',
          items: [
            { text: '入门', link: '/sdk/python/' },
          ]
        },
        {
          text: 'SDK 参考',
          items: [
            { text: 'FFI 接口规范', link: '/sdk/ffi-interface' },
            { text: 'Bus 消息总线', link: '/sdk/bus' },
            { text: 'Command 命令解析', link: '/sdk/command' },
            { text: 'Payload 载荷格式', link: '/sdk/payload' },
          ]
        },
        {
          text: '进阶',
          items: [
            { text: '开发新 SDK', link: '/sdk/dev-new-sdk' },
          ]
        }
      ],
      '/api/': [
        {
          text: 'API 参考',
          items: [
            { text: '概述', link: '/api/' },
            { text: 'WebUI API', link: '/api/webui' },
            { text: '事件类型', link: '/api/events' },
          ]
        }
      ]
    },

    socialLinks: [
      { icon: 'github', link: 'https://github.com/luoy-oss/luo9_bot' }
    ],

    footer: {
      message: '基于 GPLv3 许可发布',
      copyright: 'Copyright © 2026 luo9_bot'
    },

    search: {
      provider: 'local',
      options: {
        translations: {
          button: {
            buttonText: '搜索文档',
            buttonAriaLabel: '搜索文档'
          },
          modal: {
            noResultsText: '无法找到相关结果',
            resetButtonTitle: '清除查询条件',
            footer: {
              selectText: '选择',
              navigateText: '切换'
            }
          }
        }
      }
    },

    editLink: {
      pattern: 'https://github.com/luoy-oss/luo9_bot/edit/main/rust/docs/:path',
      text: '在 GitHub 上编辑此页面'
    },

    lastUpdated: {
      text: '最后更新于'
    },

    docFooter: {
      prev: '上一页',
      next: '下一页'
    },

    outline: {
      label: '页面导航'
    },

    returnToTopLabel: '回到顶部',
    sidebarMenuLabel: '菜单',
    darkModeSwitchLabel: '主题',
    lightModeSwitchTitle: '切换到浅色模式',
    darkModeSwitchTitle: '切换到深色模式'
  }
})
