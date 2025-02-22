import random

def random_run(prob):
    assert prob >= 0 and prob <= 1, "概率prob的值应该处在[0,1]之间！"
    if prob == 0:#概率为0，直接返回False
        return False
    if prob == 1:#概率为1，直接返回True
        return True
    p_digits = len(str(prob).split(".")[1])
    interval_begin = 1
    interval__end = pow(10, p_digits)
    R = random.randint(interval_begin, interval__end)
    if float(R)/interval__end < prob:
        return True
    else:
        return False
