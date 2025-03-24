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
    image_paths = []  # 子目录图集路径

    # 收集所有子目录
    with os.scandir(paht) as entries:
        for entry in entries:
            if entry.is_dir():
                zimulu.append(int(entry.name))
    # 对数字进行排序
    zimulu.sort()

    # 收集所有图片路径
    for i in zimulu:
        with os.scandir(paht + "/" + str(i)) as entries:
            for entry in entries:
                if entry.is_file() and "jpg" in entry.name:
                    image_paths.append(paht + "/" + str(i) + "/" + entry.name)

    if not image_paths:
        print("没有找到图片文件")
        return ""

    # 准备PDF文件路径
    pdf_file_path = pdfpath + "/" + pdfname
    if not pdf_file_path.endswith(".pdf"):
        pdf_file_path = pdf_file_path + ".pdf"
    
    # 分批处理图片，每批处理10张
    batch_size = 10
    total_images = len(image_paths)
    
    # 使用第一张图片初始化PDF
    first_img = Image.open(image_paths[0])
    if first_img.mode != "RGB":
        first_img = first_img.convert("RGB")
    first_img.save(pdf_file_path, "PDF")
    first_img.close()
    
    # 分批处理剩余图片
    for i in range(1, total_images, batch_size):
        batch_end = min(i + batch_size, total_images)
        print(f"处理图片 {i+1} 到 {batch_end} (共 {total_images} 张)")
        
        # 创建临时PDF
        temp_pdf = f"{pdf_file_path}.temp"
        
        # 打开当前批次的图片
        batch_images = []
        for j in range(i, batch_end):
            img = Image.open(image_paths[j])
            if img.mode != "RGB":
                img = img.convert("RGB")
            batch_images.append(img)
        
        # 保存当前批次到临时PDF
        batch_images[0].save(temp_pdf, "PDF", save_all=True, append_images=batch_images[1:])
        
        # 关闭所有图片释放内存
        for img in batch_images:
            img.close()
        batch_images.clear()
        
        # 合并PDF
        from PyPDF2 import PdfMerger
        merger = PdfMerger()
        merger.append(pdf_file_path)
        merger.append(temp_pdf)
        merger.write(f"{pdf_file_path}.new")
        merger.close()
        
        # 替换原PDF
        os.remove(pdf_file_path)
        os.rename(f"{pdf_file_path}.new", pdf_file_path)
        os.remove(temp_pdf)
    
    end_time = time.time()
    run_time = end_time - start_time
    print("运行时间：%3.2f 秒" % run_time)
    print("转换完成，文件保存在：%s" % pdf_file_path)
    return pdf_file_path
    
def get_JMComic_pdf(id) -> str:
    config = f"{value.plugin_path}/JMComic/config.yml"
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