# 插件系统

::: tip
想直接写插件？跳到 [Rust 插件开发指南](/sdk/rust-plugin-dev)。
:::

## 一句话概括

宿主负责收发消息，插件负责处理逻辑。它们之间通过消息总线通信。

## 插件是怎么加载的

1. 宿主启动，扫描 `plugins/` 目录
2. 找到 DLL/SO 文件，用 `dlopen` 加载
3. 为每个插件在每个 topic 上创建 subscriber
4. 调用插件的 `plugin_main`，在独立线程里跑起来

插件跑起来后，就进入了消息循环：从总线取消息，处理，可能发个回复，然后继续等下一条。

## 消息怎么分发

宿主收到 QQ 消息后，不会广播给所有插件。它按优先级从高到低，一个一个定向推送：

```
插件 A（优先级 100）→ 收到消息
    ↓ 如果 A 没有设置阻断
插件 B（优先级 50）→ 收到消息
    ↓ 如果 B 没有设置阻断
插件 C（优先级 0）→ 收到消息
```

如果插件 A 设置了 `block_enabled = true`，消息到 A 就停了，B 和 C 收不到。

这个机制让你可以做"消息拦截器"——比如一个安全插件，检查消息内容，觉得有问题就阻断，后面的插件就看不到了。

## 插件怎么回复

插件通过 `luo9_send` topic 发送回复：

```rust
Bot::send_group_msg(group_id, CString::new("你好！").unwrap());
```

宿主会从 `luo9_send` 取出请求，调用 Napcat API 发送。

## 定时任务

插件可以创建定时任务，让宿主在指定时间通知它：

```
插件 → luo9_task_miso: {"action":"schedule","task_name":"my_task","cron":"0 */5 * * * *"}
                                    ↓
                           宿主的 cron 调度器
                                    ↓ 定时触发
插件 ← luo9_task: {"event":"tick","task_name":"my_task"}
```

Cron 表达式是 6 字段格式：`秒 分 时 日 月 周`，支持 `? L W #` 等特殊字符。

## 热重载

更新插件不用停机器人：

1. 禁用插件 → 宿主调用 `unsubscribe_all()`，插件收到哨兵消息，退出循环，线程结束
2. 替换 DLL/SO 文件
3. 启用插件 → 宿主重新加载，创建新 subscriber，spawn 新线程

整个过程对用户透明，插件代码也不需要做任何特殊处理。

## 下一步

- [写个插件](/sdk/rust-plugin-dev) — 实际动手
- [配置说明](/guide/configuration) — 了解优先级和阻断的配置
