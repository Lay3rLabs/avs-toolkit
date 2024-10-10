# Oracle Verifier

This is a smart contract for managing operator votes on task prices with a slashing mechanism for incorrect votes.

- It allows operators to vote on prices for tasks.
- Votes are filtered by a specified price range to ensure only valid votes are counted.
- It tracks the power of each operator when voting.
- If an operator's vote exceeds the allowed price spread, they can be slashed.
- The contract manages tasks, including checking if they have expired.

It must be configured as follows:

- Operators are set in the contract configuration.
- A threshold percentage for valid voting is configured.
- Allowed and slashable spreads are set to control the voting range.

It works as follows:

- Operators submit votes on task prices with their voting power.
- Votes are validated within the allowed price spread.
- If a vote is outside the slashable spread, the operator will be slashed.
- Tasks have expiration times, and the contract automatically checks if a task is expired.

