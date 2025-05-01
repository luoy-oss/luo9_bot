import logging
import os
import colorlog
from logging.handlers import RotatingFileHandler
from datetime import datetime
from config import get_value
value = get_value()

log_path = value.log_path
if not os.path.exists(log_path): os.mkdir(log_path)  # 若不存在logs文件夹，则自动创建

log_colors_config = {
    # 终端输出日志颜色
    'DEBUG': 'white',
    'INFO': 'cyan',
    'WARNING': 'yellow',
    'ERROR': 'red',
    'CRITICAL': 'bold_red',
}

# ➢ ➣ ➤ ➥ ➦ ➧ ➨ ➩ ➪ ➫ ➬ ➭ ➮ ➯ ➰ ➱ ➲ ➳ ➴ ➵ ➶ ➷ ➸ ➹ ➺ ➻ ➼ ➽ ➾ ➿
# ⌬ ⌭ ⌮ ⌯ ⌰ ⌱ ⌲ ⌳ ⌴ ⌵ ⌶ ⌷ ⌸ ⌹ ⌺ ⌻ ⌼ ⌽ ⌾ ⌿
default_formats = {
    # 终端输出格式
    # 'color_format': '%(log_color)s%(asctime)s ➢ %(name)s ➢ %(levelname)s ➢ %(message)s',
    'color_format': '%(log_color)s%(levelname)s ➢ %(message)s',
    # 日志输出格式
    'log_format': '%(asctime)s ➢ %(name)s ➢ %(levelname)s ➢ %(message)s'
}



class Luo9Log:
    """
    先创建日志记录器（logging.getLogger），然后再设置日志级别（logger.setLevel），
    接着再创建日志文件，也就是日志保存的地方（logging.FileHandler），然后再设置日志格式（logging.Formatter），
    最后再将日志处理程序记录到记录器（addHandler）
    """

    def __init__(self, name: str):
        self.__name = name
        self.__now_time = datetime.now().strftime('%Y-%m-%d')
        self.__all_log_path = os.path.join(log_path, self.__now_time + "-all" + ".log")
        self.__error_log_path = os.path.join(log_path, self.__now_time + "-error" + ".log")
        self.__logger = logging.getLogger(name)
        self.__logger.setLevel(logging.DEBUG)
        self.__handlers = []
        self.__init_handlers()

    def __init_handlers(self):
        """初始化handler并添加到logger"""
        all_logger_handler = self.__init_logger_handler(self.__all_log_path)
        error_logger_handler = self.__init_logger_handler(self.__error_log_path)
        console_handle = self.__init_console_handle()

        self.__set_log_formatter(all_logger_handler)
        self.__set_log_formatter(error_logger_handler)
        self.__set_color_formatter(console_handle, log_colors_config)

        self.__set_log_handler(all_logger_handler)
        self.__set_log_handler(error_logger_handler, level=logging.ERROR)
        self.__set_color_handle(console_handle)
        
        # 保存handlers引用，便于后续更新
        self.__handlers = [all_logger_handler, error_logger_handler]

    def get_logger(self):
        return self.__logger

    @staticmethod
    def __init_logger_handler(log_path):
        """
        创建日志记录器handler，用于收集日志
        :param log_path: 日志文件路径
        :return: 日志记录器
        """
        # 写入文件，如果文件超过1M大小时，切割日志文件，仅保留3个文件
        logger_handler = RotatingFileHandler(filename=log_path, maxBytes=1 * 1024 * 1024, backupCount=3, encoding='utf-8')
        return logger_handler

    @staticmethod
    def __init_console_handle():
        """创建终端日志记录器handler，用于输出到控制台"""
        console_handle = colorlog.StreamHandler()
        return console_handle

    def __set_log_handler(self, logger_handler, level=logging.DEBUG):
        """
        设置handler级别并添加到logger收集器
        :param logger_handler: 日志记录器
        :param level: 日志记录器级别
        """
        logger_handler.setLevel(level=level)
        self.__logger.addHandler(logger_handler)

    def __set_color_handle(self, console_handle):
        """
        设置handler级别并添加到终端logger收集器
        :param console_handle: 终端日志记录器
        :param level: 日志记录器级别
        """
        console_handle.setLevel(logging.DEBUG)
        self.__logger.addHandler(console_handle)

    @staticmethod
    def __set_color_formatter(console_handle, color_config):
        """
        设置输出格式-控制台
        :param console_handle: 终端日志记录器
        :param color_config: 控制台打印颜色配置信息
        :return:
        """
        formatter = colorlog.ColoredFormatter(default_formats["color_format"], log_colors=color_config)
        console_handle.setFormatter(formatter)

    @staticmethod
    def __set_log_formatter(file_handler):
        """
        设置日志输出格式-日志文件
        :param file_handler: 日志记录器
        """
        formatter = logging.Formatter(default_formats["log_format"], datefmt='%a, %d %b %Y %H:%M:%S')
        file_handler.setFormatter(formatter)

    @staticmethod
    def __close_handler(file_handler):
        """
        关闭handler
        :param file_handler: 日志记录器
        """
        file_handler.close()

    def __check_date(self):
        """检查日期是否变更，如果变更则更新日志文件"""
        current_date = datetime.now().strftime('%Y-%m-%d')
        if current_date != self.__now_time:
            # 日期已变更，更新日志文件路径
            self.__now_time = current_date
            self.__all_log_path = os.path.join(log_path, self.__now_time + "-all" + ".log")
            self.__error_log_path = os.path.join(log_path, self.__now_time + "-error" + ".log")
            
            # 移除旧的handlers
            for handler in self.__handlers:
                self.__logger.removeHandler(handler)
                handler.close()
            
            # 创建新的handlers
            self.__handlers = []
            all_logger_handler = self.__init_logger_handler(self.__all_log_path)
            error_logger_handler = self.__init_logger_handler(self.__error_log_path)
            
            self.__set_log_formatter(all_logger_handler)
            self.__set_log_formatter(error_logger_handler)
            
            self.__set_log_handler(all_logger_handler)
            self.__set_log_handler(error_logger_handler, level=logging.ERROR)
            
            # 更新handlers引用
            self.__handlers = [all_logger_handler, error_logger_handler]
    
    def write(self, message):
        if message == '\n' or message == '\r\n':
            return
        self.__console('info', message)

    def __console(self, level, messages):
        """记录日志"""
        # 检查日期是否变更
        self.__check_date()
        message = ' '.join(messages)
        if level == 'info':
            self.__logger.info(message)
        elif level == 'debug':
            self.__logger.debug(message)
        elif level == 'warning':
            self.__logger.warning(message)
        elif level == 'error':
            self.__logger.error(message)
        elif level == 'critical':
            self.__logger.critical(message)

    def debug(self, messages):
        if type(messages) != list:
            messages = [messages]
        self.__console('debug', messages)

    def info(self, messages):
        if type(messages) != list:
            messages = [messages]
        self.__console('info', messages)

    def warning(self, messages):
        if type(messages) != list:
            messages = [messages]
        self.__console('warning', messages)

    def error(self, messages):
        if type(messages) != list:
            messages = [messages]
        self.__console('error', messages)

    def critical(self, messages):
        if type(messages) != list:
            messages = [messages]
        self.__console('critical', messages)