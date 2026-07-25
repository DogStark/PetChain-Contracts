// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "@openzeppelin/contracts/utils/Pausable.sol";

/// @title  PetChainRegistry
/// @notice Central registry for vets, pets, and medical records on the PetChain platform.
/// @dev    Inherits OpenZeppelin Pausable for emergency-stop functionality.
contract PetChainRegistry is Pausable {
    // -------------------------------------------------------------------------
    // Constants — string length limits (issue #919)
    // -------------------------------------------------------------------------

    /// @notice Maximum byte length for short string fields (name, species, breed, birthday).
    uint256 public constant MAX_SHORT_LEN = 64;

    /// @notice Maximum byte length for long string fields (diagnosis, treatment, notes).
    uint256 public constant MAX_LONG_LEN  = 1000;

    // -------------------------------------------------------------------------
    // State
    // -------------------------------------------------------------------------

    /// @notice Address of the contract administrator.
    address public admin;

    /// @notice Category of a medical record.
    enum RecordType { Checkup, Vaccination, Surgery, LabResult, Other }

    /// @notice Registered veterinarian.
    /// @param vetAddress     On-chain address of the vet.
    /// @param licenseNumber  Vet's professional licence number (original casing).
    /// @param specialization Vet's area of specialization.
    /// @param isVerified     True when the admin has verified the vet.
    /// @param isRevoked      True when the admin has revoked the vet's access.
    struct Vet {
        address vetAddress;
        string  licenseNumber;
        string  specialization;
        bool    isVerified;
        bool    isRevoked;
    }

    /// @notice Registered pet.
    /// @param petId   Unique identifier assigned at registration.
    /// @param owner   Current owner's address.
    /// @param name    Pet's name.
    /// @param species Species (e.g. "Dog", "Cat").
    /// @param breed   Breed descriptor.
    /// @param birthday Date of birth string.
    /// @param active  False when the pet has been deactivated.
    struct Pet {
        uint256 petId;
        address owner;
        string  name;
        string  species;
        string  breed;
        string  birthday;
        bool    active;
    }

    /// @notice A single medical record entry for a pet.
    /// @param recordId   Unique record identifier.
    /// @param petId      Pet this record belongs to.
    /// @param vet        Address of the vet who created the record.
    /// @param recordType Category of the record.
    /// @param diagnosis  Diagnosis text.
    /// @param treatment  Treatment text.
    /// @param notes      Additional notes (may be empty).
    /// @param timestamp  Block timestamp at time of creation.
    struct MedicalRecord {
        uint256 recordId;
        uint256 petId;
        address vet;
        RecordType recordType;
        string  diagnosis;
        string  treatment;
        string  notes;
        uint256 timestamp;
    }

    uint256 private _petCounter;
    uint256 private _recordCounter;
    uint256 private _vetCount;

    mapping(address => Vet)      public vets;
    mapping(uint256 => Pet)      public pets;
    mapping(uint256 => MedicalRecord[]) private _petRecords;
    mapping(address => uint256[]) private _ownerPets;
    mapping(uint256 => uint256) private _recordPet;   // recordId => petId
    mapping(uint256 => uint256) private _recordIndex; // recordId => index in _petRecords[petId]
    mapping(bytes32 => address)  private _licenseToVet;

    // recordId → petId, so correctMedicalRecord can locate the record
    mapping(uint256 => uint256) private _recordPetId;
    // recordId → index inside _petRecords[petId]
    mapping(uint256 => uint256) private _recordIndex;

    // Ordered list of all ever-registered vet addresses (issue #926)
    address[] private _vetAddresses;

    // -------------------------------------------------------------------------
    // Events
    // -------------------------------------------------------------------------

    /// @notice Emitted when a new vet registers.
    /// @param vet           Address of the registering vet.
    /// @param licenseNumber Licence number supplied at registration.
    event VetRegistered(address indexed vet, string licenseNumber);

    /// @notice Emitted when a vet updates their specialization.
    /// @param vet            Address of the vet.
    /// @param specialization New specialization string.
    event VetSpecializationUpdated(address indexed vet, string specialization);

    /// @notice Emitted when the admin verifies a vet.
    /// @param vet Address of the verified vet.
    event VetVerified(address indexed vet);

    /// @notice Emitted when the admin revokes a vet.
    /// @param vet Address of the revoked vet.
    event VetRevoked(address indexed vet);

    /// @notice Emitted when a new pet is registered.
    /// @param petId ID assigned to the new pet.
    /// @param owner Address of the registering owner.
    event PetRegistered(uint256 indexed petId, address indexed owner);

    /// @notice Emitted when a pet's ownership is transferred.
    /// @param petId ID of the transferred pet.
    /// @param from  Previous owner.
    /// @param to    New owner.
    event PetTransferred(uint256 indexed petId, address indexed from, address indexed to);

    /// @notice Emitted when a pet is deactivated.
    /// @param petId ID of the deactivated pet.
    event PetDeactivated(uint256 indexed petId);

    /// @notice Emitted when a previously deactivated pet is reactivated.
    /// @param petId ID of the reactivated pet.
    event PetReactivated(uint256 indexed petId);

    /// @notice Emitted when a medical record is added for a pet.
    /// @param petId    ID of the pet.
    /// @param recordId ID assigned to the new record.
    /// @param vet      Address of the vet who added the record.
    event MedicalRecordAdded(uint256 indexed petId, uint256 indexed recordId, address indexed vet);
    event MedicalRecordCorrected(
        uint256 indexed recordId,
        address indexed correctedBy,
        string  oldDiagnosis,
        string  oldTreatment,
        string  oldNotes,
        string  newDiagnosis,
        string  newTreatment,
        string  newNotes
    );

    /// @notice Emitted when a medical record is corrected.
    /// @dev    The original field values are preserved in the event log for full auditability.
    /// @param recordId         ID of the corrected record.
    /// @param petId            Pet the record belongs to.
    /// @param correctedBy      Address that performed the correction.
    /// @param originalDiagnosis Previous diagnosis text.
    /// @param originalTreatment Previous treatment text.
    /// @param originalNotes     Previous notes text.
    /// @param newDiagnosis      Updated diagnosis text.
    /// @param newTreatment      Updated treatment text.
    /// @param newNotes          Updated notes text.
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

    /// @notice Emitted when the admin role is transferred.
    /// @param previousAdmin The outgoing admin address.
    /// @param newAdmin      The incoming admin address.
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

    /// @notice Deploys the registry and sets the deployer as admin.
    constructor() {
        admin = msg.sender;
    }

    // -------------------------------------------------------------------------
    // Admin management
    // -------------------------------------------------------------------------

    /// @notice Transfer the admin role to a new address.
    /// @param newAdmin The address that will become the new admin.
    function transferAdmin(address newAdmin) external onlyAdmin {
        require(newAdmin != address(0), "PetChainRegistry: zero address");
        address previous = admin;
        admin = newAdmin;
        emit AdminTransferred(previous, newAdmin);
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

    /// @notice Register the caller as a vet with the given licence and specialization.
    /// @dev    Licence uniqueness is enforced case-insensitively.
    /// @param licenseNumber  Professional licence number (non-empty).
    /// @param specialization Area of specialization.
    function registerVet(string calldata licenseNumber, string calldata specialization) external whenNotPaused {
        require(bytes(licenseNumber).length > 0, "PetChainRegistry: empty licenseNumber");

        bytes32 key = _normalizeLicenseKey(licenseNumber);
        address existingHolder = _licenseToVet[key];
        require(existingHolder == address(0) || existingHolder == msg.sender,
            "PetChainRegistry: license already registered");

        bytes memory prevLicense = bytes(vets[msg.sender].licenseNumber);
        if (prevLicense.length > 0) {
            bytes32 prevKey = _normalizeLicenseKey(string(prevLicense));
            if (prevKey != key) {
                delete _licenseToVet[prevKey];
            }
        }
        _licenseToVet[key] = msg.sender;

        if (vets[msg.sender].vetAddress == address(0)) {
            _vetCount++;
            _vetAddresses.push(msg.sender); // issue #926 — maintain enumerable directory
        }
        vets[msg.sender] = Vet({
            vetAddress:     msg.sender,
            licenseNumber:  licenseNumber,
            specialization: specialization,
            isVerified:     false,
            isRevoked:      false
        });
        emit VetRegistered(msg.sender, licenseNumber);
    }

    /// @notice Upper-cases an ASCII licence string and hashes it for case-insensitive uniqueness.
    /// @param licenseNumber The raw licence number string.
    /// @return Keccak256 hash of the upper-cased licence bytes.
    function _normalizeLicenseKey(string memory licenseNumber) internal pure returns (bytes32) {
        bytes memory raw = bytes(licenseNumber);
        bytes memory normalized = new bytes(raw.length);
        for (uint256 i = 0; i < raw.length; i++) {
            bytes1 c = raw[i];
            if (c >= 0x61 && c <= 0x7A) { // 'a'-'z'
                normalized[i] = bytes1(uint8(c) - 32);
            } else {
                normalized[i] = c;
            }
        }
        return keccak256(normalized);
    }

    /// @notice Update the calling vet's own specialization.
    /// @param specialization New specialization string.
    function updateSpecialization(string calldata specialization) external whenNotPaused {
        require(vets[msg.sender].vetAddress == msg.sender, "PetChainRegistry: not a registered vet");
        vets[msg.sender].specialization = specialization;
        emit VetSpecializationUpdated(msg.sender, specialization);
    }

    /// @notice Verify a registered vet. Only callable by admin.
    /// @param vet Address of the vet to verify.
    function verifyVet(address vet) external onlyAdmin whenNotPaused {
        require(!vets[vet].isRevoked, "PetChainRegistry: vet is revoked");
        vets[vet].isVerified = true;
        emit VetVerified(vet);
    }

    /// @notice Revoke a vet's verified status. Only callable by admin.
    /// @param vet Address of the vet to revoke.
    function revokeVet(address vet) external onlyAdmin whenNotPaused {
        vets[vet].isVerified = false;
        vets[vet].isRevoked  = true;
        emit VetRevoked(vet);
    }

    // -------------------------------------------------------------------------
    // Pet management
    // -------------------------------------------------------------------------

    /// @notice Register a new pet owned by the caller.
    /// @param name     Pet's name (1–MAX_SHORT_LEN bytes).
    /// @param species  Species descriptor (1–MAX_SHORT_LEN bytes).
    /// @param breed    Breed descriptor (1–MAX_SHORT_LEN bytes).
    /// @param birthday Date-of-birth string (1–MAX_SHORT_LEN bytes).
    /// @return petId   The ID assigned to the newly registered pet.
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

    /// @notice Transfer ownership of a pet to another address.
    /// @param petId ID of the pet to transfer.
    /// @param to    Recipient address (non-zero).
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

    /// @notice Deactivate a pet. Only callable by the pet's owner.
    /// @param petId ID of the pet to deactivate.
    function deactivatePet(uint256 petId) external onlyPetOwner(petId) whenNotPaused {
        require(pets[petId].active, "PetChainRegistry: already inactive");
        pets[petId].active = false;
        emit PetDeactivated(petId);
    }

    /// @notice Reactivate a previously deactivated pet. Only callable by the pet's owner.
    /// @param petId ID of the pet to reactivate.
    function reactivatePet(uint256 petId) external onlyPetOwner(petId) whenNotPaused {
        require(!pets[petId].active, "PetChainRegistry: already active");
        pets[petId].active = true;
        emit PetReactivated(petId);
    }

    // -------------------------------------------------------------------------
    // Medical records
    // -------------------------------------------------------------------------

    /// @notice Add a medical record for a pet. Only callable by verified vets.
    /// @param petId      ID of the pet.
    /// @param recordType Category of the record.
    /// @param diagnosis  Diagnosis text (1–MAX_LONG_LEN bytes).
    /// @param treatment  Treatment text (1–MAX_LONG_LEN bytes).
    /// @param notes      Additional notes (0–MAX_LONG_LEN bytes).
    /// @return recordId  The ID assigned to the new record.
    function addMedicalRecord(
        uint256 petId,
        RecordType recordType,
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
        _recordPet[recordId]   = petId;
        _recordIndex[recordId] = _petRecords[petId].length;
        _petRecords[petId].push(MedicalRecord({
            recordId:   recordId,
            petId:      petId,
            vet:        msg.sender,
            recordType: recordType,
            diagnosis:  diagnosis,
            treatment:  treatment,
            notes:      notes,
            timestamp:  block.timestamp
        }));
        _recordPetId[recordId] = petId;
        _recordIndex[recordId] = _petRecords[petId].length - 1;
        emit MedicalRecordAdded(petId, recordId, msg.sender);
    }

    /// @notice Correct a previously added medical record.
    /// Only the original recording vet or admin may call this.
    /// @notice Return all medical records for a pet matching a given record type.
    /// @param petId      ID of the pet.
    /// @param recordType Category to filter by.
    /// @return filtered  Array of matching MedicalRecord structs.
    function getPetRecordsByType(uint256 petId, RecordType recordType)
        external
        view
        returns (MedicalRecord[] memory filtered)
    {
        MedicalRecord[] storage all = _petRecords[petId];
        uint256 total = all.length;

        uint256 count;
        for (uint256 i = 0; i < total; i++) {
            if (all[i].recordType == recordType) count++;
        }

        filtered = new MedicalRecord[](count);
        uint256 j;
        for (uint256 i = 0; i < total; i++) {
            if (all[i].recordType == recordType) {
                filtered[j] = all[i];
                j++;
            }
        }
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
        uint256 petId = _recordPet[recordId];
        MedicalRecord storage rec = _petRecords[petId][_recordIndex[recordId]];
        require(rec.recordId == recordId, "PetChainRegistry: record not found");
        require(
            msg.sender == rec.vet || msg.sender == admin,
            "PetChainRegistry: not authorized"
        );
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

        emit MedicalRecordCorrected(
            recordId, msg.sender,
            rec.diagnosis, rec.treatment, rec.notes,
            diagnosis, treatment, notes
        );
        rec.diagnosis = diagnosis;
        rec.treatment = treatment;
        rec.notes     = notes;
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

    /// @notice Return all pet IDs owned by `owner`.
    /// @param owner The owner address to query.
    /// @return      Array of pet IDs belonging to `owner`.
    function getPetsByOwner(address owner) external view returns (uint256[] memory) {
        return _ownerPets[owner];
    }

    /// @notice Total number of vets ever registered.
    /// @return Count of unique vet addresses that have called registerVet.
    function getTotalVets() external view returns (uint256) {
        return _vetCount;
    }

    /// @notice Whether `petId` is currently active.
    /// @param petId The pet to query.
    /// @return      True if the pet is active, false if deactivated.
    function isPetActive(uint256 petId) external view returns (bool) {
        return pets[petId].active;
    }

    /// @notice Return all medical records for a pet.
    /// @param petId ID of the pet to query.
    /// @return      Full array of MedicalRecord structs for the pet.
    function getPetRecords(uint256 petId) external view returns (MedicalRecord[] memory) {
        return _petRecords[petId];
    }

    /// @notice Return record IDs for `petId` whose timestamp falls within [startDate, endDate].
    /// @param petId     The pet to query.
    /// @param startDate Lower bound Unix timestamp (inclusive).
    /// @param endDate   Upper bound Unix timestamp (inclusive).
    /// @return ids      Array of matching record IDs.
    function getPetRecordsByDateRange(uint256 petId, uint256 startDate, uint256 endDate)
        external
        view
        returns (uint256[] memory ids)
    {
        MedicalRecord[] storage all = _petRecords[petId];
        uint256 total = all.length;

        uint256 count;
        for (uint256 i = 0; i < total; i++) {
            uint256 ts = all[i].timestamp;
            if (ts >= startDate && ts <= endDate) count++;
        }

        ids = new uint256[](count);
        uint256 j;
        for (uint256 i = 0; i < total; i++) {
            uint256 ts = all[i].timestamp;
            if (ts >= startDate && ts <= endDate) {
                ids[j] = all[i].recordId;
                j++;
            }
        }
    }

    // -------------------------------------------------------------------------
    // Paginated view functions (issue #918)
    // -------------------------------------------------------------------------

    /// @notice Return a page of pet IDs owned by `owner`.
    /// @param owner  The owner address to query.
    /// @param offset Starting index (0-based).
    /// @param limit  Maximum number of items to return.
    /// @return page  Slice of the owner's pet ID array.
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
    /// @return page  Slice of the pet's medical record array.
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

    // -------------------------------------------------------------------------
    // Paginated vet directory (issue #926)
    // -------------------------------------------------------------------------

    /// @notice Return a page of Vet structs from the registered vet directory.
    /// @param offset Starting index into the vet address list (0-based).
    /// @param limit  Maximum number of Vet structs to return.
    /// @return page  Slice of Vet structs for the requested page.
    function getVets(uint256 offset, uint256 limit) external view returns (Vet[] memory page) {
        uint256 total = _vetAddresses.length;
        if (offset >= total || limit == 0) {
            return new Vet[](0);
        }
        uint256 end = offset + limit;
        if (end > total) end = total;
        uint256 size = end - offset;
        page = new Vet[](size);
        for (uint256 i = 0; i < size; i++) {
            page[i] = vets[_vetAddresses[offset + i]];
        }
    }
}
