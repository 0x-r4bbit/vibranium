pragma solidity ^0.5.0;

import "@some-package/contracts/something.sol";

contract SimpleTestContract {
  uint public value;

  constructor(uint initialValue) public {
    value = initialValue;
  }
}

