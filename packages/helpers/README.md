# Verifier Utilities

This package provides utility functions for verifying operators in smart contracts. It includes methods for ensuring valid votes and handling task metadata.

## Key Components
`VerifierError`
An enum that defines various error types that can occur during the verification process.
`ensure_valid_vote`
This function performs all necessary checks to ensure a voter is valid and has not voted yet. It also verifies that the task is valid and still open.
`load_or_initialize_metadata`
This function checks existing task metadata or initializes new metadata if it doesn't exist.


## Usage
### To use these utilities in your smart contract:
Import the necessary dependencies.
Use ensure_valid_vote to validate votes from operators.

### Error Handling
The VerifierError enum provides specific error types for different scenarios. Make sure to handle these errors appropriately in your contract logic.
