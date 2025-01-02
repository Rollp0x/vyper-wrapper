# @blue_print
# pragma version 0.3.10

x: public(uint256)

@external
def __init__(_x: uint256):
    self.x = _x

@external
def set_x(_new_x: uint256):
    self.x = _new_x