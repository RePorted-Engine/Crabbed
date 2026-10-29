# Native launcher bug-sweep notes

## Fixed pack mutation failures

`PackChoices` now queues loads and mutations. Each resource toggle reads the latest confirmed engine list before deriving its write, so two pending toggles do not each overwrite the list derived from an earlier cache. Disabling a pack is confirmed with a fresh read. Deletion runs under the same lock and only after a successful, confirmed disable; a backend error or unchanged selection leaves the file intact. Filesystem deletion errors are surfaced instead of discarded.

Controlled asynchronous simulator-hosted tests cover two queued resource toggles, disable failure, an unconfirmed successful reply, and disable-before-delete order.

## Deferred account-state finding

Code inspection found that `MicrosoftAccount.init` restores a cached profile from preferences while `handOver` returns early when the secure Vault has no secrets. A restored profile without its keychain record can therefore remain displayed with account-name editing disabled, while no session reaches the game. This state needs a separate product decision and a secret-store-injected regression test: either mark the account as requiring sign-in again, or clear stale cached identity without confusing it with an active Microsoft session. This finding has not been reproduced against a real account and is not changed by the pack fixes. Do not manipulate a user's real keychain to test it.
