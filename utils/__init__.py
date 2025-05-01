__all__ = ['ini_files', 'checksys', 'random_runre', 'message_limit', 'record', 'download_img']

def data_encode(data_json):
    """
    数据编码
    将原始json数据各参数类型转为str字符串
    """
    data_str = {}
    for key in data_json:
        if type(data_json[key]) == dict:
            data_str[key] = data_encode(data_json[key])
        else:
            data_str[key] = str(data_json[key])
    return data_str
