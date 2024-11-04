# AI Bouncer Caller Contract

This contract serves as both a task queuer and a hook consumer for the AI
Bouncer. It should be authorized to create atomic task completion hooks upon
task creation for the AI Bouncer AVS and add members to a DAO if the AI Bouncer
determines that the user has met the membership requirements.

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

After instantiation:
- the DAO must update the admin of the cw4-group contract to this contract
- the AI Bouncer AVS admin must authorize this contract to create atomic task
  completion hooks on task creation

### Execution

To trigger the AI Bouncer, call `Trigger { session_id, message_id, message }`.

The contract handles the task completion hooks automatically:

- deserializes the task response
- if the AI bouncer made a decision about the user, it either registers that
  they were rejected or registers that they were approved and adds them to the
  cw4-group

The DAO can also teardown the contract via a `Unregister {}` message, which will
change the cw4-group contract admin back to the DAO.
