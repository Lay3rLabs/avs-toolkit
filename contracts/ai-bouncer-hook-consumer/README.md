# AI Bouncer Consumer Contract

This contract serves as a hook consumer for the AI Bouncer. It should be
registered to respond to task completion events for the AI Bouncer AVS and add
members to a DAO if the AI Bouncer determines that the user has met the
membership requirements.

## Usage

### Instantiation

The contract is instantiated with the DAO, cw4-group, and task queue addresses,
as well as the cw4-group weight to assign to new members:

```
{
  "dao": "layer...",
  "cw4_group": "layer...",
  "task_queue": "layer...",
  "new_member_weight": 1
}
```

This contract will automatically register itself as a hook consumer on the task
queue.

After instantiation, the DAO must update the admin of the cw4-group contract to
this contract.

### Execution

The contract handles the `TaskCreatedHook` messages automatically:

- deserializes the task response
- if the AI bouncer made a decision about the user, it either registers that
  they were rejected or registers that they were approved and adds them to the
  cw4-group

The DAO can also teardown the contract via a `Unregister {}` message, which will
change the cw4-group contract admin back to the DAO. It should also unregister
the contract as a task queue hook consumer, once that's possible.

After unregistering, you may re-register as a task queue hook consumer via a
`Register {}` message. The DAO must manually update the cw4-group contract admin
back to this contract.
