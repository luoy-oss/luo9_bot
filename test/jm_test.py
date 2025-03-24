import jmcomic  # 导入此模块，需要先安装.

def get_JMComic_pdf(id = '408778') -> str:
    config = f"H:/luo9/luo9_bot/test/JMComic/config.yml"
    loadConfig = jmcomic.JmOption.from_file(config)
    #如果需要下载，则取消以下注释
    manhua = ['408778']
    for id in manhua:
        jmcomic.download_album(id,loadConfig)

get_JMComic_pdf('408778')