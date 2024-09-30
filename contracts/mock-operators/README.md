# Mock Operators Contract

This is a simple contract to simulate a set of operators with voting power for use with an AVS. It provides basic functionality to query voting power and total power at specific heights, as well as list all voters.

## Actions

### Instantiate

The contract is instantiated with a list of operators and their voting powers:

```rust
pub struct InstantiateMsg {
    pub operators: Vec<InstantiateOperator>,
}

pub struct InstantiateOperator {
    pub addr: String,
    pub voting_power: u32,
}
```

During instantiation, the contract:
- Validates operator addresses
- Calculates the total voting power
- Stores the configuration in the `CONFIG` item

### Execute

Currently, there are no execute messages implemented for this contract.

### Query

- `VotingPowerAtHeight`: Get the voting power of a specific address at a given height (or latest if not specified).
- `TotalPowerAtHeight`: Get the total voting power at a given height (or latest if not specified).
- `AllVoters`: List all voters (operators) and their voting powers.
