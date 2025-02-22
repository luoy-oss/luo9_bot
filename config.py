import yaml
import os

class Value:
    def __init__(self, config):
        try:
            # NapCatQQ API的基础URL
            self.PATH = config['PATH']
            self.data_path = config['PATH'] + '/data'
            self.plugin_path = config['PATH'] + '/plugins'
            self.core_path = config['PATH'] + '/luo9'
            
            self.bot_id = config['bot_id']
            self.master = config['master']

            self.group_list = config['group_list']

            self.B站直播检测推送列表 = config['B站直播检测推送列表']
            self.节日检测推送列表 = config['节日检测推送列表']

            self.土豆直播间ID = config['土豆直播间ID']

            self.AI语音音色 = config['AI语音音色']

            self.napcat = config['napcat']['enable']

            if self.napcat:
                self.ncs_host = str(config['napcat']['httpServers']['host'])
                self.ncs_port = str(config['napcat']['httpServers']['port'])
                self.ncs_token = str(config['napcat']['httpServers']['token'])

                self.ncc_host = str(config['napcat']['httpClients']['host'])
                self.ncc_port = str(config['napcat']['httpClients']['port'])
                self.ncc_token = str(config['napcat']['httpClients']['token'])

        except KeyError:
            print("请检查config.yaml文件是否缺少配置项")
            exit(0)

    @property
    # bot 消息推送基础url
    def base_url(self):
        if self.napcat:
            return self.ncs_host + ":" + self.ncs_port
        else:
            return ''

    @property
    # bot token
    def access_token(self):
        if self.napcat:
            return self.ncs_token
        else:
            return ''


# 全局变量存储配置
config = {}

def load_config(file_path='config.yaml'):
    print('load_config')
    if not os.path.exists(file_path):
        print("请检查config.(example).yaml同级目录下的配置文件", file_path, "是否创建")
        exit(0)
    global config
    with open(file_path, 'r', encoding='utf-8') as file:
        config = yaml.safe_load(file)
    print(config)


def get_config():
    return config

def get_value():
    return Value(config)

# def update_config(key, value):
#     global config
#     config[key] = value

# def save_config(file_path='config.yaml'):
#     with open(file_path, 'w', encoding='utf-8') as file:
#         yaml.safe_dump(config, file, default_flow_style=False, allow_unicode=True)