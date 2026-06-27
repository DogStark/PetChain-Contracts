// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

contract PetChainRegistry {
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

    // recordId → petId, so correctMedicalRecord can locate the record
    mapping(uint256 => uint256) private _recordPetId;
    // recordId → index inside _petRecords[petId]
    mapping(uint256 => uint256) private _recordIndex;

    // -------------------------------------------------------------------------
    // Events
    // -------------------------------------------------------------------------
    event VetRegistered(address indexed vet, string licenseNumber);
    event VetVerified(address indexed vet);
    event VetRevoked(address indexed vet);           // issue #916
    event PetRegistered(uint256 indexed petId, address indexed owner);
    event PetTransferred(uint256 indexed petId, address indexed from, address indexed to);
    event PetDeactivated(uint256 indexed petId);     // issue #916
    event PetReactivated(uint256 indexed petId);     // issue #917
    event MedicalRecordAdded(uint256 indexed petId, uint256 indexed recordId, address indexed vet);
    /// @dev Emitted when a medical record is corrected.
    ///      The original field values are preserved in the event log for full auditability.
    event MedicalRecordCorrected(
        uint256 indexed recordId,
        uint256 indexed petId,
        address indexed correctedBy,
        string  originalDiagnosis,
        string  originalTreatment,
        string  originalNotes,
        string  newDiagnosis,
        string  newTreatment,
        string  newNotes
    );
    event AdminTransferred(address indexed previousAdmin, address indexed newAdmin);

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
    // Admin management
    // -------------------------------------------------------------------------

    /// @notice Transfer admin role to a new address.
    /// @param newAdmin The address that will become the new admin.
    function transferAdmin(address newAdmin) external onlyAdmin {
        require(newAdmin != address(0), "PetChainRegistry: zero address");
        address previous = admin;
        admin = newAdmin;
        emit AdminTransferred(previous, newAdmin);
    }

    // -------------------------------------------------------------------------
    // Vet management
    // -------------------------------------------------------------------------
    function registerVet(string calldata licenseNumber) external {
        require(bytes(licenseNumber).length > 0, "PetChainRegistry: empty licenseNumber");
        vets[msg.sender] = Vet({
            vetAddress:    msg.sender,
            licenseNumber: licenseNumber,
            isVerified:    false,
            isRevoked:     false
        });
        emit VetRegistered(msg.sender, licenseNumber);
    }

    function verifyVet(address vet) external onlyAdmin {
        require(!vets[vet].isRevoked, "PetChainRegistry: vet is revoked");
        vets[vet].isVerified = true;
        emit VetVerified(vet);
    }

    function revokeVet(address vet) external onlyAdmin {
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
    ) external returns (uint256 petId) {
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

    function transferPet(uint256 petId, address to) external onlyPetOwner(petId) {
        require(to != address(0), "PetChainRegistry: zero address");
        require(pets[petId].active, "PetChainRegistry: pet inactive");
        address from = pets[petId].owner;
        pets[petId].owner = to;
        _ownerPets[to].push(petId);
        emit PetTransferred(petId, from, to);
    }

    function deactivatePet(uint256 petId) external onlyPetOwner(petId) {
        require(pets[petId].active, "PetChainRegistry: already inactive");
        pets[petId].active = false;
        emit PetDeactivated(petId);   // issue #916
    }

    /// issue #917 — reactivate a previously deactivated pet.
    function reactivatePet(uint256 petId) external onlyPetOwner(petId) {
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
    ) external onlyVerifiedVet returns (uint256 recordId) {
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
        // Store reverse-lookup so correctMedicalRecord can find the record in O(1)
        _recordPetId[recordId] = petId;
        _recordIndex[recordId] = _petRecords[petId].length - 1;
        emit MedicalRecordAdded(petId, recordId, msg.sender);
    }

    /// @notice Correct an existing medical record.
    /// @dev    Only the vet who originally created the record, or the admin, may call this.
    ///         The original field values are emitted in MedicalRecordCorrected for auditability.
    /// @param recordId  The ID of the record to correct.
    /// @param diagnosis Updated diagnosis text (non-empty, ≤ MAX_LONG_LEN).
    /// @param treatment Updated treatment text (non-empty, ≤ MAX_LONG_LEN).
    /// @param notes     Updated notes (may be empty, ≤ MAX_LONG_LEN).
    function correctMedicalRecord(
        uint256 recordId,
        string calldata diagnosis,
        string calldata treatment,
        string calldata notes
    ) external {
        uint256 petId = _recordPetId[recordId];
        require(petId != 0, "PetChainRegistry: record does not exist");

        MedicalRecord storage rec = _petRecords[petId][_recordIndex[recordId]];
        require(
            msg.sender == rec.vet || msg.sender == admin,
            "PetChainRegistry: not authorised to correct record"
        );

        require(bytes(diagnosis).length > 0 && bytes(diagnosis).length <= MAX_LONG_LEN,
            "PetChainRegistry: invalid diagnosis length");
        require(bytes(treatment).length > 0 && bytes(treatment).length <= MAX_LONG_LEN,
            "PetChainRegistry: invalid treatment length");
        require(bytes(notes).length <= MAX_LONG_LEN,
            "PetChainRegistry: notes too long");

        // Snapshot originals for the event before overwriting
        string memory origDiagnosis = rec.diagnosis;
        string memory origTreatment = rec.treatment;
        string memory origNotes     = rec.notes;

        rec.diagnosis = diagnosis;
        rec.treatment = treatment;
        rec.notes     = notes;

        emit MedicalRecordCorrected(
            recordId,
            petId,
            msg.sender,
            origDiagnosis,
            origTreatment,
            origNotes,
            diagnosis,
            treatment,
            notes
        );
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
