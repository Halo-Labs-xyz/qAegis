// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import {Test} from "forge-std/Test.sol";
import {QRMSOracle} from "../src/QRMSOracle.sol";

contract QRMSOracleTest is Test {
    QRMSOracle public oracle;
    address public qrmUpdater;

    function setUp() public {
        qrmUpdater = address(0x1234);
        oracle = new QRMSOracle(qrmUpdater);
    }

    function test_UpdateRiskScore() public {
        vm.prank(qrmUpdater);
        oracle.updateRiskScore(5000);
        assertEq(oracle.riskScore(), 5000);
    }

    function test_OnlyQRMUpdaterCanUpdate() public {
        vm.expectRevert();
        oracle.updateRiskScore(5000);
    }
}
