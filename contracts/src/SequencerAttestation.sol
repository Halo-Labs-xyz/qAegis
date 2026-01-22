// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import "./PQCVerifier.sol";

/// @title SequencerAttestation
/// @notice Verifies TEE attestations from QRMS sequencer
/// @dev Batches are PQC-signed within TEE enclave
contract SequencerAttestation {
    // ========================================================================
    // Types
    // ========================================================================
    
    struct Attestation {
        bytes32 mrenclave;          // Enclave measurement
        bytes32 mrsigner;           // Signer measurement
        uint64 timestamp;           // Attestation time
        bytes32 batchHash;          // Hash of batch data
        uint256 batchNumber;        // Batch sequence number
        PQCVerifier.DualSignature signature;
    }
    
    struct SequencerInfo {
        bytes32 mrenclave;          // Expected enclave measurement
        PQCVerifier.DualPublicKey publicKey;
        bool active;
        uint256 registeredAt;
    }
    
    // ========================================================================
    // State
    // ========================================================================
    
    PQCVerifier public immutable pqcVerifier;
    
    mapping(address => SequencerInfo) public sequencers;
    address[] public sequencerList;
    
    mapping(uint256 => bytes32) public verifiedBatches;
    uint256 public lastVerifiedBatch;
    
    address public admin;
    
    // ========================================================================
    // Events
    // ========================================================================
    
    event SequencerRegistered(
        address indexed sequencer,
        bytes32 mrenclave
    );
    
    event SequencerDeactivated(address indexed sequencer);
    
    event BatchVerified(
        uint256 indexed batchNumber,
        bytes32 batchHash,
        address sequencer
    );
    
    event AttestationFailed(
        uint256 indexed batchNumber,
        string reason
    );
    
    // ========================================================================
    // Constructor
    // ========================================================================
    
    constructor(address _pqcVerifier) {
        pqcVerifier = PQCVerifier(_pqcVerifier);
        admin = msg.sender;
    }
    
    // ========================================================================
    // Sequencer Registration
    // ========================================================================
    
    /// @notice Register a new sequencer with TEE attestation
    function registerSequencer(
        address _sequencer,
        bytes32 _mrenclave,
        bytes calldata _mldsaPubkey,
        bytes calldata _slhdsaPubkey
    ) external {
        require(msg.sender == admin, "SequencerAttestation: unauthorized");
        require(!sequencers[_sequencer].active, "SequencerAttestation: already registered");
        
        sequencers[_sequencer] = SequencerInfo({
            mrenclave: _mrenclave,
            publicKey: PQCVerifier.DualPublicKey({
                mldsa: _mldsaPubkey,
                slhdsa: _slhdsaPubkey
            }),
            active: true,
            registeredAt: block.timestamp
        });
        
        sequencerList.push(_sequencer);
        
        emit SequencerRegistered(_sequencer, _mrenclave);
    }
    
    /// @notice Deactivate a sequencer
    function deactivateSequencer(address _sequencer) external {
        require(msg.sender == admin, "SequencerAttestation: unauthorized");
        require(sequencers[_sequencer].active, "SequencerAttestation: not active");
        
        sequencers[_sequencer].active = false;
        emit SequencerDeactivated(_sequencer);
    }
    
    /// @notice Update sequencer's MRENCLAVE (for upgrades)
    function updateMrenclave(
        address _sequencer,
        bytes32 _newMrenclave
    ) external {
        require(msg.sender == admin, "SequencerAttestation: unauthorized");
        require(sequencers[_sequencer].active, "SequencerAttestation: not active");
        
        sequencers[_sequencer].mrenclave = _newMrenclave;
    }
    
    // ========================================================================
    // Attestation Verification
    // ========================================================================
    
    /// @notice Verify a batch attestation from sequencer
    /// @param _sequencer Address of the sequencer
    /// @param _attestation Full attestation data
    /// @return valid True if attestation is valid
    function verifyAttestation(
        address _sequencer,
        Attestation calldata _attestation
    ) external returns (bool valid) {
        SequencerInfo storage seq = sequencers[_sequencer];
        
        // Check sequencer is active
        if (!seq.active) {
            emit AttestationFailed(_attestation.batchNumber, "sequencer not active");
            return false;
        }
        
        // Check MRENCLAVE matches
        if (_attestation.mrenclave != seq.mrenclave) {
            emit AttestationFailed(_attestation.batchNumber, "mrenclave mismatch");
            return false;
        }
        
        // Check batch number is sequential
        if (_attestation.batchNumber != lastVerifiedBatch + 1) {
            emit AttestationFailed(_attestation.batchNumber, "batch number not sequential");
            return false;
        }
        
        // Verify PQC signature on attestation
        bytes memory attestationData = abi.encodePacked(
            _attestation.mrenclave,
            _attestation.mrsigner,
            _attestation.timestamp,
            _attestation.batchHash,
            _attestation.batchNumber
        );
        
        bool sigValid = pqcVerifier.verifyDualView(
            attestationData,
            _attestation.signature,
            seq.publicKey
        );
        
        if (!sigValid) {
            emit AttestationFailed(_attestation.batchNumber, "signature invalid");
            return false;
        }
        
        // Store verified batch
        verifiedBatches[_attestation.batchNumber] = _attestation.batchHash;
        lastVerifiedBatch = _attestation.batchNumber;
        
        emit BatchVerified(
            _attestation.batchNumber,
            _attestation.batchHash,
            _sequencer
        );
        
        return true;
    }
    
    /// @notice Verify attestation without state changes (view)
    function verifyAttestationView(
        address _sequencer,
        Attestation calldata _attestation
    ) external view returns (bool valid, string memory reason) {
        SequencerInfo storage seq = sequencers[_sequencer];
        
        if (!seq.active) {
            return (false, "sequencer not active");
        }
        
        if (_attestation.mrenclave != seq.mrenclave) {
            return (false, "mrenclave mismatch");
        }
        
        if (_attestation.batchNumber != lastVerifiedBatch + 1) {
            return (false, "batch number not sequential");
        }
        
        bytes memory attestationData = abi.encodePacked(
            _attestation.mrenclave,
            _attestation.mrsigner,
            _attestation.timestamp,
            _attestation.batchHash,
            _attestation.batchNumber
        );
        
        bool sigValid = pqcVerifier.verifyDualView(
            attestationData,
            _attestation.signature,
            seq.publicKey
        );
        
        if (!sigValid) {
            return (false, "signature invalid");
        }
        
        return (true, "");
    }
    
    // ========================================================================
    // View Functions
    // ========================================================================
    
    function isSequencerActive(address _sequencer) external view returns (bool) {
        return sequencers[_sequencer].active;
    }
    
    function getSequencerCount() external view returns (uint256) {
        return sequencerList.length;
    }
    
    function getSequencerAt(uint256 _index) external view returns (address) {
        return sequencerList[_index];
    }
    
    function getBatchHash(uint256 _batchNumber) external view returns (bytes32) {
        return verifiedBatches[_batchNumber];
    }
    
    // ========================================================================
    // Admin
    // ========================================================================
    
    function transferAdmin(address _newAdmin) external {
        require(msg.sender == admin, "SequencerAttestation: unauthorized");
        require(_newAdmin != address(0), "SequencerAttestation: zero address");
        admin = _newAdmin;
    }
}
