// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

/// @title PQCVerifier
/// @notice On-chain verification of post-quantum cryptographic signatures
/// @dev Uses precompiles at 0x20 (ML-DSA) and 0x21 (SLH-DSA)
///      For testnet: simulates verification until precompiles are deployed
contract PQCVerifier {
    // ========================================================================
    // Precompile Addresses (custom for QRMS chain)
    // ========================================================================
    
    address constant MLDSA_VERIFY = address(0x20);
    address constant SLHDSA_VERIFY = address(0x21);
    
    // ========================================================================
    // Algorithm Sizes
    // ========================================================================
    
    // ML-DSA-87 (Dilithium)
    uint256 constant MLDSA_PUBKEY_SIZE = 2592;
    uint256 constant MLDSA_SIG_SIZE = 4595;
    
    // SLH-DSA-256s (SPHINCS+)
    uint256 constant SLHDSA_PUBKEY_SIZE = 64;
    uint256 constant SLHDSA_SIG_SIZE = 29792;
    
    // ========================================================================
    // Types
    // ========================================================================
    
    struct DualPublicKey {
        bytes mldsa;    // 2592 bytes
        bytes slhdsa;   // 64 bytes
    }
    
    struct DualSignature {
        bytes mldsa;    // 4595 bytes
        bytes slhdsa;   // 29792 bytes
    }
    
    // ========================================================================
    // State
    // ========================================================================
    
    bool public precompilesEnabled;
    address public admin;
    
    // ========================================================================
    // Events
    // ========================================================================
    
    event SignatureVerified(bytes32 indexed messageHash, bool mldsaValid, bool slhdsaValid);
    event PrecompilesToggled(bool enabled);
    
    // ========================================================================
    // Constructor
    // ========================================================================
    
    constructor() {
        admin = msg.sender;
        precompilesEnabled = false; // Simulate by default on testnet
    }
    
    // ========================================================================
    // Admin
    // ========================================================================
    
    modifier onlyAdmin() {
        require(msg.sender == admin, "PQCVerifier: unauthorized");
        _;
    }
    
    function setPrecompilesEnabled(bool _enabled) external onlyAdmin {
        precompilesEnabled = _enabled;
        emit PrecompilesToggled(_enabled);
    }
    
    function transferAdmin(address _newAdmin) external onlyAdmin {
        require(_newAdmin != address(0), "PQCVerifier: zero address");
        admin = _newAdmin;
    }
    
    // ========================================================================
    // Verification (AND Combiner - Security)
    // ========================================================================
    
    /// @notice Verify dual signature with AND combiner (both must be valid)
    /// @param message The message that was signed
    /// @param sig Dual signature (ML-DSA + SLH-DSA)
    /// @param pubkey Dual public key
    /// @return valid True if BOTH signatures are valid
    function verifyDual(
        bytes calldata message,
        DualSignature calldata sig,
        DualPublicKey calldata pubkey
    ) external returns (bool valid) {
        bool mldsaValid = _verifyMldsa(message, sig.mldsa, pubkey.mldsa);
        bool slhdsaValid = _verifySlhdsa(message, sig.slhdsa, pubkey.slhdsa);
        
        emit SignatureVerified(keccak256(message), mldsaValid, slhdsaValid);
        
        return mldsaValid && slhdsaValid;
    }
    
    /// @notice View version for gas estimation (no event)
    function verifyDualView(
        bytes calldata message,
        DualSignature calldata sig,
        DualPublicKey calldata pubkey
    ) external view returns (bool valid) {
        bool mldsaValid = _verifyMldsaView(message, sig.mldsa, pubkey.mldsa);
        bool slhdsaValid = _verifySlhdsaView(message, sig.slhdsa, pubkey.slhdsa);
        return mldsaValid && slhdsaValid;
    }
    
    // ========================================================================
    // Verification (OR Combiner - Availability)
    // ========================================================================
    
    /// @notice Verify dual signature with OR combiner (either can be valid)
    /// @param message The message that was signed
    /// @param sig Dual signature (ML-DSA + SLH-DSA)
    /// @param pubkey Dual public key
    /// @return valid True if EITHER signature is valid
    function verifyDualOr(
        bytes calldata message,
        DualSignature calldata sig,
        DualPublicKey calldata pubkey
    ) external returns (bool valid) {
        bool mldsaValid = _verifyMldsa(message, sig.mldsa, pubkey.mldsa);
        if (mldsaValid) {
            emit SignatureVerified(keccak256(message), true, false);
            return true;
        }
        
        bool slhdsaValid = _verifySlhdsa(message, sig.slhdsa, pubkey.slhdsa);
        emit SignatureVerified(keccak256(message), false, slhdsaValid);
        return slhdsaValid;
    }
    
    // ========================================================================
    // Individual Verification
    // ========================================================================
    
    /// @notice Verify ML-DSA-87 signature only
    function verifyMldsa(
        bytes calldata message,
        bytes calldata sig,
        bytes calldata pubkey
    ) external view returns (bool) {
        return _verifyMldsaView(message, sig, pubkey);
    }
    
    /// @notice Verify SLH-DSA-256s signature only
    function verifySlhdsa(
        bytes calldata message,
        bytes calldata sig,
        bytes calldata pubkey
    ) external view returns (bool) {
        return _verifySlhdsaView(message, sig, pubkey);
    }
    
    // ========================================================================
    // Internal Verification
    // ========================================================================
    
    function _verifyMldsa(
        bytes calldata message,
        bytes calldata sig,
        bytes calldata pubkey
    ) internal returns (bool) {
        if (!precompilesEnabled) {
            return _simulateMldsaVerify(message, sig, pubkey);
        }
        
        bytes memory input = abi.encodePacked(pubkey, sig, message);
        (bool success, bytes memory result) = MLDSA_VERIFY.call(input);
        return success && result.length > 0 && result[0] == 0x01;
    }
    
    function _verifyMldsaView(
        bytes calldata message,
        bytes calldata sig,
        bytes calldata pubkey
    ) internal view returns (bool) {
        if (!precompilesEnabled) {
            return _simulateMldsaVerify(message, sig, pubkey);
        }
        
        bytes memory input = abi.encodePacked(pubkey, sig, message);
        (bool success, bytes memory result) = MLDSA_VERIFY.staticcall(input);
        return success && result.length > 0 && result[0] == 0x01;
    }
    
    function _verifySlhdsa(
        bytes calldata message,
        bytes calldata sig,
        bytes calldata pubkey
    ) internal returns (bool) {
        if (!precompilesEnabled) {
            return _simulateSlhdsaVerify(message, sig, pubkey);
        }
        
        bytes memory input = abi.encodePacked(pubkey, sig, message);
        (bool success, bytes memory result) = SLHDSA_VERIFY.call(input);
        return success && result.length > 0 && result[0] == 0x01;
    }
    
    function _verifySlhdsaView(
        bytes calldata message,
        bytes calldata sig,
        bytes calldata pubkey
    ) internal view returns (bool) {
        if (!precompilesEnabled) {
            return _simulateSlhdsaVerify(message, sig, pubkey);
        }
        
        bytes memory input = abi.encodePacked(pubkey, sig, message);
        (bool success, bytes memory result) = SLHDSA_VERIFY.staticcall(input);
        return success && result.length > 0 && result[0] == 0x01;
    }
    
    // ========================================================================
    // Simulation (Testnet)
    // ========================================================================
    
    /// @dev Simulated ML-DSA verification for testnet
    ///      Validates structure and returns true if format is correct
    function _simulateMldsaVerify(
        bytes calldata message,
        bytes calldata sig,
        bytes calldata pubkey
    ) internal pure returns (bool) {
        // Validate sizes
        if (pubkey.length != MLDSA_PUBKEY_SIZE) return false;
        if (sig.length != MLDSA_SIG_SIZE) return false;
        if (message.length == 0) return false;
        
        // Simulated verification: check signature format
        // In production, precompile does real crypto
        bytes32 sigHash = keccak256(abi.encodePacked(sig, pubkey, message));
        
        // Accept if hash has specific pattern (simulation)
        return uint256(sigHash) % 100 < 95; // 95% success rate for valid format
    }
    
    /// @dev Simulated SLH-DSA verification for testnet
    function _simulateSlhdsaVerify(
        bytes calldata message,
        bytes calldata sig,
        bytes calldata pubkey
    ) internal pure returns (bool) {
        // Validate sizes
        if (pubkey.length != SLHDSA_PUBKEY_SIZE) return false;
        if (sig.length != SLHDSA_SIG_SIZE) return false;
        if (message.length == 0) return false;
        
        bytes32 sigHash = keccak256(abi.encodePacked(sig, pubkey, message));
        return uint256(sigHash) % 100 < 95;
    }
    
    // ========================================================================
    // Gas Estimation Helpers
    // ========================================================================
    
    function estimatedGasMldsa() external pure returns (uint256) {
        return 15000; // Precompile cost
    }
    
    function estimatedGasSlhdsa() external pure returns (uint256) {
        return 50000; // Precompile cost (hash-based, slower)
    }
    
    function estimatedGasDual() external pure returns (uint256) {
        return 65000; // Both precompiles
    }
}
