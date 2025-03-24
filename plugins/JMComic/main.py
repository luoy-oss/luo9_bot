import re
import os
import time
import yaml
import jmcomic
from PIL import Image
from luo9.api_manager import luo9
from luo9.message import GroupMessage
import utils.download_img as uimg
from utils.message_limit import MessageLimit
from config import get_value
value = get_value()

config = {
    'name': 'JMComic',
    'describe': 'JMComic (禁漫天堂) 本子下载',
    'author': 'drluo',
    'version': '1.0.0',
    'message_types': ['group_message']
}

def all2PDF(input_folder, pdfpath, pdfname):
    start_time = time.time()
    paht = input_folder
    zimulu = []  # 子目录（里面为image）
    image = []  # 子目录图集
    sources = []  # pdf格式的图

    with os.scandir(paht) as entries:
        for entry in entries:
            if entry.is_dir():
                zimulu.append(int(entry.name))
    # 对数字进行排序
    zimulu.sort()

    for i in zimulu:
        with os.scandir(paht + "/" + str(i)) as entries:
            for entry in entries:
                if entry.is_dir():
                    print("这一级不应该有自录")
                if entry.is_file():
                    image.append(paht + "/" + str(i) + "/" + entry.name)

    if "jpg" in image[0]:
        output = Image.open(image[0])
        image.pop(0)

    for file in image:
        if "jpg" in file:
            img_file = Image.open(file)
            if img_file.mode == "RGB":
                img_file = img_file.convert("RGB")
            sources.append(img_file)

    pdf_file_path = pdfpath + "/" + pdfname
    if pdf_file_path.endswith(".pdf") == False:
        pdf_file_path = pdf_file_path + ".pdf"
    output.save(pdf_file_path, "pdf", save_all=True, append_images=sources)
    end_time = time.time()
    run_time = end_time - start_time
    print("运行时间：%3.2f 秒" % run_time)
    print("转换完成，文件保存在：%s" % pdf_file_path)
    return pdf_file_path
    
def get_JMComic_pdf(id) -> str:
    config = f"{value.plugin_path}/JMComic/config.yaml"
    loadConfig = jmcomic.JmOption.from_file(config)
    jmcomic.download_album(id,loadConfig)

    with open(config, "r", encoding="utf8") as f:
        data = yaml.load(f, Loader=yaml.FullLoader)
        path = f"{value.data_path}/plugins/JMComic"

    pdf_file_path = ""
    with os.scandir(path) as entries:
        for entry in entries:
            if entry.is_dir() and entry.name == f"{id}":
                print(entry.name)
                if os.path.exists(os.path.join(path +'/' +entry.name + ".pdf")):
                    print("文件：《%s》 已存在，跳过" % entry.name)
                    pdf_file_path = path +'/' +entry.name + ".pdf"
                else:
                    print("开始转换：%s " % entry.name)
                    pdf_file_path = all2PDF(path + "/" + entry.name, path, entry.name)
                break
    print("pdf_file_path:", pdf_file_path)
    return pdf_file_path

jm_limit = MessageLimit('JMComic')
async def group_handle(message: GroupMessage):
    group_id = message.group_id
    content = message.content.strip()

    # 匹配 /jm id 或 /jmid 格式的指令
    jm_pattern = re.compile(r'/jm\s*(\d+)|/jm(\d+)')
    match = jm_pattern.search(content)
    
    if match and jm_limit.check(3):
        # 获取匹配到的ID (可能在第一个或第二个捕获组中)
        comic_id = match.group(1) or match.group(2)
        if comic_id:
            response = await luo9.get_group_files_by_folder(group_id, "/084d297e-ab55-4743-8aab-d1a6d08596e3", 10)
            for file in response['data']['files']:
                if file['file_name'] == f"{comic_id}.pdf":
                    # 找到了同名的文件，跳过下载操作
                    await luo9.send_group_message(group_id, f"漫画 {comic_id} 在群文件中已存在\n请前往JMComic目录查看")
                    return

            # 没有找到同名的文件，执行上传操作
            await luo9.send_group_message(group_id, f"开始下载漫画 ID: {comic_id}，请稍候...")
            pdf_file_path = get_JMComic_pdf(comic_id)
            # 利用luo9.get_group_root_files(group_id)查找到：
            # JMComic文件夹id：/084d297e-ab55-4743-8aab-d1a6d08596e3
            await luo9.send_group_message(group_id, f"漫画 {comic_id} 下载完成，正在上传...")
            file_name = f"{comic_id}.pdf"
            await luo9.send_group_file(group_id, pdf_file_path, file_name, "/084d297e-ab55-4743-8aab-d1a6d08596e3")
        else:
            await luo9.send_group_message(group_id, "请提供有效的漫画ID，格式: /jm ID 或 /jmID")