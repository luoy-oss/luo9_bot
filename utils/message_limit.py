from datetime import datetime 
class MessageLimit:
    def __init__(self, tag):
        self.time_now = datetime.now()
        self.time_before = datetime(2000, 1, 1, 0, 0, 0, 0) 
        self.tag = tag

    def check(self, seconds):
        self.handle()
        if (self.time_now - self.time_before).seconds > seconds:
            self.time_before = datetime.now()
            return True
        else:
            return False
    def handle(self):
        self.time_now = datetime.now()
    def get_tag(self):
        return self.tag