// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "@openzeppelin/contracts/utils/Pausable.sol";

contract PetChainRegistry is Pausable {
    // -------------------------------------------------------------------------
    // Constants — string length limits (issue #919)
    // -------------------------------------------------------------------------
    uint256 public constant MAX_SHORT_LEN = 64;   // name, species, breed, birthday
    uint256 public constant MAX_LONG_LEN  = 1000; // diagnosis, treatment, notes

    // -------------------------------------------------------------------------
    // State
    // -------------------------------------------------------------------------
    address public admin;

    struct Vet {
        address vetAddress;
        string  licenseNumber;
        string  specialization;   // issue #921
        bool    isVerified;
        bool    isRevoked;
    }

    struct Pet {
        uint256 petId;
        address owner;
        string  name;
        string  species;
        string  breed;
        string  birthday;
        bool    active;
    }

    struct MedicalRecord {
        uint256 recordId;
        uint256 petId;
        address vet;
        string  diagnosis;
        string  treatment;
        string  notes;
        uint256 timestamp;
    }

    uint256 private _petCounter;
    uint256 private _recordCounter;

    mapping(address => Vet)      public vets;
    mapping(uint256 => Pet)      public pets;
    mapping(uint256 => MedicalRecord[]) private _petRecords;
    mapping(address => uint256[]) private _ownerPets;

    // -------------------------------------------------------------------------
    // Events
    // -------------------------------------------------------------------------
    event VetRegistered(address indexed vet, string licenseNumber);
    event VetSpecializationUpdated(address indexed vet, string specialization);  // issue #921
    event VetVerified(address indexed vet);
    event VetRevoked(address indexed vet);           // issue #916
    event PetRegistered(uint256 indexed petId, address indexed owner);
    event PetTransferred(uint256 indexed petId, address indexed from, address indexed to);
    event PetDeactivated(uint256 indexed petId);     // issue #916
    event PetReactivated(uint256 indexed petId);     // issue #917
    event MedicalRecordAdded(uint256 indexed petId, uint256 indexed recordId, address indexed vet);

    // -------------------------------------------------------------------------
    // Modifiers
    // -------------------------------------------------------------------------
    modifier onlyAdmin() {
        require(msg.sender == admin, "PetChainRegistry: not admin");
        _;
    }

    modifier onlyPetOwner(uint256 petId) {
        require(pets[petId].owner == msg.sender, "PetChainRegistry: not pet owner");
        _;
    }

    modifier onlyVerifiedVet() {
        require(vets[msg.sender].isVerified && !vets[msg.sender].isRevoked,
            "PetChainRegistry: not a verified vet");
        _;
    }

    // -------------------------------------------------------------------------
    // Constructor
    // -------------------------------------------------------------------------
    constructor() {
        admin = msg.sender;
    }

    // -------------------------------------------------------------------------
    // Emergency stop (issue #928)
    // -------------------------------------------------------------------------

    /// @notice Halts all state-mutating operations. Only callable by admin.
    function pause() external onlyAdmin {
        _pause();
    }

    /// @notice Resumes state-mutating operations. Only callable by admin.
    function unpause() external onlyAdmin {
        _unpause();
    }

    // -------------------------------------------------------------------------
    // Vet management
    // -------------------------------------------------------------------------
    function registerVet(string calldata licenseNumber, string calldata specialization) external whenNotPaused {
        require(bytes(licenseNumber).length > 0, "PetChainRegistry: empty licenseNumber");
        vets[msg.sender] = Vet({
            vetAddress:     msg.sender,
            licenseNumber:  licenseNumber,
            specialization: specialization,
            isVerified:     false,
            isRevoked:      false
        });
        emit VetRegistered(msg.sender, licenseNumber);
    }

    /// @notice Update the calling vet's own specialization. issue #921
    function updateSpecialization(string calldata specialization) external whenNotPaused {
        require(vets[msg.sender].vetAddress == msg.sender, "PetChainRegistry: not a registered vet");
        vets[msg.sender].specialization = specialization;
        emit VetSpecializationUpdated(msg.sender, specialization);
    }

    function verifyVet(address vet) external onlyAdmin whenNotPaused {
        require(!vets[vet].isRevoked, "PetChainRegistry: vet is revoked");
        vets[vet].isVerified = true;
        emit VetVerified(vet);
    }

    function revokeVet(address vet) external onlyAdmin whenNotPaused {
        vets[vet].isVerified = false;
        vets[vet].isRevoked  = true;
        emit VetRevoked(vet);   // issue #916
    }

    // -------------------------------------------------------------------------
    // Pet management
    // -------------------------------------------------------------------------

    /// @notice Register a new pet.
    /// issue #919: enforce non-empty and max-length checks on string fields.
    function registerPet(
        string calldata name,
        string calldata species,
        string calldata breed,
        string calldata birthday
    ) external whenNotPaused returns (uint256 petId) {
        require(bytes(name).length > 0 && bytes(name).length <= MAX_SHORT_LEN,
            "PetChainRegistry: invalid name length");
        require(bytes(species).length > 0 && bytes(species).length <= MAX_SHORT_LEN,
            "PetChainRegistry: invalid species length");
        require(bytes(breed).length > 0 && bytes(breed).length <= MAX_SHORT_LEN,
            "PetChainRegistry: invalid breed length");
        require(bytes(birthday).length > 0 && bytes(birthday).length <= MAX_SHORT_LEN,
            "PetChainRegistry: invalid birthday length");

        petId = ++_petCounter;
        pets[petId] = Pet({
            petId:   petId,
            owner:   msg.sender,
            name:    name,
            species: species,
            breed:   breed,
            birthday: birthday,
            active:  true
        });
        _ownerPets[msg.sender].push(petId);
        emit PetRegistered(petId, msg.sender);
    }

    function transferPet(uint256 petId, address to) external onlyPetOwner(petId) whenNotPaused {
        require(to != address(0), "PetChainRegistry: zero address");
        require(pets[petId].active, "PetChainRegistry: pet inactive");
        address from = pets[petId].owner;

        // Remove petId from the previous owner's array (swap-and-pop)
        uint256[] storage fromPets = _ownerPets[from];
        for (uint256 i = 0; i < fromPets.length; i++) {
            if (fromPets[i] == petId) {
                fromPets[i] = fromPets[fromPets.length - 1];
                fromPets.pop();
                break;
            }
        }

        pets[petId].owner = to;
        _ownerPets[to].push(petId);
        emit PetTransferred(petId, from, to);
    }

    function deactivatePet(uint256 petId) external onlyPetOwner(petId) whenNotPaused {
        require(pets[petId].active, "PetChainRegistry: already inactive");
        pets[petId].active = false;
        emit PetDeactivated(petId);   // issue #916
    }

    /// issue #917 — reactivate a previously deactivated pet.
    function reactivatePet(uint256 petId) external onlyPetOwner(petId) whenNotPaused {
        require(!pets[petId].active, "PetChainRegistry: already active");
        pets[petId].active = true;
        emit PetReactivated(petId);
    }

    // -------------------------------------------------------------------------
    // Medical records
    // -------------------------------------------------------------------------

    /// @notice Add a medical record for a pet.
    /// issue #919: enforce non-empty and max-length checks on string fields.
    function addMedicalRecord(
        uint256 petId,
        string calldata diagnosis,
        string calldata treatment,
        string calldata notes
    ) external onlyVerifiedVet whenNotPaused returns (uint256 recordId) {
        require(pets[petId].active, "PetChainRegistry: pet inactive");
        require(bytes(diagnosis).length > 0 && bytes(diagnosis).length <= MAX_LONG_LEN,
            "PetChainRegistry: invalid diagnosis length");
        require(bytes(treatment).length > 0 && bytes(treatment).length <= MAX_LONG_LEN,
            "PetChainRegistry: invalid treatment length");
        require(bytes(notes).length <= MAX_LONG_LEN,
            "PetChainRegistry: notes too long");

        recordId = ++_recordCounter;
        _petRecords[petId].push(MedicalRecord({
            recordId:  recordId,
            petId:     petId,
            vet:       msg.sender,
            diagnosis: diagnosis,
            treatment: treatment,
            notes:     notes,
            timestamp: block.timestamp
        }));
        emit MedicalRecordAdded(petId, recordId, msg.sender);
    }

    // -------------------------------------------------------------------------
    // View functions — full arrays
    // -------------------------------------------------------------------------
    function getPetsByOwner(address owner) external view returns (uint256[] memory) {
        return _ownerPets[owner];
    }

    function getPetRecords(uint256 petId) external view returns (MedicalRecord[] memory) {
        return _petRecords[petId];
    }

    // -------------------------------------------------------------------------
    // Paginated view functions (issue #918)
    // -------------------------------------------------------------------------

    /// @notice Return a page of petIds owned by `owner`.
    /// @param owner  The owner address to query.
    /// @param offset Starting index (0-based).
    /// @param limit  Maximum number of items to return.
    function getPetsByOwnerPaged(
        address owner,
        uint256 offset,
        uint256 limit
    ) external view returns (uint256[] memory page) {
        uint256[] storage all = _ownerPets[owner];
        uint256 total = all.length;
        if (offset >= total || limit == 0) {
            return new uint256[](0);
        }
        uint256 end = offset + limit;
        if (end > total) end = total;
        uint256 size = end - offset;
        page = new uint256[](size);
        for (uint256 i = 0; i < size; i++) {
            page[i] = all[offset + i];
        }
    }

    /// @notice Return a page of medical records for `petId`.
    /// @param petId  The pet to query.
    /// @param offset Starting index (0-based).
    /// @param limit  Maximum number of items to return.
    function getPetRecordsPaged(
        uint256 petId,
        uint256 offset,
        uint256 limit
    ) external view returns (MedicalRecord[] memory page) {
        MedicalRecord[] storage all = _petRecords[petId];
        uint256 total = all.length;
        if (offset >= total || limit == 0) {
            return new MedicalRecord[](0);
        }
        uint256 end = offset + limit;
        if (end > total) end = total;
        uint256 size = end - offset;
        page = new MedicalRecord[](size);
        for (uint256 i = 0; i < size; i++) {
            page[i] = all[offset + i];
        }
    }
}
