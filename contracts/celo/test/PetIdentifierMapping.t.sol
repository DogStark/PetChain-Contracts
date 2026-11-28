// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "forge-std/Test.sol";
import "../src/PetIdentifierMapping.sol";

contract PetIdentifierMappingTest is Test {
    PetIdentifierMapping internal mapping;

    address internal admin = address(0xA11CE);
    address internal owner = address(0xB0B);
    address internal stranger = address(0xCAFE);

    bytes32 internal constant CELO_ID_1 = keccak256("celo-pet-1");
    bytes32 internal constant CELO_ID_2 = keccak256("celo-pet-2");
    bytes32 internal constant CANONICAL_1 = keccak256("canonical-pet-1");
    bytes32 internal constant CANONICAL_2 = keccak256("canonical-pet-2");

    event MappingSet(bytes32 indexed celoId, bytes32 indexed canonicalId, address indexed operator);
    event MappingRevoked(bytes32 indexed celoId, bytes32 indexed canonicalId, address indexed operator);

    function setUp() public {
        mapping = new PetIdentifierMapping(admin);
        vm.prank(admin);
        mapping.setOwner(owner);
    }

    function testOwnerCanSetMapping() public {
        vm.expectEmit(true, true, true, true);
        emit MappingSet(CELO_ID_1, CANONICAL_1, owner);

        vm.prank(owner);
        mapping.setMapping(CELO_ID_1, CANONICAL_1);

        assertEq(mapping.canonicalOf(CELO_ID_1), CANONICAL_1);
        assertEq(mapping.celoOf(CANONICAL_1), CELO_ID_1);
        assertTrue(mapping.isMapped(CELO_ID_1));
    }

    function testAdminCanSetMapping() public {
        vm.prank(admin);
        mapping.setMapping(CELO_ID_1, CANONICAL_1);
        assertEq(mapping.canonicalOf(CELO_ID_1), CANONICAL_1);
    }

    function testUnauthorizedCannotSetMapping() public {
        vm.prank(stranger);
        vm.expectRevert(PetIdentifierMapping.Unauthorized.selector);
        mapping.setMapping(CELO_ID_1, CANONICAL_1);

        assertFalse(mapping.isMapped(CELO_ID_1));
        assertEq(mapping.canonicalOf(CELO_ID_1), bytes32(0));
    }

    function testCannotDoubleMapCeloId() public {
        vm.prank(owner);
        mapping.setMapping(CELO_ID_1, CANONICAL_1);

        vm.prank(owner);
        vm.expectRevert(PetIdentifierMapping.AlreadyMapped.selector);
        mapping.setMapping(CELO_ID_1, CANONICAL_2);

        assertEq(mapping.canonicalOf(CELO_ID_1), CANONICAL_1);
        assertEq(mapping.celoOf(CANONICAL_2), bytes32(0));
    }

    function testCannotDoubleMapCanonicalId() public {
        vm.prank(owner);
        mapping.setMapping(CELO_ID_1, CANONICAL_1);

        vm.prank(owner);
        vm.expectRevert(PetIdentifierMapping.CanonicalAlreadyMapped.selector);
        mapping.setMapping(CELO_ID_2, CANONICAL_1);

        assertEq(mapping.celoOf(CANONICAL_1), CELO_ID_1);
        assertFalse(mapping.isMapped(CELO_ID_2));
    }

    function testRejectsZeroIdentifiers() public {
        vm.prank(owner);
        vm.expectRevert(PetIdentifierMapping.InvalidIdentifier.selector);
        mapping.setMapping(bytes32(0), CANONICAL_1);

        vm.prank(owner);
        vm.expectRevert(PetIdentifierMapping.InvalidIdentifier.selector);
        mapping.setMapping(CELO_ID_1, bytes32(0));
    }

    function testRevokeMapping() public {
        vm.prank(owner);
        mapping.setMapping(CELO_ID_1, CANONICAL_1);

        vm.expectEmit(true, true, true, true);
        emit MappingRevoked(CELO_ID_1, CANONICAL_1, owner);

        vm.prank(owner);
        mapping.revokeMapping(CELO_ID_1);

        assertFalse(mapping.isMapped(CELO_ID_1));
        assertEq(mapping.canonicalOf(CELO_ID_1), bytes32(0));
        assertEq(mapping.celoOf(CANONICAL_1), bytes32(0));
    }

    function testRevokeMissingMappingReverts() public {
        vm.prank(owner);
        vm.expectRevert(PetIdentifierMapping.NotMapped.selector);
        mapping.revokeMapping(CELO_ID_1);
    }

    function testUnauthorizedCannotRevoke() public {
        vm.prank(owner);
        mapping.setMapping(CELO_ID_1, CANONICAL_1);

        vm.prank(stranger);
        vm.expectRevert(PetIdentifierMapping.Unauthorized.selector);
        mapping.revokeMapping(CELO_ID_1);

        assertEq(mapping.canonicalOf(CELO_ID_1), CANONICAL_1);
    }

    function testRemapAfterRevoke() public {
        vm.prank(owner);
        mapping.setMapping(CELO_ID_1, CANONICAL_1);

        vm.prank(owner);
        mapping.revokeMapping(CELO_ID_1);

        vm.prank(owner);
        mapping.setMapping(CELO_ID_1, CANONICAL_2);

        assertEq(mapping.canonicalOf(CELO_ID_1), CANONICAL_2);
        assertEq(mapping.celoOf(CANONICAL_2), CELO_ID_1);
    }

    function testStorageVersion() public view {
        assertEq(mapping.storageVersion(), 1);
    }
}
