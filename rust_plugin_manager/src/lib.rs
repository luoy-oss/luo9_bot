//! Luo9 Plugin Manager Rust实现
//! 
//! 这个库提供了Luo9机器人的插件管理系统的Rust实现，
//! 旨在提高性能并保持与现有Python代码的兼容性。

use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList, PyModule};
use pyo3::wrap_pyfunction;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

/// 插件配置结构体
#[derive(Debug, Deserialize, Serialize, Clone)]
struct PluginConfig {
    name: String,
    priority: i32,
    enable: bool,
}

/// 插件结构体
#[derive(Debug, Clone)]
struct Plugin {
    name: String,
    describe: String,
    author: String,
    version: String,
    priority: i32,
    message_types: Vec<String>,
    module: PyObject,
}

/// Rust实现的插件管理器
#[pyclass]
struct PluginManager {
    plugin_dir: String,
    plugins: Vec<Plugin>,
    data_path: String,
}

#[pymethods]
impl PluginManager {
    /// 创建新的插件管理器实例
    #[new]
    fn new(plugin_dir: &str) -> PyResult<Self> {
        let mut manager = PluginManager {
            plugin_dir: plugin_dir.to_string(),
            plugins: Vec::new(),
            data_path: String::new(),
        };
        
        // 获取数据路径
        Python::with_gil(|py| {
            let config_module = PyModule::import(py, "config")?;
            let get_value = config_module.getattr("get_value")?;
            let value = get_value.call0()?;
            manager.data_path = value.getattr("data_path")?.extract()?;
            Ok::<_, PyErr>(())
        })?;
        
        // 加载插件
        manager.load_plugins()?;
        Ok(manager)
    }
    
    /// 加载插件
    fn load_plugins(&mut self) -> PyResult<()> {
        Python::with_gil(|py| {
            // 读取配置文件
            let config_path = Path::new(&self.plugin_dir).join("config.yaml");
            let config_content = fs::read_to_string(config_path)?;
            let config: HashMap<String, Vec<PluginConfig>> = serde_yaml::from_str(&config_content)
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Failed to parse config: {}", e)))?;
            
            let plugins_config = &config["plugins"];
            println!("插件总数：{}", plugins_config.len());
            print_flip(py)?;
            
            let mut load_num = 0;
            for plugin_config in plugins_config {
                if plugin_config.enable {
                    let plugin_name = &plugin_config.name;
                    let plugin_path = Path::new(&self.plugin_dir).join(plugin_name);
                    
                    if plugin_path.is_dir() {
                        // 导入插件模块
                        let module_name = format!("plugins.{}.main", plugin_name);
                        let plugin_module = PyModule::import(py, module_name.as_str())?;
                        let config = plugin_module.getattr("config")?;
                        
                        // 创建插件对象
                        let plugin = Plugin {
                            name: plugin_name.clone(),
                            describe: config.get_item("describe")?.extract()?,
                            author: config.get_item("author")?.extract()?,
                            version: config.get_item("version")?.extract()?,
                            module: plugin_module.into(),
                            priority: plugin_config.priority,
                            message_types: config.get_item("message_types")?.extract()?,
                        };
                        
                        // 创建插件数据目录
                        let plugin_data_path = format!("{}/plugins/{}", self.data_path, plugin.name);
                        println!("{}", plugin_data_path);
                        
                        if !Path::new(&plugin_data_path).exists() {
                            fs::create_dir_all(&plugin_data_path)
                                .map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(format!("Failed to create directory: {}", e)))?;
                            
                            // 设置权限（非Windows系统）
                            let platform_module = PyModule::import(py, "platform")?;
                            let system = platform_module.call_method0("system")?;
                            if system.extract::<String>()? != "Windows" {
                                let os_module = PyModule::import(py, "os")?;
                                let stat_module = PyModule::import(py, "stat")?;
                                let s_irwxo = stat_module.getattr("S_IRWXO")?;
                                os_module.call_method1("chmod", (plugin_data_path, s_irwxo))?;
                            }
                        }
                        
                        // 打印插件信息
                        println!("加载插件：{}\n作者：{}\n插件描述：{}\n插件版本：{}\n插件需求：{:?}", 
                            plugin.name, plugin.author, plugin.describe, plugin.version, plugin.message_types);
                        print_flip(py)?;
                        
                        load_num += 1;
                        self.plugins.push(plugin);
                    }
                }
            }
            
            println!("加载完成：{}/{}", load_num, plugins_config.len());
            print_flip(py)?;
            
            Ok(())
        })
    }
    
    /// 处理群组消息
    fn handle_group_message<'py>(&self, py: Python<'py>, message: PyObject) -> PyResult<&'py PyAny> {
        // 按优先级排序插件
        let mut sorted_plugins = self.plugins.clone();
        sorted_plugins.sort_by_key(|p| p.priority);
        
        // 创建异步函数
        pyo3_asyncio::tokio::future_into_py(py, async move {
            // 遍历插件并调用处理函数
            for plugin in sorted_plugins {
                if plugin.message_types.contains(&"group_message".to_string()) {
                    Python::with_gil(|py| {
                        // 获取模块对象
                        let module: PyObject = plugin.module;
                        let module = module.as_ref(py);
                        
                        // 调用group_handle方法
                        // 处理call_method1返回的Result，确保传递给into_future的是&PyAny类型
                        let result = module.call_method1("group_handle", (message.clone_ref(py),))?;
                        pyo3_asyncio::tokio::into_future(result)
                        
                    })?.await?;
                }
            }
            Ok(Python::with_gil(|py| py.None()))
        })
    }
    
    /// 处理私聊消息
    fn handle_private_message<'py>(&self, py: Python<'py>, message: PyObject) -> PyResult<&'py PyAny> {
        // 按优先级排序插件
        let mut sorted_plugins = self.plugins.clone();
        sorted_plugins.sort_by_key(|p| p.priority);
        
        // 创建异步函数
        pyo3_asyncio::tokio::future_into_py(py, async move {
            // 遍历插件并调用处理函数
            for plugin in sorted_plugins {
                if plugin.message_types.contains(&"private_message".to_string()) {
                    Python::with_gil(|py| {
                        // 获取模块对象
                        let module: PyObject = plugin.module;
                        let module = module.as_ref(py);
                        
                        // 调用private_handle方法
                        // 处理call_method1返回的Result，确保传递给into_future的是&PyAny类型
                        let result = module.call_method1("private_handle", (message.clone_ref(py),))?;
                        pyo3_asyncio::tokio::into_future(result)
                    })?.await?;
                }
            }
            Ok(Python::with_gil(|py| py.None()))
        })
    }
    
    /// 处理群组戳一戳事件
    /// 
    /// 将字符串引用转换为拥有所有权的String类型，以解决生命周期问题
    fn handle_group_poke<'py>(&self, py: Python<'py>, target_id: &str, user_id: &str, group_id: &str) -> PyResult<&'py PyAny> {
        // 按优先级排序插件
        let mut sorted_plugins = self.plugins.clone();
        sorted_plugins.sort_by_key(|p| p.priority);
        
        // 将字符串引用转换为拥有所有权的String类型
        let target_id_owned = target_id.to_string();
        let user_id_owned = user_id.to_string();
        let group_id_owned = group_id.to_string();
        
        // 创建异步函数
        pyo3_asyncio::tokio::future_into_py(py, async move {
            // 遍历插件并调用处理函数
            for plugin in sorted_plugins {
                if plugin.message_types.contains(&"group_poke".to_string()) {
                    Python::with_gil(|py| {
                        // 获取模块对象
                        let module: PyObject = plugin.module;
                        let module = module.as_ref(py);
                        
                        let result = module.call_method1("group_poke_handle", (&target_id_owned, &user_id_owned, &group_id_owned))?;
                        pyo3_asyncio::tokio::into_future(result)
                    })?.await?;
                }
            }
            Ok(Python::with_gil(|py| py.None()))
        })
    }
}

/// 打印分隔线
#[pyfunction]
fn print_flip(py: Python) -> PyResult<()> {
    println!("---------------------------");
    Ok(())
}

/// 模块初始化函数
#[pymodule]
fn luo9_plugin_manager(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<PluginManager>()?;
    m.add_function(wrap_pyfunction!(print_flip, m)?)?;
    Ok(())
}