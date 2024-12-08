object "Manager" {

  /**
   * Storage Layout:
   * + (0) address weth;
   * + (1) address shUSD;
   * + (2) address oracle;
   * + (3) mapping(address => uint) public address2deposit;
   * + (4) mapping(address => uint) public address2minted;
   */

  code {
    let dataOffset := dataoffset("runtime")
    let dataSize := datasize("runtime")
    datacopy(0x00, add(dataOffset, dataSize), 0x60)
    sstore(0x00, mload(0x00)) /* weth */
    sstore(0x01, mload(0x20)) /* shUSD */
    sstore(0x02, mload(0x40)) /* oracle */
    datacopy(0x00, dataOffset, dataSize) /* runtimeBytecode */
    return (0x00, dataSize)
  }

  object "runtime" {
    code {
      switch sig()
        case 0x4c3b7ee7 /* MIN_COLLAT_RATIO */ {
          mstore(0x00, 0x14d1120d7b160000)
          return(0x00, 0x20)
        }
        case 0x3fc8cef3 /* weth() */ {
          mstore(0x00, sload(0x00))
          return(0x00, 0x20)
        }
        case 0x1f5036c8 /* shUSD() */ {
          mstore(0x00, sload(0x01))
          return(0x00, 0x20)
        }
        case 0x7dc0d1d0 /* oracle() */ {
          mstore(0x00, sload(0x02))
          return(0x00, 0x20)
        }
        case 0xdfb1d156 /* address2deposit(address) */ {
          mstore(0x00, sload(slot_address2deposit(calldataload(0x04))))
          return(0x00, 0x20)
        }
        case 0xa02081bc /* address2minted(address) */ {
          mstore(0x00, sload(slot_address2minted(calldataload(0x04))))
          return(0x00, 0x20)
        }
        case 0xb6b55f25 /* deposit(uint256) */ {
          let ptr := mload(0x40)
          let amount := calldataload(0x04)
          mstore(ptr, shl(0xe0, 0x23b872dd))
          mstore(add(ptr, 0x04), caller())
          mstore(add(ptr, 0x24), address())
          mstore(add(ptr, 0x44), amount)

          let success := call(gas(), sload(0x00), 0, ptr, 0x64, 0x00, 0x00) // TODO: missing safe transfer
          if iszero(success) { revert (0x00, 0x00) }

          let slot := slot_address2deposit(caller())
          sstore(slot, add(sload(slot), amount))

          return (0x00, 0x00)
        }
        case 0x42966c68 /* burn(uint256) */ {
          let amount := calldataload(0x04)

          if iszero(amount) { return (0x00, 0x00) }

          let slot := slot_address2minted(caller())

          let minted := sload(slot)
          let minted_after := sub(minted, amount)

          if gt(minted_after, minted) { revert(0x00, 0x00) }
          sstore(slot, minted_after)

          let ptr := mload(0x40)
          mstore(ptr, shl(0xe0, 0x9dc29fac))
          mstore(add(ptr, 0x04), caller())
          mstore(add(ptr, 0x24), amount)

          let success := call(gas(), sload(0x01), 0, ptr, 0x44, 0x00, 0x00)
          if iszero(success) { revert (0x00, 0x00) }

          return (0x00, 0x00)

        }
        case 0xa0712d68 /* mint(uint256) */ {
          let amount := calldataload(0x04)
          if iszero(amount) { return (0x00, 0x00) }

          let slot := slot_address2minted(caller())
          let minted := sload(slot)
          let minted_after := add(minted, amount)

          if gt(minted, minted_after) {
            revert (0x00, 0x00)
          }

          if gt(0x14d1120d7b160000, get_collateral_ratio(sload(slot_address2deposit(caller())), minted_after)) {
            revert(0x00, 0x00)
          }

          sstore(slot, minted_after)

          let ptr := mload(0x40)
          mstore(ptr, shl(0xe0, 0x40c10f19))
          mstore(add(ptr, 0x04), caller())
          mstore(add(ptr, 0x24), amount)

          let success := call(gas(), sload(0x01), 0, ptr, 0x44, 0x00, 0x00)
          if iszero(success) { revert (0x00, 0x00) }

          return (0x00, 0x00)
        }
        case 0x2e1a7d4d /* withdraw(uint256) */ {
          let amount := calldataload(0x04)
          if iszero(amount) { return (0x00, 0x00) }

          let slot := slot_address2deposit(caller())
          let deposit := sload(slot)
          let deposit_after := sub(deposit, amount)
          if gt(deposit_after, deposit) { revert (0x00, 0x00) }

          if gt(0x14d1120d7b160000, get_collateral_ratio(deposit_after, sload(slot_address2minted(caller())))) {
            revert(0x00, 0x00)
          }

          sstore(slot, deposit_after)

          let ptr := mload(0x40)
          mstore(ptr, shl(0xe0, 0xa9059cbb))
          mstore(add(ptr, 0x04), caller())
          mstore(add(ptr, 0x24), amount)

          let success := call(gas(), sload(0x00), 0, ptr, 0x44, 0x00, 0x20) // TODO: missing safe transfer
          if iszero(success) { revert (0x00, 0x00) }

          return (0x00, 0x00)
        }
        case 0x2f865568 /* liquidate(address) */ {
          let user := calldataload(0x04)
          let slot_minted := slot_address2minted(user)
          let slot_deposit := slot_address2deposit(user)

          let minted := sload(slot_minted)
          let deposit := sload(slot_deposit)

          if iszero(gt(0x14d1120d7b160000, get_collateral_ratio(deposit, minted))) { revert(0x00, 0x00) }

          sstore(slot_minted, 0)
          sstore(slot_deposit, 0)

          let ptr := mload(0x40)
          mstore(ptr, shl(0xe0, 0x9dc29fac))
          mstore(add(ptr, 0x04), user)
          mstore(add(ptr, 0x24), minted)

          if iszero(call(gas(), sload(0x01), 0, ptr, 0x44, 0x00, 0x00)) { revert (0x00, 0x00) }

          mstore(ptr, shl(0xe0, 0xa9059cbb))
          mstore(add(ptr, 0x04), caller())
          mstore(add(ptr, 0x24), deposit)

          if iszero(call(gas(), sload(0x00), 0, ptr, 0x44, 0x00, 0x00)) { revert (0x00, 0x00) }

          return (0x00, 0x00)
        }
        case 0x6f370a09 /* collatRatio(address) */ {
          let user := calldataload(0x04)
          let minted := sload(slot_address2minted(user)) 
          mstore(0x00, get_collateral_ratio(sload(slot_address2deposit(user)), minted))
          return (0x00, 0x20)
        }
        default {
          mstore(0x00, 0x7352d91c)  /* InvalidSelector() */
          revert(0x1c, 0x04)
        }

      function sig() -> s {
        s := shr(0xe0, calldataload(0))
      }
      function slot_address2deposit(addr) -> e {
        mstore(0x00, 0x03)
        mstore(0x20, addr)
        e := keccak256(0, 0x40)
      }
      function slot_address2minted(addr) -> e {
        mstore(0x00, 0x04)
        mstore(0x20, addr)
        e := keccak256(0, 0x40)
      }
      function get_collateral_ratio(deposit, minted) -> e {
          e := not(0)
          if gt(minted, 0) {
            mstore(0x00, shl(0xe0, 0x50d25bcd))
            let success := call(gas(), sload(0x02), 0, 0x00, 0x04, 0x00, 0x20)
            if iszero(success) { revert (0x00, 0x00) }
            let current_answer := mload(0x00)
            e := div(
              mul(mul(deposit, current_answer), 0x2540be400),
              mul(minted, 0xde0b6b3a7640000)
            )
          }
      }
    }

  }

}


