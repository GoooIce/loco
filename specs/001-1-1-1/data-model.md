# Data Model: Loco-MCP System

**Created**: 2025-10-03
**Purpose**: Define entities, relationships, and validation rules for MCP tools

## Core Entities

### 1. ModelGenerationRequest
Represents a request to generate a model with specified fields.

**Attributes**:
- `model_name`: string (required) - Name of the model to generate
- `fields`: list of FieldDefinition (required) - Field definitions for the model
- `table_name`: string (optional) - Custom table name (derived from model_name if not provided)

**Validation Rules**:
- model_name must follow loco-rs naming conventions (snake_case, valid Rust identifier)
- fields list must contain at least one field
- Each field must have valid type and constraints

### 2. FieldDefinition
Defines a single field in a model.

**Attributes**:
- `name`: string (required) - Field name (snake_case)
- `type`: FieldType (required) - Data type
- `constraints`: list of FieldConstraint (optional) - Field constraints
- `optional`: boolean (default: false) - Whether field is optional

**Validation Rules**:
- name must be valid Rust identifier
- type must be supported loco-rs field type
- Constraints must be compatible with field type

### 3. FieldType
Enumeration of supported field types.

**Supported Types**:
- `string` - Text fields
- `i32`, `i64` - Integer types
- `f32`, `f64` - Float types
- `boolean` - Boolean values
- `datetime` - Date and time values
- `uuid` - UUID values
- `json` - JSON data
- `text` - Long text fields

### 4. FieldConstraint
Constraints that can be applied to fields.

**Constraint Types**:
- `unique` - Field must have unique values
- `primary_key` - Field is primary key
- `foreign_key` - Field references another model
- `default:<value>` - Default value for field
- `nullable` - Field can be null (database level)

### 5. ScaffoldGenerationRequest
Represents a request to generate complete CRUD scaffolding.

**Attributes**:
- `model_name`: string (required) - Name of the model
- `fields`: list of FieldDefinition (required) - Model fields
- `include_views`: boolean (default: true) - Generate view templates
- `include_controllers`: boolean (default: true) - Generate controller
- `api_only`: boolean (default: false) - API-only scaffolding

**Validation Rules**:
- All ModelGenerationRequest validation rules apply
- Cannot have both api_only and include_views as true

### 6. ControllerViewGenerationRequest
Represents a request to generate controllers and views for existing model.

**Attributes**:
- `model_name`: string (required) - Name of existing model
- `actions`: list of string (optional) - Specific controller actions to generate
- `view_types`: list of ViewType (optional) - Types of views to generate

**Validation Rules**:
- model_name must correspond to existing model in project
- actions must be valid controller actions (index, show, create, update, delete)
- view_types must be supported (list, show, form, edit)

### 7. GenerationResponse
Structured response from generation operations.

**Attributes**:
- `success`: boolean - Whether operation succeeded
- `created_files`: list of FilePath - Files that were created
- `modified_files`: list of FilePath - Files that were modified
- `errors`: list of ErrorMessage - Error messages (if any)
- `warnings`: list of WarningMessage - Warning messages (if any)

### 8. FilePath
Represents a file path with additional metadata.

**Attributes**:
- `path`: string - Relative file path
- `type`: FileType - Type of file (model, controller, view, migration, etc.)
- `size_bytes`: integer - File size in bytes
- `checksum`: string - File content checksum

### 9. FileType
Enumeration of file types that can be generated.

**File Types**:
- `model` - Rust model file
- `migration` - Database migration file
- `controller` - Controller file
- `view` - View template file
- `route` - Route definition file
- `test` - Test file
- `config` - Configuration file

## Entity Relationships

```
ModelGenerationRequest
├── FieldDefinition (1..*)
│   ├── FieldType
│   └── FieldConstraint (0..*)
└── GenerationResponse (result)

ScaffoldGenerationRequest
├── ModelGenerationRequest (composition)
└── GenerationResponse (result)

ControllerViewGenerationRequest
└── GenerationResponse (result)

GenerationResponse
├── FilePath (0..*)
│   └── FileType
├── ErrorMessage (0..*)
└── WarningMessage (0..*)
```

## State Transitions

### Model Generation Flow
1. **Validate Request** → Check naming, types, constraints
2. **Check Existing Model** → Verify model doesn't exist
3. **Generate Model File** → Create Rust model struct
4. **Generate Migration** → Create database migration
5. **Update Response** → Add generated files to response

### Scaffold Generation Flow
1. **Validate Request** → Check model, fields, options
2. **Generate Model** → Reuse model generation logic
3. **Generate Controller** → Create REST controller
4. **Generate Views** → Create view templates
5. **Generate Routes** → Update route definitions
6. **Update Response** → Add all generated files

### Controller/View Generation Flow
1. **Validate Existing Model** → Check model exists
2. **Generate Controller** → Create controller with actions
3. **Generate Views** → Create specified view types
4. **Update Response** → Add generated files

## Input Validation Rules

### Model Names
- Must be snake_case
- Must start with letter
- Can contain letters, numbers, underscores
- Maximum 64 characters
- Cannot be Rust reserved keyword

### Field Names
- Same rules as model names
- Cannot be 'id' (reserved for primary key)
- Cannot duplicate existing field names

### Field Types
- Must be in supported types list
- Constraints must be type-compatible
- Optional fields must support nullability

### File Paths
- Must be within project directory
- Cannot overwrite existing files (unless explicitly requested)
- Must follow loco-rs directory structure

## Error Handling

### Validation Errors
- `InvalidModelName`: Model name doesn't follow conventions
- `InvalidFieldName`: Field name doesn't follow conventions
- `UnsupportedFieldType`: Field type not supported
- `IncompatibleConstraints`: Constraint incompatible with field type
- `DuplicateField`: Field name already exists in model

### File System Errors
- `FileAlreadyExists`: Target file already exists
- `PermissionDenied`: Insufficient permissions for file operation
- `DiskFull`: Insufficient disk space
- `PathNotFound`: Target directory doesn't exist

### Project Errors
- `NotLocoProject`: Current directory is not a loco-rs project
- `ModelAlreadyExists`: Model with given name already exists
- `ModelNotFound`: Specified model doesn't exist

## Performance Considerations

### Memory Usage
- Field definitions are lightweight (~100 bytes per field)
- Generation responses include file lists but not file contents
- Large models (>100 fields) may require chunked processing

### Processing Time
- Validation: ~1ms per model
- File generation: ~2-5ms per file
- Response formatting: ~1ms

### Caching Strategy
- Template compilation results cached
- Project structure validation cached
- File existence checks cached per operation