// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import "forge-std/Script.sol";
import "../src/PQCVerifier.sol";
import "../src/QRMSOracle.sol";
import "../src/SequencerAttestation.sol";

contract DeployScript is Script {
    function run() external {
        uint256 deployerPrivateKey = vm.envUint("ADMIN_PRIVATE_KEY");
        address qrmUpdater = vm.envAddress("QRM_UPDATER_ADDRESS");
        
        vm.startBroadcast(deployerPrivateKey);
        
        // Deploy PQCVerifier
        PQCVerifier pqcVerifier = new PQCVerifier();
        console.log("PQCVerifier deployed at:", address(pqcVerifier));
        
        // Deploy QRMSOracle
        QRMSOracle qrmsOracle = new QRMSOracle(
            address(pqcVerifier),
            qrmUpdater
        );
        console.log("QRMSOracle deployed at:", address(qrmsOracle));
        
        // Deploy SequencerAttestation
        SequencerAttestation seqAttestation = new SequencerAttestation(
            address(pqcVerifier)
        );
        console.log("SequencerAttestation deployed at:", address(seqAttestation));
        
        vm.stopBroadcast();
        
        // Output addresses for .env
        console.log("\n--- Add to .env ---");
        console.log("PQC_VERIFIER=", address(pqcVerifier));
        console.log("QRMS_ORACLE=", address(qrmsOracle));
        console.log("SEQUENCER_ATTESTATION=", address(seqAttestation));
    }
}
