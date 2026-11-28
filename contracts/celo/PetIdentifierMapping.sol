// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

/**
 * @title PetIdentifierMapping
 * @notice Explicit mapping from Celo pet identifiers to canonical PetChain identifiers.
 * @dev Enforces uniqueness (one Celo id -> one canonical id, and one canonical id -> one
 *      Celo id) and supports authorized revocation. Follows the Celo contract conventions:
 *      owner/admin authorization, a storage version constant, bounded input sizes, and
 *      documented error codes and events.
 *
 * Error codes:
 *  - NotOwner: caller is not the contract owner.
 *  - NotAdmin: caller is not an authorized admin.
 *  - ZeroAddress: a required address argument was the zero address.
 *  - EmptyIdentifier: a pet identifier argument was empty.
 *  - IdentifierTooLong: a pet identifier exceeded MAX_IDENTIFIER_LENGTH.
 *  - AlreadyMapped: the Celo pet id already has a canonical mapping.
 *  - CanonicalAlreadyMapped: the canonical id is already bound to a different Celo id.
 *  - MappingNotFound: revocation requested for a non-existent mapping.
 *
 * Events:
 *  - MappingCreated: emitted when a new mapping is created.
 *  - MappingRevoked: emitted when an existing mapping is revoked.
 *  - AdminUpdated: emitted when an admin is granted/revoked.
 *  - OwnershipTransferred: emitted when ownership is transferred.
 *
 * Storage version: STORAGE_VERSION = 1.
 */
contract PetIdentifierMapping {
    /// @notice Storage layout version for compatibility fixtures.
    uint256 public constant STORAGE_VERSION = 1;

    /// @notice Maximum allowed length for a pet identifier (bytes).
    uint256 public constant MAX_IDENTIFIER_LENGTH = 128;

    address public owner;

    /// @notice Celo pet id => canonical PetChain id.
    mapping(bytes32 => bytes32) private _celoToCanonical;

    /// @notice Canonical PetChain id => Celo pet id (reverse uniqueness guard).
    mapping(bytes32 => bytes32) private _canonicalToCelo;

    /// @notice Whether a Celo pet id currently has a mapping.
    mapping(bytes32 => bool) private _hasMapping;

    /// @notice Authorized admins allowed to write mappings.
    mapping(address => bool) public isAdmin;

    error NotOwner();
    error NotAdmin();
    error ZeroAddress();
    error EmptyIdentifier();
    error IdentifierTooLong();
    error AlreadyMapped();
    error CanonicalAlreadyMapped();
    error MappingNotFound();

    event MappingCreated(bytes32 indexed celoPetId, bytes32 indexed canonicalPetId, address indexed operator);
    event MappingRevoked(bytes32 indexed celoPetId, bytes32 indexed canonicalPetId, address indexed operator);
    event AdminUpdated(address indexed admin, bool enabled);
    event OwnershipTransferred(address indexed previousOwner, address indexed newOwner);

    modifier onlyOwner() {
        if (msg.sender != owner) revert NotOwner();
        _;
    }

    modifier onlyAuthorized() {
        if (msg.sender != owner && !isAdmin[msg.sender]) revert NotAdmin();
        _;
    }

    constructor(address initialOwner) {
        if (initialOwner == address(0)) revert ZeroAddress();
        owner = initialOwner;
        emit OwnershipTransferred(address(0), initialOwner);
    }

    /// @notice Grant or revoke admin privileges. Owner only.
    function setAdmin(address admin, bool enabled) external onlyOwner {
        if (admin == address(0)) revert ZeroAddress();
        isAdmin[admin] = enabled;
        emit AdminUpdated(admin, enabled);
    }

    /// @notice Transfer contract ownership. Owner only.
    function transferOwnership(address newOwner) external onlyOwner {
        if (newOwner == address(0)) revert ZeroAddress();
        address previous = owner;
        owner = newOwner;
        emit OwnershipTransferred(previous, newOwner);
    }

    /// @notice Create a mapping from a Celo pet id to a canonical PetChain id.
    /// @dev Reverts if either side is already mapped, leaving state unchanged.
    function setMapping(bytes32 celoPetId, bytes32 canonicalPetId) external onlyAuthorized {
        if (celoPetId == bytes32(0) || canonicalPetId == bytes32(0)) revert EmptyIdentifier();
        if (_hasMapping[celoPetId]) revert AlreadyMapped();
        if (_canonicalToCelo[canonicalPetId] != bytes32(0)) revert CanonicalAlreadyMapped();

        _celoToCanonical[celoPetId] = canonicalPetId;
        _canonicalToCelo[canonicalPetId] = celoPetId;
        _hasMapping[celoPetId] = true;

        emit MappingCreated(celoPetId, canonicalPetId, msg.sender);
    }

    /// @notice Revoke an existing mapping. Authorized callers only.
    /// @dev Reverts with MappingNotFound if no mapping exists (explicit, non-idempotent).
    function revokeMapping(bytes32 celoPetId) external onlyAuthorized {
        if (!_hasMapping[celoPetId]) revert MappingNotFound();

        bytes32 canonicalPetId = _celoToCanonical[celoPetId];
        delete _celoToCanonical[celoPetId];
        delete _canonicalToCelo[canonicalPetId];
        delete _hasMapping[celoPetId];

        emit MappingRevoked(celoPetId, canonicalPetId, msg.sender);
    }

    /// @notice Returns the canonical PetChain id for a Celo pet id (zero if unmapped).
    function canonicalOf(bytes32 celoPetId) external view returns (bytes32) {
        return _celoToCanonical[celoPetId];
    }

    /// @notice Returns the Celo pet id bound to a canonical id (zero if unmapped).
    function celoOf(bytes32 canonicalPetId) external view returns (bytes32) {
        return _canonicalToCelo[canonicalPetId];
    }

    /// @notice Whether a Celo pet id currently has a mapping.
    function hasMapping(bytes32 celoPetId) external view returns (bool) {
        return _hasMapping[celoPetId];
    }

    /// @notice Helper to derive a bounded bytes32 identifier from a string.
    /// @dev Reverts if the string is empty or exceeds MAX_IDENTIFIER_LENGTH.
    function toIdentifier(string calldata value) external pure returns (bytes32) {
        bytes calldata raw = bytes(value);
        if (raw.length == 0) revert EmptyIdentifier();
        if (raw.length > MAX_IDENTIFIER_LENGTH) revert IdentifierTooLong();
        return keccak256(raw);
    }
}
