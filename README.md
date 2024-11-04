Storymint
---------

This is an implementation of a certain NFT project. The NFT project differs from others in that the buyer locks a yield bearing token when the NFT is minted. The NFT can be burned, unlocking the token. The NFT project receives funds from the yield, which is diverted to the project.

Additionally, NFT metadata can be updated dynamically, as the user progresses through a story.

Devnet
------
The anchor program is deployed on devnet. The frontend is deployed with firebase hosting: [storymint.web.app](https://storymint.web.app)

Instructions
------------

The anchor program has four instructions:

1. `initialize_collection`
2. `mint_asset`
3. `update_metadata`
4. `burn_and_withdraw`

For simplicity, rather than a yield bearing token, this implementation locks 1 SOL.

The expected roles are:

1. server authority
    * can create collections
    * can update metadata
2. users
    * can mint assets, locking 1 SOL on mint
      * minted NFTs should be transferable to other users
        * if transfered, the new holder can burn the NFT, unlocking the 1 SOL
    * can burn the NFT, unlocking the 1 SOL

Tests
-----
[It seems metaplex mpl-core uses umi for tests](https://github.com/metaplex-foundation/mpl-core/blob/main/clients/js/test/_setupRaw.ts), so I tried that.

To run the tests:

```bash
pnpm install
pnpm anchor-test
```

```bash
Running test suite: 

 PASS  tests/storymint.spec.ts (353.673 s)
  Storymint
    ✓ should initialize collection with correct master state (29 ms)
    ✓ should fail with unauthorized update authority (22 ms)
    ✓ should fail to initialize same collection twice (13470 ms)
    ✓ should successfully mint an asset (13572 ms)
    ✓ should increment master state total minted (13490 ms)
    ✓ should handle concurrent mints correctly (13519 ms)
    ✓ should fail to mint an asset without a collection (33 ms)
    ✓ should update metadata URI and name (27052 ms)
    ✓ should update metadata after transfer to another user (40580 ms)
    ✓ should fail with unauthorized authority (13516 ms)
    ✓ should burn asset and withdraw SOL (27019 ms)
    ✓ should decrement master state total minted (26998 ms)
    ✓ should transfer the asset to another user who can burn it and withdraw SOL (54025 ms)
    ✓ should fail to burn by the original owner, after transfer to another user (40552 ms)
    ✓ should fail to burn with update authority (13519 ms)
    ✓ should fail to burn with wrong owner (13509 ms)

Test Suites: 1 passed, 1 total
Tests:       16 passed, 16 total
```

Frontend
--------

To run the frontend locally:

```bash
pnpm dev
```
