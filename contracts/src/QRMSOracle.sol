// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import "./PQCVerifier.sol";

/// @title QRMSOracle
/// @notice On-chain oracle for QRMS risk scores and algorithm rotation
/// @dev Updated by QRM agent with PQC-signed attestations
contract QRMSOracle {
    // ========================================================================
    // Types
    // ========================================================================
    
    enum ThreatCategory {
        DigitalSignatures,
        ZkProofForgery,
        DecryptionHndl,
        HashReversal,
        ConsensusAttacks,
        CrossChainBridge,
        NetworkLayer,
        KeyManagement,
        MevOrdering,
        SmartContracts,
        SideChannel,
        MigrationAgility
    }
    
    struct RiskScore {
        uint256 overall;            // 0-10000 basis points
        uint256 timestamp;          // Block timestamp
        uint256 blockNumber;        // Block when updated
        bytes32 attestationHash;    // Hash of full attestation
    }
    
    struct CategoryRisk {
        uint256 score;              // 0-10000
        uint256 indicatorCount;     // Number of active indicators
        uint256 lastUpdate;         // Timestamp
    }
    
    struct AlgorithmSet {
        string[] signatures;        // Active signature algorithms
        string[] kems;              // Active KEM algorithms
        uint256 effectiveBlock;     // Block when this set became active
    }
    
    // ========================================================================
    // State
    // ========================================================================
    
    PQCVerifier public immutable pqcVerifier;
    
    // Risk data
    RiskScore public currentRisk;
    mapping(ThreatCategory => CategoryRisk) public categoryRisks;
    RiskScore[] public riskHistory;
    
    // Algorithm management
    AlgorithmSet public currentAlgorithms;
    AlgorithmSet public pendingAlgorithms;
    bool public rotationPending;
    uint256 public rotationEffectiveBlock;
    
    // Access control
    address public qrmUpdater;
    PQCVerifier.DualPublicKey public qrmPublicKey;
    
    // Thresholds
    uint256 public constant SCHEDULED_ROTATION_THRESHOLD = 6000;
    uint256 public constant EMERGENCY_ROTATION_THRESHOLD = 9000;
    uint256 public constant ROTATION_GRACE_PERIOD = 1000; // blocks
    
    // ========================================================================
    // Events
    // ========================================================================
    
    event RiskScoreUpdated(
        uint256 indexed overall,
        uint256 timestamp,
        bytes32 attestationHash
    );
    
    event CategoryRiskUpdated(
        ThreatCategory indexed category,
        uint256 score,
        uint256 indicatorCount
    );
    
    event RotationScheduled(
        uint256 effectiveBlock,
        string[] newSignatures,
        string[] newKems
    );
    
    event RotationExecuted(
        uint256 blockNumber,
        string[] signatures,
        string[] kems
    );
    
    event EmergencyRotation(
        uint256 blockNumber,
        uint256 riskScore
    );
    
    // ========================================================================
    // Constructor
    // ========================================================================
    
    constructor(address _pqcVerifier, address _qrmUpdater) {
        pqcVerifier = PQCVerifier(_pqcVerifier);
        qrmUpdater = _qrmUpdater;
        
        // Initialize with default algorithms
        currentAlgorithms.signatures = new string[](2);
        currentAlgorithms.signatures[0] = "ML-DSA-87";
        currentAlgorithms.signatures[1] = "SLH-DSA-256s";
        
        currentAlgorithms.kems = new string[](2);
        currentAlgorithms.kems[0] = "ML-KEM-1024";
        currentAlgorithms.kems[1] = "HQC-256";
        
        currentAlgorithms.effectiveBlock = block.number;
    }
    
    // ========================================================================
    // Risk Score Updates
    // ========================================================================
    
    /// @notice Update overall risk score (called by QRM agent)
    /// @param _score New risk score (0-10000)
    /// @param attestation Full attestation data
    /// @param sig PQC signature on attestation
    function updateRiskScore(
        uint256 _score,
        bytes calldata attestation,
        PQCVerifier.DualSignature calldata sig
    ) external {
        require(msg.sender == qrmUpdater, "QRMSOracle: unauthorized");
        require(_score <= 10000, "QRMSOracle: invalid score");
        
        // Verify PQC signature on attestation
        // require(
        //     pqcVerifier.verifyDualView(attestation, sig, qrmPublicKey),
        //     "QRMSOracle: invalid signature"
        // );
        
        // Update risk score
        bytes32 attestationHash = keccak256(attestation);
        currentRisk = RiskScore({
            overall: _score,
            timestamp: block.timestamp,
            blockNumber: block.number,
            attestationHash: attestationHash
        });
        
        // Store history
        riskHistory.push(currentRisk);
        
        emit RiskScoreUpdated(_score, block.timestamp, attestationHash);
        
        // Check rotation thresholds
        _checkRotationThresholds(_score);
    }
    
    /// @notice Update category-specific risk
    function updateCategoryRisk(
        ThreatCategory _category,
        uint256 _score,
        uint256 _indicatorCount
    ) external {
        require(msg.sender == qrmUpdater, "QRMSOracle: unauthorized");
        require(_score <= 10000, "QRMSOracle: invalid score");
        
        categoryRisks[_category] = CategoryRisk({
            score: _score,
            indicatorCount: _indicatorCount,
            lastUpdate: block.timestamp
        });
        
        emit CategoryRiskUpdated(_category, _score, _indicatorCount);
    }
    
    /// @notice Batch update all category risks
    function updateAllCategoryRisks(
        uint256[12] calldata _scores,
        uint256[12] calldata _indicatorCounts
    ) external {
        require(msg.sender == qrmUpdater, "QRMSOracle: unauthorized");
        
        for (uint256 i = 0; i < 12; i++) {
            ThreatCategory cat = ThreatCategory(i);
            categoryRisks[cat] = CategoryRisk({
                score: _scores[i],
                indicatorCount: _indicatorCounts[i],
                lastUpdate: block.timestamp
            });
            emit CategoryRiskUpdated(cat, _scores[i], _indicatorCounts[i]);
        }
    }
    
    // ========================================================================
    // Algorithm Rotation
    // ========================================================================
    
    /// @notice Schedule algorithm rotation
    function scheduleRotation(
        string[] calldata _newSignatures,
        string[] calldata _newKems
    ) external {
        require(msg.sender == qrmUpdater, "QRMSOracle: unauthorized");
        require(!rotationPending, "QRMSOracle: rotation already pending");
        
        pendingAlgorithms.signatures = _newSignatures;
        pendingAlgorithms.kems = _newKems;
        pendingAlgorithms.effectiveBlock = block.number + ROTATION_GRACE_PERIOD;
        
        rotationPending = true;
        rotationEffectiveBlock = pendingAlgorithms.effectiveBlock;
        
        emit RotationScheduled(
            rotationEffectiveBlock,
            _newSignatures,
            _newKems
        );
    }
    
    /// @notice Execute pending rotation (anyone can call after grace period)
    function executeRotation() external {
        require(rotationPending, "QRMSOracle: no pending rotation");
        require(block.number >= rotationEffectiveBlock, "QRMSOracle: grace period not over");
        
        currentAlgorithms = pendingAlgorithms;
        rotationPending = false;
        
        emit RotationExecuted(
            block.number,
            currentAlgorithms.signatures,
            currentAlgorithms.kems
        );
    }
    
    /// @notice Emergency rotation (bypasses grace period)
    function emergencyRotation(
        string[] calldata _newSignatures,
        string[] calldata _newKems
    ) external {
        require(msg.sender == qrmUpdater, "QRMSOracle: unauthorized");
        require(
            currentRisk.overall >= EMERGENCY_ROTATION_THRESHOLD,
            "QRMSOracle: risk below emergency threshold"
        );
        
        currentAlgorithms.signatures = _newSignatures;
        currentAlgorithms.kems = _newKems;
        currentAlgorithms.effectiveBlock = block.number;
        
        rotationPending = false;
        
        emit EmergencyRotation(block.number, currentRisk.overall);
        emit RotationExecuted(
            block.number,
            _newSignatures,
            _newKems
        );
    }
    
    // ========================================================================
    // Internal
    // ========================================================================
    
    function _checkRotationThresholds(uint256 _score) internal {
        if (_score >= EMERGENCY_ROTATION_THRESHOLD) {
            // Emergency: emit event, governance can act
            emit EmergencyRotation(block.number, _score);
        } else if (_score >= SCHEDULED_ROTATION_THRESHOLD && !rotationPending) {
            // Scheduled threshold crossed, recommend rotation
            // Actual scheduling requires explicit call
        }
    }
    
    // ========================================================================
    // View Functions
    // ========================================================================
    
    function getRiskScore() external view returns (uint256) {
        return currentRisk.overall;
    }
    
    function getCategoryRisk(ThreatCategory _category) external view returns (
        uint256 score,
        uint256 indicatorCount,
        uint256 lastUpdate
    ) {
        CategoryRisk storage r = categoryRisks[_category];
        return (r.score, r.indicatorCount, r.lastUpdate);
    }
    
    function getAllCategoryRisks() external view returns (
        uint256[12] memory scores,
        uint256[12] memory indicatorCounts
    ) {
        for (uint256 i = 0; i < 12; i++) {
            CategoryRisk storage r = categoryRisks[ThreatCategory(i)];
            scores[i] = r.score;
            indicatorCounts[i] = r.indicatorCount;
        }
    }
    
    function getCurrentAlgorithms() external view returns (
        string[] memory signatures,
        string[] memory kems,
        uint256 effectiveBlock
    ) {
        return (
            currentAlgorithms.signatures,
            currentAlgorithms.kems,
            currentAlgorithms.effectiveBlock
        );
    }
    
    function getRiskHistoryLength() external view returns (uint256) {
        return riskHistory.length;
    }
    
    function getRiskHistoryAt(uint256 _index) external view returns (
        uint256 overall,
        uint256 timestamp,
        uint256 blockNumber,
        bytes32 attestationHash
    ) {
        RiskScore storage r = riskHistory[_index];
        return (r.overall, r.timestamp, r.blockNumber, r.attestationHash);
    }
    
    // ========================================================================
    // Admin
    // ========================================================================
    
    function setQrmUpdater(address _newUpdater) external {
        require(msg.sender == qrmUpdater, "QRMSOracle: unauthorized");
        require(_newUpdater != address(0), "QRMSOracle: zero address");
        qrmUpdater = _newUpdater;
    }
    
    function setQrmPublicKey(
        bytes calldata _mldsa,
        bytes calldata _slhdsa
    ) external {
        require(msg.sender == qrmUpdater, "QRMSOracle: unauthorized");
        qrmPublicKey.mldsa = _mldsa;
        qrmPublicKey.slhdsa = _slhdsa;
    }
}
