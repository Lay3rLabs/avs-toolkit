# Simple Verifier

This is a rather minimal, but correct, implementation of a verifier contract.

- It does on-chain aggregation of the AVS results.
- It doesn't enforce any particular data format (only valid JSON).
- It only accepts exact matches in counting quorum (okay for a bridge / deterministic computation, not for an oracle).
- It does not do any slashing for votes not matching quorum.
- It does ensure that the proper validators voted before marking the task as completed.

It must be configured as follows:

- Operators points to a (DAO DAO groups interface?) contract that allows us to query total power and power by operator at previous heights
- A quorum is configured in this contract as too what percentage of voting power is needed to mark as completed
- At least one task queue has been deployed that references this contract as a trusted verifier

It works as follows:

- Anyone can post an "ExecutedTask" message, containing a Task Queue Contract Address, Task ID, and purported result.
- The verifier will ensure this is a valid vote on the contract:
  - Verifier will ensure the signer has not already submitted a vote on this task
  - Verifier will query the Task ID on the given Task Queue and ensure that it is still open (not completed, not expired), and get the creation height
  - Verifier will query the Operators contract to ensure the signer was an operator of this verifier at the height the task was created, and get their voting power
- The verifier will then aggregate it with existing votes (if any)
  - If the first vote, we record the metadata of the task (contract, id, total operator power at this height)
  - We create or update the proposal, indexed on the (contract, id, result) tuple. Recording the total number of voting power in favor of this
- The verifier will check if the last updated tuple now meets quorum, and if so:
  - It will execute a TaskCompleted message with on the specified TaskQueue contract with the result that has met quorum
  - If the TaskQueue does not accept this verifier (any more), the transaction will be reverted, meaning the last vote will not be counted
