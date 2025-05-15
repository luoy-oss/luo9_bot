from PIL import Image, ImageDraw, ImageFont, ImageFilter
import os
import requests
from io import BytesIO

def __format_duration(seconds=0, minutes=0):
    """将秒转换为 XhYmin 格式"""
    seconds = int(seconds) + int(minutes) * 60
    
    _h = seconds // 3600
    _m = (seconds % 3600) // 60
    if _h > 0:
        return f"{_h}h{_m}min"
    return f"{_m}min"

def __get_avatar_from_url(pic_id):
    """从URL获取头像"""
    url = f"https://playerhub.df.qq.com/playerhub/60004/object/{pic_id}.png"
    try:
        response = requests.get(url, timeout=5)
        response.raise_for_status() # 如果请求失败则引发HTTPError
        avatar_img = Image.open(BytesIO(response.content)).convert("RGBA")
        return avatar_img
    except requests.exceptions.RequestException as e:
        print(f"从URL下载头像失败: {e}")
    except IOError as e:
        print(f"打开下载的头像数据失败: {e}")
    return None

def create_game_stats_image(data, background_image_path=None, output_path='game_stats_output.png'):
    """
    创建游戏数据统计图，包含用户头像、昵称和分类数据展示。

    :param data: 包含玩家所有统计数据的字典。
    :param background_image_path: 背景图片的路径 (可选)。
    :param output_path: 生成图片的保存路径。
    """
    width, height = 800, 750
    background_color = (25, 28, 32)
    padding = 25
    section_padding = 15
    item_padding_y = 10

    img = Image.new('RGB', (width, height), color=background_color)

    # 加载背景图片 (如果提供)
    if background_image_path and os.path.exists(background_image_path):
        try:
            bg_img = Image.open(background_image_path).convert("RGBA")
            # 调整背景图片大小以适应画布，同时保持比例，然后裁剪
            bg_aspect_ratio = bg_img.width / bg_img.height
            canvas_aspect_ratio = width / height
            if bg_aspect_ratio > canvas_aspect_ratio:
                new_height = height
                new_width = int(new_height * bg_aspect_ratio)
            else:
                new_width = width
                new_height = int(new_width / bg_aspect_ratio)
            bg_img = bg_img.resize((new_width, new_height), Image.Resampling.LANCZOS)
            
            # 居中裁剪
            left = (new_width - width) / 2
            top = (new_height - height) / 2
            right = (new_width + width) / 2
            bottom = (new_height + height) / 2
            bg_img = bg_img.crop((left, top, right, bottom))

            # 可以选择模糊背景以突出前景内容
            # bg_img = bg_img.filter(ImageFilter.GaussianBlur(3))
            img.paste(bg_img, (0,0), bg_img if bg_img.mode == 'RGBA' else None)
        except Exception as e:
            print(f"加载背景图片失败: {e}")

    draw = ImageDraw.Draw(img)

    # 字体加载
    try:
        title_font = ImageFont.truetype('msyhbd.ttc', 28) # 微软雅黑粗体
        nickname_font = ImageFont.truetype('msyh.ttc', 32)
        section_title_font = ImageFont.truetype('msyh.ttc', 22)
        stat_label_font = ImageFont.truetype('msyh.ttc', 16)
        stat_value_font = ImageFont.truetype('msyhbd.ttc', 26) # 数值用粗体
    except IOError:
        print("警告: 微软雅黑字体加载失败，将使用默认字体。")
        title_font = ImageFont.load_default()
        nickname_font = ImageFont.load_default()
        section_title_font = ImageFont.load_default()
        stat_label_font = ImageFont.load_default()
        stat_value_font = ImageFont.load_default()

    # 颜色定义
    text_color_light = (230, 230, 230)
    text_color_dim = (180, 180, 180)
    text_color_accent = (100, 220, 180) # 强调色
    section_bg_color = (40, 45, 50, 200) # 半透明区块背景

    current_y = padding

    # 玩家信息区域
    player_data = data.get('player', {})
    user_nickname = player_data.get('charac_name', '未知用户')
    avatar_pic_id = player_data.get('picurl')

    avatar_size = 90
    avatar_x, avatar_y = padding + section_padding, current_y + section_padding

    avatar_img = None
    if avatar_pic_id:
        avatar_img = __get_avatar_from_url(avatar_pic_id)
    
    if avatar_img:
        avatar_img = avatar_img.resize((avatar_size, avatar_size), Image.Resampling.LANCZOS)
        mask = Image.new('L', (avatar_size, avatar_size), 0)
        mask_draw = ImageDraw.Draw(mask)
        mask_draw.ellipse((0, 0, avatar_size, avatar_size), fill=255)
        # 创建一个临时图像用于粘贴头像，避免直接在主图上操作alpha
        temp_avatar_layer = Image.new('RGBA', img.size, (0,0,0,0))
        temp_avatar_layer.paste(avatar_img, (avatar_x, avatar_y), mask)
        img = Image.alpha_composite(img.convert("RGBA"), temp_avatar_layer).convert("RGB")
        draw = ImageDraw.Draw(img) # 重新获取draw对象
    else:
        draw.ellipse([avatar_x, avatar_y, avatar_x + avatar_size, avatar_y + avatar_size], fill=(60,60,60), outline=text_color_dim, width=2)
        draw.text((avatar_x + avatar_size//2, avatar_y + avatar_size//2), "无头像", fill=text_color_dim, font=stat_label_font, anchor='mm')

    nickname_x = avatar_x + avatar_size + 25
    nickname_y = avatar_y + avatar_size // 2
    draw.text((nickname_x, nickname_y), user_nickname, fill=text_color_light, font=nickname_font, anchor='lm')
    current_y += avatar_size + section_padding * 2

    # 辅助函数：绘制带背景的区块和数据项
    def draw_section(title, stats_list, x_start, y_start, section_width, num_cols=2):
        nonlocal draw # 确保使用的是最新的draw对象
        box_padding = 15
        content_x_start = x_start + box_padding
        content_y_start = y_start + box_padding

        # 计算区块高度
        title_bbox = draw.textbbox((0,0), title, font=section_title_font)
        title_height = title_bbox[3] - title_bbox[1] + 10 # 加上一些间距
        
        num_items = len(stats_list)
        items_per_col = (num_items + num_cols -1) // num_cols
        # 确保 item_height_estimate 是整数
        item_height_estimate = int(
            (stat_label_font.getbbox("A")[3] - stat_label_font.getbbox("A")[1]) + \
            (stat_value_font.getbbox("0")[3] - stat_value_font.getbbox("0")[1]) + item_padding_y * 2.5
        )
        
        content_height = items_per_col * item_height_estimate
        section_height = int(title_height + content_height + box_padding * 2)

        # 绘制区块背景
        section_rect_img = Image.new('RGBA', (section_width, section_height), section_bg_color)
        img.paste(section_rect_img, (x_start, int(y_start)), section_rect_img)
        draw = ImageDraw.Draw(img)

        # 绘制区块标题
        draw.text((content_x_start, content_y_start), title, fill=text_color_light, font=section_title_font)
        current_item_y = content_y_start + title_height

        # 绘制统计项
        col_width = (section_width - box_padding * 2) // num_cols
        for i, (label, value) in enumerate(stats_list):
            col_idx = i % num_cols
            row_idx = i // num_cols
            
            item_x = content_x_start + col_idx * col_width
            # 为每列的起始Y坐标独立计算，避免跨列时Y坐标不一致
            # 这里简化处理，假设每行高度一致
            item_y_for_row = int(current_item_y + row_idx * item_height_estimate)

            draw.text((item_x, int(item_y_for_row + item_padding_y)), label, fill=text_color_dim, font=stat_label_font)
            draw.text((item_x, int(item_y_for_row + item_padding_y + 20)), str(value), fill=text_color_accent, font=stat_value_font)
        
        # 确保返回的 Y 坐标是整数
        return int(y_start + section_height + section_padding)

    # 游戏数据区域
    game_data = data.get('game', {})
    assets_data = {
        "哈夫币": data.get('money', 0),
        "三角券": data.get('tickets', 0),
        "三角币": data.get('coin', 0)
    }

    # 烽火地带数据
    fh_stats = [
        ("总对局", game_data.get('soltotalfght', 'N/A')),
        ("撤离成功", game_data.get('solttotalescape', 'N/A')),
        ("撤离率", game_data.get('solescaperatio', 'N/A')),
        ("击败干员", game_data.get('soltotalkill', 'N/A')),
        ("总时长", __format_duration(seconds = game_data.get('solduration', 0))),
        ("总积分", game_data.get('rankpoint', 'N/A'))
    ]

    # 全面战场数据
    qm_stats = [
        ("总对局", game_data.get('tdmtotalfight', 'N/A')),
        ("胜场", game_data.get('totalwin', 'N/A')),
        ("胜率", game_data.get('tdmsuccessratio', 'N/A')),
        ("总击杀", game_data.get('tdmtotalkill', 'N/A')),
        ("分均击杀", f"{game_data.get('avgkillperminute', 0) / 100:.1f}" if game_data.get('avgkillperminute') is not None else 'N/A'),
        ("总时长", __format_duration(minutes = game_data.get('tdmduration', 0))),
        ("总积分", game_data.get('tdmrankpoint', 'N/A'))
    ]

    # 资产数据
    asset_stats_list = [
        ("哈夫币", f"{assets_data['哈夫币']:,}"), # 使用千位分隔符
        ("三角券", f"{assets_data['三角券']:,}"),
        ("三角币", f"{assets_data['三角币']:,}")
    ]

    # 布局区块
    section_width_half = (width - padding * 2 - section_padding) // 2
    section_width_full = width - padding * 2

    # 烽火地带 和 全面战场 并排
    y_start_for_side_by_side_sections = current_y # 保存两个并排区块的共同起始Y坐标

    # 绘制 烽火地带 区块
    # draw_section 函数返回的是该区块结束后，下一个区块（如果直接放在其下方）的起始Y坐标
    fh_actual_end_y = draw_section(
        "烽火地带",
        fh_stats,
        padding,  # x_start
        y_start_for_side_by_side_sections,  # y_start
        section_width_half,
        num_cols=1
    )

    # 绘制 全面战场 区块，与烽火地带从相同的y_start开始
    qm_actual_end_y = draw_section(
        "全面战场",
        qm_stats,
        padding + section_width_half + section_padding,  # x_start (位于烽火地带右侧)
        y_start_for_side_by_side_sections,  # y_start (与烽火地带相同)
        section_width_half,
        num_cols=1
    )
    
    # 更新 current_y，使其位于两个并排区块中较高者的下方。
    current_y = max(fh_actual_end_y, qm_actual_end_y)

    # 资产信息，占满一行
    current_y = draw_section("我的资产", asset_stats_list, padding, int(current_y), section_width_full, num_cols=3)

    # 添加一个小的底部信息或logo（可选）
    try:
        footer_font = ImageFont.truetype('msyh.ttc', 12)
    except IOError:
        footer_font = ImageFont.load_default()
    draw.text((width - padding, height - padding), "[DEBUG]图片由luo9_bot\ndeltaforce插件生成", fill=text_color_dim, font=footer_font, anchor='rd')

    # 保存图片
    try:
        img.save(output_path)
        print(f"图片已保存至: {output_path}")
    except Exception as e:
        print(f"保存图片失败: {e}")
    return img

if __name__ == "__main__":
    full_game_data_example = {
        'player': {
            'picurl': '42010040094',  # 角色ID, 用于获取头像
            'charac_name': ''  # 角色名
        },
        'game': {
            'result': 0,
            'error_info': 0,
            'rankpoint': '0',  # 烽火地带总积分
            'tdmrankpoint': '0',  # 全面战场总积分
            'soltotalfght': '0',  # 烽火地带总对局数
            'solttotalescape': '0',  # 烽火地带撤离成功场数
            'solduration': 0,  # 烽火地带总时长（秒）
            'soltotalkill': '0',  # 烽火地带击败干员数
            'solescaperatio': '0%',  # 烽火地带撤离率
            'avgkillperminute': 0,  # 分均击杀(需/100)
            'tdmduration': 0,  # 全面战场总时长（秒） - 假设原数据是分钟，转换为秒
            'tdmsuccessratio': '0%',  # 全面战场胜率
            'tdmtotalfight': '0',  # 全面战场总对局数
            'totalwin': '0',  # 全面战场胜场
            'tdmtotalkill': 0  # 全面战场总击杀
        },
        'coin': 0,  # 三角币
        'tickets': 0,
        'money': 0  # 哈夫币
    }

    # 确保你有一个名为 placeholder_background.png 的图片在脚本同目录下，或者提供正确路径
    # 或者设置为 None 不使用背景图片
    background_image_example_path = "placeholder_background.png" 
    if not os.path.exists(background_image_example_path):
        print(f"提示: 示例背景图片 {background_image_example_path} 未找到, 将不使用背景图片。")
        background_image_example_path = None
    else:
        print(f"使用背景图片: {background_image_example_path}")

    # 为了能从URL获取头像，请确保网络连接正常
    # 如果 picurl 对应的头像是私有的或无法公开访问，get_avatar_from_url 会失败
    # 在这种情况下，可以修改 get_avatar_from_url 或手动提供本地头像路径
    image = create_game_stats_image(
        full_game_data_example,
        background_image_path=background_image_example_path, 
        output_path='game_stats_detailed_output.png'
    )
