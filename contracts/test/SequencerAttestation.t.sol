// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import {Test} from "forge-std/Test.sol";
import {SequencerAttestation} from "../src/SequencerAttestation.sol";
import {PQCVerifier} from "../src/PQCVerifier.sol";

contract SequencerAttestationTest is Test {
    SequencerAttestation public attestation;
    PQCVerifier public verifier;

    function setUp() public {
        verifier = new PQCVerifier();
        attestation = new SequencerAttestation(address(verifier));
    }

    function test_RegistersSequencerByAdmin() public {
        address seq = address(0xBEEF);
        bytes32 mrenclave = keccak256("mrenclave");
        attestation.registerSequencer(seq, mrenclave, hex"01", hex"02");
        assertTrue(attestation.isSequencerActive(seq));
    }
}
