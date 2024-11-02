from configparser import ConfigParser
import os, stat

async def 读配置项(file, 节名称, 配置项名称, 默认值=""):
    if not os.path.isfile(file):
        os.mknod(file)
        os.chmod(file, stat.S_IRWXO)

    config = ConfigParser()
    config.read(file)
    读取到的值 = ""
    if config.has_section(节名称):
        读取到的值 = config[节名称][配置项名称]
    else:
        print("WARNING: 节名称{name}不存在".format(name = 节名称))    
    if 读取到的值 == "":
        读取到的值 = 默认值
    return 读取到的值

async def 写配置项(file, 节名称, 配置项名称="", 欲写入值=""):
    if not os.path.isfile(file):
        os.mknod(file)
        os.chmod(file, stat.S_IRWXO)
    config = ConfigParser()
    config.read(file)
    if 配置项名称 == "":
        config.remove_section(节名称)
    elif 欲写入值 == "":
        config.remove_option(节名称, 配置项名称)
    else:
        config.set(节名称, 配置项名称, 欲写入值)
    
    # 写入文件
    with open(file, 'w') as configfile:
        config.write(configfile)

async def 配置项初始化(file, config):
    if not os.path.isfile(file):
        os.mknod(file)
        os.chmod(file, stat.S_IRWXO)

    with open(file, 'w') as configfile:
        config.write(configfile)


