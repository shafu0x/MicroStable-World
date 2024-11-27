# ported from https://github.com/shafu0x/MicroStable/blob/main/src/MicroStable.sol

from snekmate.tokens import erc20
from snekmate.auth import ownable

initializes: ownable
initializes: erc20[ownable := ownable]
exports: erc20.IERC20

MANAGER: immutable(address)

@deploy
def __init__(manager: address):
    ownable.__init__()
    erc20.__init__("Vyper USD", "VyUSD", 18, "", "")

    MANAGER = manager

@external
def mint(to: address, amount: uint256):
    assert msg.sender == MANAGER
    erc20._mint(to, amount)

@external
def burn(to: address, amount: uint256):
    assert msg.sender == MANAGER
    erc20._burn(to, amount)