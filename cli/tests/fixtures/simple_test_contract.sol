pragma solidity ^0.5.0;

contract SimpleTestContract {
  uint public value;

  constructor(uint initialValue) public {
    value = initialValue;
  }
}
