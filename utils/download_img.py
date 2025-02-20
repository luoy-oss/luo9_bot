import os
import urllib.request
import hashlib

async def calculate_file_hash(file_path):
    """计算文件的MD5哈希值"""
    hash_md5 = hashlib.md5()
    with open(file_path, "rb") as f:
        for chunk in iter(lambda: f.read(4096), b""):
            hash_md5.update(chunk)
    return hash_md5.hexdigest()

async def download_image_if_needed(message, img_url, save_path):
    """如果本地图片不存在或与网络图片不一致，则下载图片"""
    if os.path.exists(save_path):
        # 计算本地图片的哈希值
        local_hash = await calculate_file_hash(save_path)
        
        # 下载网络图片到临时文件
        temp_path = save_path + ".temp"
        urllib.request.urlretrieve(img_url, temp_path)
        
        # 计算网络图片的哈希值
        remote_hash = await calculate_file_hash(temp_path)
        
        if local_hash == remote_hash:
            # 如果哈希值一致，删除临时文件，无需下载
            os.remove(temp_path)
            print("图片已存在且一致，无需下载")
            return
        else:
            # 如果哈希值不一致，替换本地图片
            os.replace(temp_path, save_path)
            print("图片已更新")
    else:
        # 如果本地图片不存在，直接下载
        urllib.request.urlretrieve(img_url, save_path)
        print("图片已下载")