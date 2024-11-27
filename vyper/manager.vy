# ported from https://github.com/shafu0x/MicroStable/blob/main/src/MicroStable.sol

from ethereum.ercs import IERC20
import micro_stable

interface Oracle:
    def latest_answer() -> uint256: view

MIN_COLLAT_RATIO: public(constant(uint256)) = 15 * 10**17

WETH: public(immutable(IERC20))
VY_USD: public(immutable(micro_stable.__interface__))
ORACLE: public(immutable(Oracle))


address2deposit: HashMap[address, uint256]
address2minted: HashMap[address, uint256]


@deploy
def __init__(weth: IERC20, vy_usd: micro_stable.__interface__, oracle: Oracle):
    WETH = weth
    VY_USD = vy_usd
    ORACLE = oracle

@external
def deposit(amount: uint256):
    extcall WETH.transferFrom(msg.sender, self, amount)
    self.address2deposit[msg.sender] += amount

@external
def burn(amount: uint256):
    self.address2minted[msg.sender] -= amount
    extcall VY_USD.burn(msg.sender, amount)

@external
def mint(amount: uint256):
    self.address2minted[msg.sender] += amount
    assert self._collatRatio(msg.sender) >= MIN_COLLAT_RATIO
    extcall VY_USD.mint(msg.sender, amount)

@external
def withdraw(amount: uint256):
    self.address2deposit[msg.sender] -= amount
    assert self._collatRatio(msg.sender) >= MIN_COLLAT_RATIO
    extcall WETH.transfer(msg.sender, amount)

@external
def liquidate(user: address):
    assert self._collatRatio(user) >= MIN_COLLAT_RATIO
    extcall VY_USD.burn(msg.sender, self.address2minted[user])
    extcall WETH.transfer(msg.sender, self.address2deposit[user])
    self.address2deposit[user] = 0
    self.address2minted[user] = 0

@external
def collatRatio(user: address) -> uint256:
    return self._collatRatio(user)

def _collatRatio(user: address) -> uint256:
    minted: uint256 = self.address2minted[user]
    if minted == 0:
        return max_value(uint256)

    oracle_value: uint256 = staticcall ORACLE.latest_answer()
    total_value: uint256 = self.address2deposit[user] * oracle_value // (10**18)
    return total_value // minted