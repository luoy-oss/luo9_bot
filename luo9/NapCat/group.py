import requests
from config import get_value
value = get_value()

async def send_group_message(group_id, message):
    url = f"{value.base_url}/send_group_msg"
    params = {
        "group_id": group_id,
        "message": [message],
        "access_token": value.access_token
    }
    
    response = requests.get(url, params=params)
    # response = requests.post(url, data = params)

    if response.status_code == 200:
        print("Message sent successfully")
    else:
        print("Failed to send message")
        print(response.status_code, response.text)

async def send_group_ai_record(group_id, character, text):
    url = f"{value.base_url}/send_group_ai_record"
    params = {
        "group_id": group_id,
        "character": character,
        "text": text,
        "access_token": value.access_token
    }
    response = requests.get(url, params=params)

    if response.status_code == 200:
        print("AI record sent successfully")
    else:
        print("Failed to send AI record")
        print(response.status_code, response.text)
        

async def send_group_at(group_id, qq):
    url = f"{value.base_url}/send_group_msg"
    params = {
        "group_id": group_id,
        "message": ['[CQ:at,qq={qq}]'.format(qq=qq)],
        "access_token": value.access_token
    }
    response = requests.get(url, params=params)

    if response.status_code == 200:
        print("Message sent successfully")
    else:
        print("Failed to send message")
        print(response.status_code, response.text)

async def send_group_image(group_id, file):
    url = f"{value.base_url}/send_group_msg"
    params = {
        "group_id": group_id,
        "message": ['[CQ:image,file={file}]'.format(file=file)],
        "access_token": value.access_token
    }
    response = requests.get(url, params=params)

    if response.status_code == 200:
        print("Message sent successfully")
    else:
        print("Failed to send message")
        print(response.status_code, response.text)

async def upload_group_file(group_id, file, name, folder_id):
    url = f"{value.base_url}/upload_group_file"
    params = {
        "group_id": group_id,
        "file": file,
        "name": name,
        "folder_id": folder_id,
        "access_token": value.access_token
    }
    response = requests.get(url, params=params)

    if response.status_code == 200:
        print("File uploaded successfully")
    else:
        print("Failed to upload file")
        print(response.status_code, response.text)

async def send_group_poke(group_id, user_id):
    if user_id != value.bot_id:
        url = f"{value.base_url}/group_poke"
        params = {
            "group_id": group_id,
            "user_id": user_id,
            "access_token": value.access_token
        }
        print(params)
        response = requests.get(url, params=params)
        print(response.json())

        if response.status_code == 200:
            print("poke sent successfully")
        else:
            print("Failed to send poke")
            print(response.status_code, response.text)

async def get_ai_radio_list(group_id, chat_type):
    url = f"{value.base_url}/get_ai_characters"
    params = {
        "group_id": group_id,
        "chat_type": chat_type,
        "access_token": value.access_token
    }
    response = requests.get(url, params=params)
    print(response.json()['data'])

    # if response.status_code == 200:
    #     print("Message sent successfully")
    # else:
    #     print("Failed to send message")
    #     print(response.status_code, response.text)

async def send_group_ai_radio(group_id, character, text):
    url = f"{value.base_url}/send_group_ai_record"
    params = {
        "group_id": group_id,
        "character": character,
        "text": text,
        "access_token": value.access_token
    }
    response = requests.get(url, params=params)

    if response.status_code == 200:
        print("AI radio sent successfully")
    else:
        print("Failed to send AI radio")
        print(response.status_code, response.text)



async def get_group_files_by_folder(group_id, folder_id, file_count):
    url = f"{value.base_url}/get_group_files_by_folder"
    params = {
        "group_id": group_id,
        "folder_id": folder_id,
        "file_count": file_count,
        "access_token": value.access_token
    }
    response = requests.get(url, params=params)
    if response.status_code == 200:
        print("File list retrieved successfully")
        return response.json()
    else:
        print("Failed to retrieve file list")
        print(response.status_code, response.text)
        return None

async def get_group_root_files(group_id):
    url = f"{value.base_url}/get_group_root_files"
    params = {
        "group_id": group_id,
        "access_token": value.access_token
    }
    response = requests.get(url, params=params)
    if response.status_code == 200:
        print("File list retrieved successfully")
        return response.json()
    else:
        print("Failed to retrieve file list")
        print(response.status_code, response.text)
        return None