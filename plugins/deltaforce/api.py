import requests
import json
from logger import Luo9Log

log = Luo9Log(__name__)


class DeltaForceAPI:
    """
    三角洲行动API接口类，处理与API的所有交互
    """
    BASE_URL = "https://deltaforce.drluo.top"
    # BASE_URL = "http://127.0.0.1:8000"

    @staticmethod
    async def get_data():
        """
        获取登录数据
        
        Returns:
            dict: 包含登录数据的字典，失败返回None
        """
        try:
            response = requests.get(f"{DeltaForceAPI.BASE_URL}/qq/sig")
            if response.status_code == 200:
                response_json = response.json()
                # log.info(f"三角洲API：获取登录数据成功: {json.dumps(response_json)}")
                return response_json['data']
            else:
                log.error(f"三角洲API：获取登录数据失败，状态码: {response.status_code}")
        except Exception as e:
            log.error(f"三角洲API：请求异常: {str(e)}")
        
        return None
    
    @staticmethod
    async def check_login_status(login_info):
        """
        检查登录状态
        
        Args:
            login_info: 登录信息，包含token、qrSig和loginSig
            
        Returns:
            dict: 登录状态响应，失败返回None
        """

        try:
            response = requests.post(f"{DeltaForceAPI.BASE_URL}/qq/status",
                params={
                    'qrToken': login_info['token'],
                    'qrSig': login_info['qrSig'],
                    'loginSig': login_info['loginSig'],
                },
                data={
                    'cookie': json.dumps(login_info['cookie']),
                }
            )
            print(response.text)
            response = response.json()
            if response['code'] == 0:
                log.info("三角洲API：登录成功")
                return response
            else:
                log.warning(f"三角洲API：未登录: {response['msg']}")
                return response
        except Exception as e:
            log.error(f"三角洲API：检查状态异常: {str(e)}")

        return None
    
    @staticmethod
    async def perform_query(query_type, query_content, login_info):
        """
        执行查询
        
        Args:
            query_type: 查询类型
            query_content: 查询内容
            login_info: 登录信息
            
        Returns:
            dict: 查询响应，失败返回None
        """
        # 对查询内容进行URL编码
        # encoded_query = requests.utils.quote(query_content)
        
        params = {
            'openid': login_info['openid'],
            'access_token': login_info['access_token'],
        }
        print(f"params: {params}")
        # 根据查询类型构建不同的URL
        if query_type == "data":
            query_url = f"{DeltaForceAPI.BASE_URL}/game/{query_type}"
            params['seasonid'] = query_content
        elif query_type == "record":
            query_url = f"{DeltaForceAPI.BASE_URL}/game/{query_type}"
        elif query_type == "items":
            query_url = f"{DeltaForceAPI.BASE_URL}/game/{query_type}"
        elif query_type == "config":
            query_url = f"{DeltaForceAPI.BASE_URL}/game/{query_type}"
        elif query_type == "player":
            query_url = f"{DeltaForceAPI.BASE_URL}/game/{query_type}"
        elif query_type == "price":
            query_url = f"{DeltaForceAPI.BASE_URL}/game/{query_type}?ids=37100500001&recent=1"
        elif query_type == "assets":
            query_url = f"{DeltaForceAPI.BASE_URL}/game/{query_type}"
        elif query_type == "logs":
            # 1是登录日志，2是道具日志，3是哈夫币日志
            query_url = f"{DeltaForceAPI.BASE_URL}/game/{query_type}?type=1"
        elif query_type == "password":
            query_url = f"{DeltaForceAPI.BASE_URL}/game/{query_type}"
        else:
            query_url = f"{DeltaForceAPI.BASE_URL}/game/data"
        
        try:
            print(f"params: {params}")
            response = requests.get(query_url, params=params)
            
            if response.status_code == 200:
                response_json = response.json()
                # log.info(f"三角洲API：查询响应: {response_json}")
                return response_json
            else:
                log.error(f"三角洲API：查询请求失败，状态码: {response.status_code}")
        except Exception as e:
            log.error(f"三角洲API：查询异常: {str(e)}")
        
        return None