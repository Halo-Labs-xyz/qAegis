// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import {Test} from "forge-std/Test.sol";
import {QRMSOracle} from "../src/QRMSOracle.sol";
import {PQCVerifier} from "../src/PQCVerifier.sol";

contract QRMSOracleTest is Test {
    QRMSOracle public oracle;
    PQCVerifier public verifier;
    address public qrmUpdater;

    function setUp() public {
        qrmUpdater = address(0x1234);
        verifier = new PQCVerifier();
        oracle = new QRMSOracle(address(verifier), qrmUpdater);
    }

    function test_UpdateRiskScore() public {
        bytes memory attestation = abi.encodePacked("risk_attestation");
        PQCVerifier.DualSignature memory sig = PQCVerifier.DualSignature({
            mldsa: "",
            slhdsa: ""
        });

        vm.prank(qrmUpdater);
        oracle.updateRiskScore(5000, attestation, sig);
        assertEq(oracle.getRiskScore(), 5000);
    }

    function test_OnlyQRMUpdaterCanUpdate() public {
        bytes memory attestation = abi.encodePacked("risk_attestation");
        PQCVerifier.DualSignature memory sig = PQCVerifier.DualSignature({
            mldsa: "",
            slhdsa: ""
        });
        vm.expectRevert("QRMSOracle: unauthorized");
        oracle.updateRiskScore(5000, attestation, sig);
    }
}
