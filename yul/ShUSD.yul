// SPDX-License-Identifier: CC0-1.0
object "ShUSD" {

  /**
   * Storage Layout:
   * + (0) address manager;
   * + (1) uint256 totalSupply;
   * + (2) mapping(address => uint256) balanceOf;
   * + (3) mapping(address => mapping(address => uint256)) allowance;
   */

  code {
    let dataOffset := dataoffset("runtime")
    let dataSize := datasize("runtime")
    datacopy(0x00, add(dataOffset, dataSize), 0x20)
    sstore(0x00, mload(0x00)) /* manager */
    datacopy(0x00, dataOffset, dataSize) /* runtimeBytecode */
    return (0x00, dataSize)
  }

  object "runtime" {
    code {
      switch sig()
        case 0x481c6a75 /* manager() */ {
          mstore(0x00, sload(0))
          return(0x00, 0x20)
        }
        case 0x18160ddd /* totalSupply() */ {
          mstore(0x00, sload(1))
          return(0x00, 0x20)
        }
        case 0x70a08231 /* balanceOf(address) */ {
          mstore(0x00, sload(slot_balanceOf(calldataload(0x04))))
          return(0x00, 0x20)
        }
        case 0x40c10f19 /* mint(address,uint256) */ {
          only_manager()

          let amount := calldataload(0x24)
          if iszero(amount) { return(0x00, 0x00) }

          let total_supply := sload(1)
          let total_supply_after := add(amount, total_supply)

          if iszero(gt(total_supply_after, total_supply)) {
            revert(0x00, 0x00)
          }
          sstore(1, total_supply_after)

          let slot := slot_balanceOf(calldataload(0x04))
          sstore(slot, add(sload(slot), amount)) /* checked_totalSupply */

          return(0x00, 0x00)
        }
        case 0x9dc29fac /* burn(address,uint256) */ {
          only_manager()

          let amount := calldataload(0x24)
          if iszero(amount) { return(0x00, 0x00) }

          let slot := slot_balanceOf(calldataload(0x04))

          let bal := sload(slot)
          let bal_after := sub(bal, amount)
          if gt(bal_after, bal) { revert(0x00, 0x00) }

          sstore(slot, bal_after)
          sstore(1, sub(sload(1), amount)) /* checked_balanceAfter */
          return(0x00, 0x00)
        }
        case 0xa9059cbb /* transfer(address,uint256) */ {
          let amount := calldataload(0x24)
          if iszero(amount) {
            mstore(0x00, 0x01)
            return(0x00, 0x20)
          }

          let slot := slot_balanceOf(caller())
          let bal := sload(slot)
          let bal_after := sub(bal, amount)

          if gt(bal_after, bal) { revert(0x00, 0x00) }
          sstore(slot, bal_after)

          slot := slot_balanceOf(calldataload(0x04))
          sstore(slot, add(sload(slot), amount)) /* checked_balanceOf */
          mstore(0x00, 0x01)
          return(0x00, 0x20)
        }
        case 0x23b872dd /* transferFrom(address,address,uint256) */ {
          let from := calldataload(0x04)
          let to := calldataload(0x24)
          let amount := calldataload(0x44)

          if iszero(amount) {
            mstore(0x00, 0x01)
            return(0x00, 0x20)
          }

          if iszero(eq(caller(), from)) {
            let slot := slot_allowance(from, caller())
            let allowance := sload(slot)
            if iszero(eq(allowance, not(0))) {
              let allowance_next := sub(allowance, amount)
              if gt(allowance_next, allowance) { revert(0x00, 0x00) }
              sstore(slot, allowance_next)
            }
          }

          let slot := slot_balanceOf(from)
          let bal := sload(slot)
          let bal_after := sub(bal, amount)
          if gt(bal_after, bal) { revert(0x00, 0x00) }
          sstore(slot, bal_after)

          slot := slot_balanceOf(to)
          sstore(slot, add(sload(slot), amount)) /* checked_balanceOf */

          mstore(0x00, 0x01)
          return(0x00, 0x20)
        }
        case 0xdd62ed3e /* allowance(address, address) */ {
          mstore(0x00, sload(slot_allowance(calldataload(0x04), calldataload(0x24))))
          return(0x00, 0x20)
        }
        case 0x095ea7b3 /* approve(address,uint256)*/ {
          sstore(slot_allowance(caller(), calldataload(0x04)), calldataload(0x24))
          mstore(0x00, 0x01)
          return(0x00, 0x20)
        }
        case 0x06fdde03 /* name() */ {
          mstore(0x20, 0x20)
          mstore(0x49, 0x09536861667520555344) /* "Shafu USD" */
          return(0x20, 0x60)
        }
        case 0x95d89b41 /* symbol() */ {
          mstore(0x20, 0x20)
          mstore(0x45, 0x057368555344) /* "shUSD" */
          return(0x20, 0x60)
        }
        case 0x313ce567 /* decimals() */ {
          mstore(0x00, 0x12) /* 18 */
          return(0x00, 0x20)
        }
        default {
          mstore(0x00, 0x7352d91c) /* InvalidSelector() */
          revert(0x1c, 0x04)
        }
      function sig() -> e {
        e := shr(0xe0, calldataload(0))
      }
      function slot_balanceOf(account) -> e {
        mstore(0x00, 0x02)
        mstore(0x20, account)
        e := keccak256(0, 0x40)
      }
      function slot_allowance(account, spender) -> e {
        mstore(0x00, 0x03)
        mstore(0x20, account)
        mstore(0x40, spender)
        e := keccak256(0, 0x60)
      }
      function only_manager() {
        if iszero(eq(sload(0), caller())) {
          revert(0x00, 0x00)
        }
      }
    }

  }

}

