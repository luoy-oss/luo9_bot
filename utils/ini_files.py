from configparser import ConfigParser
import os
import stat
import platform


async def 读配置项(file, 节名称, 配置项名称, 默认值=""):
    if not os.path.isfile(file):
        with open(file, 'w'):
            pass
        if platform.system() != 'Windows':
            os.chmod(file, stat.S_IRWXO)

    config = ConfigParser()
    config.read(file)
    读取到的值 = ""
    if config.has_section(节名称):
        if config.has_option(节名称, 配置项名称):
            读取到的值 = config[节名称][配置项名称]
        else:
            print("WARNING: 配置项名称{name}不存在".format(name=配置项名称))
    else:
        print("WARNING: 节名称{name}不存在".format(name=节名称))

    return 读取到的值 if 读取到的值 != "" else 默认值


async def 写配置项(file, 节名称, 配置项名称="", 欲写入值=""):
    if not os.path.isfile(file):
        with open(file, 'w'):
            pass
        if platform.system() != 'Windows':
            os.chmod(file, stat.S_IRWXO)

    config = ConfigParser()
    config.read(file)

    if not config.has_section(节名称):
        config.add_section(节名称)
        if 配置项名称:
            config.set(节名称, 配置项名称, 欲写入值)
    elif 配置项名称 == "":
        config.remove_section(节名称)
    elif 欲写入值 == "":
        config.remove_option(节名称, 配置项名称)
    else:
        config.set(节名称, 配置项名称, 欲写入值)

    with open(file, 'w') as configfile:
        config.write(configfile)


async def 配置项初始化(file, config):
    if not os.path.isfile(file):
        with open(file, 'w'):
            pass
        if platform.system() != 'Windows':
            os.chmod(file, stat.S_IRWXO)

    with open(file, 'w') as configfile:
        config.write(configfile)