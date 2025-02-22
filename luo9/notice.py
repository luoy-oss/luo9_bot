from config import get_value
value = get_value()



from luo9.plugin_manager import plugin_manager

async def poke_handle(target_id, user_id, group_id=''):
    # 群戳一戳
    if group_id != '':
        await plugin_manager.handle_group_poke(target_id, user_id, group_id)
    # 私聊戳一戳
    else:
        pass
        
async def notice_handle(message_objects):
    if message_objects['notice_type'] == 'group_increase':
        group_id = message_objects['group_id']
        user_id = message_objects['user_id']
        # await luo9.send_group_message(group_id, f"欢迎新成员 [CQ:at,qq={user_id}] 加入群聊！")
    elif message_objects['notice_type'] == 'group_decrease':
        group_id = message_objects['group_id']
        user_id = message_objects['user_id']
        sub_type = message_objects['sub_type']
        # if sub_type == 'leave':
        #     await luo9.send_group_message(group_id, f"成员 [CQ:at,qq={user_id}] 已主动退出群聊。")
        # elif sub_type == 'kick':
        #     operator_id = message_objects['operator_id']
        #     await luo9.send_group_message(group_id, f"成员 [CQ:at,qq={user_id}] 被管理员 [CQ:at,qq={operator_id}] 踢出群聊。")
    elif message_objects['notice_type'] == 'notify':
        # 戳一戳(需要PacketServer才能进行回复)
        if message_objects['sub_type'] == 'poke':
            group_id = message_objects['group_id']
            target_id = message_objects['target_id']
            user_id = message_objects['user_id']
            if 'group_id' in message_objects:
                await poke_handle(target_id, user_id, group_id)            
            else:
                await poke_handle(target_id, user_id)

        pass
        