# Tranche 2 testnet receipt archive

Envelope, result, meta and diagnostic-event XDR of every Tranche 2 testnet transaction not already archived with its XDR, recorded before SDF testnet RPC retention (120,960 ledgers, about 7 days) drops them. All are in the pre-2026-12-16-reset epoch ([A76](../../../doc/wiki/decisions.md)).

- **Command:** `python3 scripts/record-evidence.py --testnet docs/evidence/<file> <hash>...`, one run per file, on 2026-11-11 (15:38 UTC; `t2-d4-10-step2b.json` at 15:49 UTC, `t2-d4-10-step2c-drill.json` at 16:32 UTC, `t2-d5-user-deposit-1.json` at 18:09 UTC, `t2-keeper-cycles-2026-11-11-evening.json` at 19:58 UTC), and `t2-keeper-cycles-2026-11-12.json` on 2026-11-12 at 14:16 UTC; `t2-liquidity-setup-2026-11-12.json` was recorded on 2026-11-12 at 15:07 UTC (its `recorded_at`; that it was written by this command is inferred from its format, which is the same). Each run asserts the RPC passphrase and `SUCCESS` for every hash. `t2-liquidity-faucet-keys-2026-11-12.json` at 15:18 UTC (2026-11-12), and `t2-d5-rebalance-run-2026-11-12.json` at 20:22 UTC (2026-11-12), and `t2-liquidity-remove-2026-11-12.json` at about 21:37 UTC (2026-11-12), and `t2-d6-04-basket-exit-2026-11-12.json` at about 22:53 UTC (2026-11-12), and `t2-liquidity-refills-2026-11-13.json` at about 07:25 UTC (2026-11-13), and `t2-a102-realign-2026-11-13.json` at about 08:28 UTC (2026-11-13).
- **Endpoint:** `https://soroban-testnet.stellar.org`. **Manifest:** `deploy/testnet/pre-2026-12-16-reset.json`, SHA-256 `cfa30d48adb6c5559dfbed446a4a0763c67ed0672305b18910e493b1920c2220`.
- **Hashes:** resolved in full from Horizon operations of the flagship executor `GDGCL46K…JT7N`, `etesia-admin` `GAQKH3PL…DUVY`, `etesia-realign` `GDRVYVG3…ZM3R`, `etesia-v3-treasury`, the guard keys, the faucet wallets and the user browser wallets, from ledger 5,650,000 (failed included); last read 2026-11-11 19:58 UTC at ledger 5,680,691, when all 229 were archived. The 14 failed ones are the D4-02 failed swap and the 13 D4-09 failing executes, archived elsewhere (below). Read again from Horizon operations and transactions of the same accounts on 2026-11-12 at 14:15 UTC (latest ledger 5,693,851): 136 transactions after ledger 5,680,691, none failed; 133 are in `t2-keeper-cycles-2026-11-12.json`, two in `t2-d5-allocation-run-2026-11-12.json`, and the tRESERVE mint `bb02fc03…478b` is left to the liquidity seed's archive.
- **Check:** every archived envelope hashes (testnet passphrase) to its recorded hash, and every result and meta XDR decodes (stellar-sdk, 206 of 206 entries in the twelve `t2-*.json` files; re-run on 2026-11-12 at 14:16 UTC, 341 of 341 entries in the fourteen `t2-*.json` files; `t2-liquidity-setup-2026-11-12.json` checked on its own on 2026-11-12 at 15:12 UTC, 28 of 28). Execute actions below are read from the meta's contract events.

| File | Scope | Txs | Ledgers | Closes (UTC) |
| --- | --- | --- | --- | --- |
| [`t2-d4-02-03-setup.json`](t2-d4-02-03-setup.json) | D4-02 and D4-03 setup: keys, trustlines, faucet funding, v3 copy, tRESERVE pair, provider, flagship WASM and guard deploys, realigns. Receipts already archived elsewhere are not repeated (see below). | 30 | 5,657,737–5,661,064 | 2026-11-10 12:05 – 2026-11-10 16:42 |
| [`t2-d4-06-receipts.json`](t2-d4-06-receipts.json) | D4-06: the deployed keeper's first two cycles. | 3 | 5,665,600–5,665,790 | 2026-11-10 23:00 – 2026-11-10 23:16 |
| [`t2-d4-08-receipts.json`](t2-d4-08-receipts.json) | D4-08: the Blend Supply and Withdraw cited for criterion 7 (`aa8f8e19…` is also in fixture `02-execute.json`; `4ed46e77…` also in `t2-d4-06-receipts.json`). | 2 | 5,661,252–5,665,790 | 2026-11-10 16:58 – 2026-11-10 23:16 |
| [`t2-keeper-cycles-2026-11-10-to-11.json`](t2-keeper-cycles-2026-11-10-to-11.json) | Every other deployed-keeper transaction from cycle 3 (2026-11-10 23:31 UTC) up to the `flat` realign (2026-11-11 13:02 UTC). | 97 | 5,665,972–5,675,691 | 2026-11-10 23:31 – 2026-11-11 13:01 |
| [`t2-a77-resume-announce.json`](t2-a77-resume-announce.json) | A77: guard `announce(Resume)`. | 1 | 5,675,326–5,675,326 | 2026-11-11 12:31 – 2026-11-11 12:31 |
| [`t2-d4-10-step1.json`](t2-d4-10-step1.json) | D4-10 step 1, ledgers 5,675,700–5,677,422: the tRESERVE realign that opened it, `flat` publishes from cycle 55 (including the skipped cycles 55–62), and the three A80 flat plans each after a realign. The step-1 Blend Supply is `aa8f8e19…b4c8` (D4-03, under `flat` target `d89fe725…630b`), archived in `t2-d4-08-receipts.json` and `fixtures/d5-04-testnet/01`–`02`. | 17 | 5,675,700–5,677,422 | 2026-11-11 13:02 – 2026-11-11 15:25 |
| [`t2-d4-10-step2.json`](t2-d4-10-step2.json) | D4-10 step 2 from ledger 5,677,423 as of ledger 5,677,483 (recorded 15:38 UTC): the XLM realign, the publish and the Withdraw `8e6056c7…`. Step 2 continues in `t2-d4-10-step2b.json`. | 3 | 5,677,455–5,677,483 | 2026-11-11 15:28 – 2026-11-11 15:30 |
| [`t2-d4-10-step2b.json`](t2-d4-10-step2b.json) | D4-10 step 2 as of ledger 5,677,678 (recorded 15:49 UTC): two XLM realigns, a publish and a Swap execute. Step 2 continues; later receipts go in a new file. | 4 | 5,677,637–5,677,678 | 2026-11-11 15:43 – 2026-11-11 15:47 |
| [`t2-faucet-a81.json`](t2-faucet-a81.json) | A81 faucet float: the three fresh `etesia-faucet-1`–`3` keys (friendbot), their Blend faucet claims and the 1,000-USDC payments. | 9 | 5,677,274–5,677,299 | 2026-11-11 15:13 – 2026-11-11 15:15 |
| [`t2-keeper-cycles-2026-11-11-evening.json`](t2-keeper-cycles-2026-11-11-evening.json) | Every tracked-account transaction after `t2-d4-10-step2c-drill.json` up to ledger 5,680,691 (19:58 UTC) not archived elsewhere: keeper cycles 72–87, the XLM realign during D4-09, and the user wallet's funding, `withdraw_bounded` and `redeem_bounded` after the hosted-page deposit. | 31 | 5,678,145–5,680,678 | 2026-11-11 16:26 – 2026-11-11 19:57 |
| [`t2-keeper-cycles-2026-11-12.json`](t2-keeper-cycles-2026-11-12.json) | Every tracked-account transaction after ledger 5,680,691 up to the Horizon read at ledger 5,693,851 (2026-11-12 14:15 UTC) not archived elsewhere: 69 keeper publishes and 64 executes (nonces 68–131), the Builder run and the tRESERVE mint excluded. | 133 | 5,680,861–5,693,700 | 2026-11-11 20:12 – 2026-11-12 14:02 |
| [`t2-liquidity-setup-2026-11-12.json`](t2-liquidity-setup-2026-11-12.json) | A99 testnet liquidity setup and seed: the `etesia-liquidity` account (friendbot), its USDC and tRESERVE trustlines, the 50,000 tRESERVE mint, ten Blend faucet claims (`etesia-faucet-4`–`13`) each with its forward-and-merge, the XLM and tRESERVE realigns before the seed, and the seed of both pairs. The ten faucet keys' friendbot creations are not in it (below). | 28 | 5,692,774–5,694,451 | 2026-11-12 12:45 – 2026-11-12 15:04 |
| [`t2-liquidity-faucet-keys-2026-11-12.json`](t2-liquidity-faucet-keys-2026-11-12.json) | A99 faucet keys: the Friendbot `create_account` of each of `etesia-faucet-4`–`13` (hashes from each key's first Horizon operation, read 2026-11-12). Their claims and forward/merges are in `t2-liquidity-setup-2026-11-12.json`. | 10 | 5,692,784–5,694,431 | 2026-11-12 12:46 – 2026-11-12 15:03 |
| [`t2-liquidity-remove-2026-11-12.json`](t2-liquidity-remove-2026-11-12.json) | D6-04 row 13: a small real `liquidity remove` from the tRESERVE/USDC pair (A101). | 1 | 5,699,152 | 2026-11-12 21:36 |
| [`t2-d6-04-basket-exit-2026-11-12.json`](t2-d6-04-basket-exit-2026-11-12.json) | D6-04 row 6: a basket exit (partial `withdraw_bounded`, then the full `redeem_bounded`) by a fresh Etesia test key with the keeper and indexer off, and its setup. | 6 | 5,699,983–5,700,052 | 2026-11-12 22:45 – 2026-11-12 22:51 |
| [`t2-liquidity-refills-2026-11-13.json`](t2-liquidity-refills-2026-11-13.json) | A99 automated refills by the quant service after `ETESIA_BLEND_FAUCET_URL` was set: three fresh faucet keys, each Friendbot, claim, forward-and-merge into `etesia-liquidity`. | 9 | 5,705,784–5,706,147 | 2026-11-13 06:49 – 2026-11-13 07:19 |
| [`t2-a102-realign-2026-11-13.json`](t2-a102-realign-2026-11-13.json) | A102: the first just-in-time realign by the keeper, XLM/USDC toward the Reflector mid. | 1 | 5,706,941 | 2026-11-13 08:25 |

**Archived elsewhere, not repeated:** D4-02 direct proof `e58ed4546ea00938b83f512ba8fec968e3b5e994e0010d6fb4420f2900991e35` and failed swap `4c55ed60dc50e7eaf29d6a0cd9bfe47d2aaf085e8eb641a236826103637c37bf` (FAILED by design) in `d4-testnet-probes.json` keys `direct_proof` and `failed_swap`; flagship deploy `35ca57046571f19315aa19fbb20f864328c0d04735d8034dac03f77b153b614d`, D4-03 publish `d89fe72583d47db84521c19c30a087bd2928da9a7f4efc028fd8dfd25e05630b`, D4-04 publish `f76c3c1a0e6eba91a3c7b12af3e5b8bc011f4e420bc0b012570ba122d68d09a5`, D4-05 publish `d24e9f3363fb107038be5c477f80f895109dfe12ddc40ce0f12261844eb380f9` and execute `7b7bf088079423e92fc5c3df2f31683fd11a8b4ae408a08bb81a0bd4203fd408` in `fixtures/d5-04-testnet/00`–`05`.

**Not archived:** the third-party v3 copy deploy `a383fe94…c91e` (2026-10-16, before Tranche 2 and outside retention). The D4-09 guard cases are archived by their harness in [d4-09-guard-pre-2026-12-16-reset.jsonl](d4-09-guard-pre-2026-12-16-reset.jsonl) (vault `602ef34`). Records are written once, so later keeper transactions (after ledger 5,693,700) go in a new file, recorded before about 2026-11-19 (retention).

## t2-d4-02-03-setup.json

| Ledger | Close (UTC) | Hash | Label |
| --- | --- | --- | --- |
| 5,657,737 | 2026-11-10 12:05:28 | `a97c04cab0d51f565c706c944a55604c66241a4ab3aff5e4b05fa6b2a7e1a568` | Friendbot creates `etesia-admin` |
| 5,657,742 | 2026-11-10 12:05:53 | `81a4e5aea3c1593538a031785c16a50d1c948bfe99f23d99e183512c03173f7e` | Friendbot creates `etesia-v3-treasury` |
| 5,657,747 | 2026-11-10 12:06:18 | `b26762c29e471557c54b0a44733e0c5a28e6eef3b9b8661d06505e7a8ceb8e2d` | Friendbot creates `etesia-realign` (tRESERVE issuer) |
| 5,657,751 | 2026-11-10 12:06:38 | `e54809f1705023cd2d4618f0a0b45cc035ab2ea3beb2b9de121f356cf3d4ad29` | `etesia-realign` Blend USDC trustline (D4-02) |
| 5,657,766 | 2026-11-10 12:07:53 | `a015174b2cbb6e395a3e432af0f6e9ac88ef11c6f50641fdb73d14bb801a3a60` | v3 aggregator copy deploy `CBE32GUT…V5TH` (D4-02) |
| 5,657,773 | 2026-11-10 12:08:28 | `bb2cdd925b9ec4895d54696b0d5b8438e875ff874b5303841e86e90fb7960f91` | v3 copy instance TTL extend (D4-02) |
| 5,657,780 | 2026-11-10 12:09:03 | `5c3fba248c80fb1b89c47959e453858d9883e9e5e62bbf5f3672821727fb8721` | v3 copy code TTL extend (D4-02) |
| 5,660,032 | 2026-11-10 15:16:43 | `bf600b2e1f2a0b018dc09b1fc9c394eff8573e2ea12bc2704cff66e0dead97e7` | `etesia-admin` Blend USDC trustline (D4-03) |
| 5,660,109 | 2026-11-10 15:23:08 | `69c2dec4ba5698ad29775f37438555a81230a2a89d17dbd78d41b7b5687bb20a` | Friendbot creates user browser wallet `GA6L…5CJP` |
| 5,660,113 | 2026-11-10 15:23:28 | `74dc88d692929ab99e0d4c38a5b752f296797f2cbd7b64246b1c8bf67865b722` | Friendbot creates user browser wallet `GA5A…2XO5` |
| 5,660,115 | 2026-11-10 15:23:38 | `8ec880a7834c22d6a019a5bcedccf86a47edf320089e55feb5e283f8f83eac6d` | Friendbot creates user browser wallet `GCF2…AVDW` |
| 5,660,354 | 2026-11-10 15:43:33 | `849624f021c574a5b13bd7e0c6fc932590b3c0461092e11a2eb4ded673c645fa` | Blend faucet claim, `GA6L…5CJP` |
| 5,660,370 | 2026-11-10 15:44:53 | `27355c4a16934f1d279aa9a22f39259e816be1c643f0132dc4fb364318082885` | Blend faucet claim, `GCF2…AVDW` |
| 5,660,383 | 2026-11-10 15:45:58 | `78f274a09ab652a607bd7d355ed89cda93480f8ecbf86002d8f347b383d00678` | Blend faucet claim, `GA5A…2XO5` |
| 5,660,415 | 2026-11-10 15:48:38 | `8d8d9f3cfde94463cda95c83436200ab90a5f71af57faa921d9e58b192fece3d` | 1,000 USDC `GA6L…5CJP` → `etesia-admin` |
| 5,660,421 | 2026-11-10 15:49:08 | `0bdd25d4f7edf262784f79622e87abe2e526e9421ab39f1e05744dbac8f589b7` | 1,000 USDC `GCF2…AVDW` → `etesia-admin` |
| 5,660,428 | 2026-11-10 15:49:43 | `9f75669bf8c5d9399e4394b09728c2f6f93d986053d2a6ee649ebe238b885fb2` | 1,000 USDC `GA5A…2XO5` → `etesia-realign` |
| 5,660,781 | 2026-11-10 16:19:08 | `586120118cfce2cf1b71bf8ecf76a62ba5b3ab4f7cf917ddd64fd6c6a8aeecbc` | Friendbot creates `etesia-executor` |
| 5,660,787 | 2026-11-10 16:19:38 | `ef878b2ae0fa3c08926944756182f10bffae59798c5c9c8571dbc12afb737705` | Friendbot creates `etesia-guardian` |
| 5,660,794 | 2026-11-10 16:20:13 | `0231d952b9b90f5cb774e0a711fe40fcc2b7e1f84df1376b015fa665fb760907` | Friendbot creates `etesia-guard-executor` |
| 5,660,800 | 2026-11-10 16:20:43 | `e76b6cd2b7b1a1c08b4f5a1ff9284a1a3ea0d1886827fd63103d16cc39f56f3d` | Friendbot creates `etesia-guard-guardian` |
| 5,660,805 | 2026-11-10 16:21:08 | `cfab3a70d2d69a2725cc3e440ecb38df2df16ec4076bbe2d2c4ce6a99310b4d8` | `etesia-v3-treasury` Blend USDC trustline |
| 5,660,821 | 2026-11-10 16:22:28 | `971d1f222412c796d83712b095710e761e08455120ac935312017630f757fba0` | tRESERVE SAC deploy `CCOF6WEY…Q5K7` |
| 5,660,833 | 2026-11-10 16:23:28 | `20b27ce3abfd1c82c5dd941a1a11f14c4666b2d392c2d9f1edb80ae439aebbbb` | tRESERVE/USDC pair `add_liquidity` (1,000 / 1,000) |
| 5,660,867 | 2026-11-10 16:26:18 | `59b16f2b01bc1c2b6fef55f68fbf5ec37edc433dd294a69048462827407bb5a7` | Pricing provider WASM upload |
| 5,660,871 | 2026-11-10 16:26:38 | `93e274eb52080f441ed043fa0cbbaeea6f324bc411680eedaa4dedeb442d386c` | Pricing provider deploy `CC3ASLC6…X5EB` |
| 5,660,901 | 2026-11-10 16:29:08 | `5e58fa051628d6ccdfc96b534afffad1668c4b91998d6ca59ccc56c42519c943` | Flagship WASM upload |
| 5,660,913 | 2026-11-10 16:30:08 | `8782f4f2eae4f93a2461a3c0cc201d358f8b012baf098eb2194893523f219a1c` | Guard vault deploy `CCUHQJ3F…C6VO` |
| 5,660,957 | 2026-11-10 16:33:48 | `189fa9767c08aab7960bb62340098152be415d603537f07d08d2944ef3127697` | XLM pair realign 1 (+534.9 bps → 0) |
| 5,661,064 | 2026-11-10 16:42:43 | `d1678c92a5d0e3b7053f9e1a4f6b709528670652e9d9554274e301565ed454c3` | XLM pair realign 2 (+58.0 bps → 0) |

## t2-d4-06-receipts.json

| Ledger | Close (UTC) | Hash | Label |
| --- | --- | --- | --- |
| 5,665,600 | 2026-11-10 23:00:43 | `879da75b55675abdfdd76cd709514a2df535a81a0d074b360a175d32aa503e11` | `publish_target`, deployed keeper cycle 1 (day decision target) |
| 5,665,788 | 2026-11-10 23:16:23 | `e0f5b9b536927982680b2ff763a417dd5a460ef69871fe1282bda79ca8b09712` | `publish_target`, deployed keeper cycle 2 |
| 5,665,790 | 2026-11-10 23:16:33 | `4ed46e779b20af87f72dceb77fb563f73e859996bb50c7d26791306da0fa1cd7` | `execute`: Blend Withdraw 0.0197 USDC, accrual trim by the deployed keeper (cycle 2) |

## t2-d4-08-receipts.json

| Ledger | Close (UTC) | Hash | Label |
| --- | --- | --- | --- |
| 5,661,252 | 2026-11-10 16:58:23 | `aa8f8e1938be1311db3fc3004f9c6ace82f745f42beb1edd4780a777a739b4c8` | `execute`: Swap 25 USDC→XLM through the v3 copy + Blend Supply 94.977 USDC (D4-03 rehearsal; D4-08 Supply; D4-10 step-1 Supply) |
| 5,665,790 | 2026-11-10 23:16:33 | `4ed46e779b20af87f72dceb77fb563f73e859996bb50c7d26791306da0fa1cd7` | `execute`: Blend Withdraw 0.0197 USDC, accrual trim by the deployed keeper (cycle 2) |

## t2-keeper-cycles-2026-11-10-to-11.json

| Ledger | Close (UTC) | Hash | Label |
| --- | --- | --- | --- |
| 5,665,972 | 2026-11-10 23:31:43 | `083dd7f2f21bad84d202ea220fdfaaac441a0400276ffa8729e1134d83c4a5e1` | Keeper `publish_target` |
| 5,665,974 | 2026-11-10 23:31:53 | `0f02123f07acd26bd29b732920702ff7d28be172839d337a2853b87c42af333d` | Keeper `execute`: Blend Supply |
| 5,666,156 | 2026-11-10 23:47:03 | `f26e0a18a6f8574e3e326e83bc61b644ca525069c916f135aced23a96af0ebd1` | Keeper `publish_target` |
| 5,666,158 | 2026-11-10 23:47:13 | `0d931430ebcf2460ede44f6fb8a516a70e83f2b01fbd1387edc45a56b799923c` | Keeper `execute`: Blend Supply |
| 5,666,340 | 2026-11-11 00:02:23 | `e02bd470c317f12bf7c2b41b5e93f901b3ba0037424a35c0ed954c66e620927d` | Keeper `publish_target` |
| 5,666,342 | 2026-11-11 00:02:33 | `e849f1132965914448f2aca0337c8c9389552b50b6ea9202ce9a3ecf3e6dbd94` | Keeper `execute`: Blend Supply |
| 5,666,560 | 2026-11-11 00:20:43 | `de0ca71ed5b1e60f91fc2d24ba1b7fc8ff1e4b9be45c40793f8cc0e6df3dd2c8` | Keeper `publish_target` |
| 5,666,562 | 2026-11-11 00:20:53 | `db75b2d4b184ca6c085f083cf5453974101c57dad164fc959e10cba1735b6890` | Keeper `execute`: Blend Supply |
| 5,666,744 | 2026-11-11 00:36:03 | `d40ba88bd087d2eb839889046b0242f65bbeac4cb97d25bd42061e9db7a0d1eb` | Keeper `publish_target` |
| 5,666,927 | 2026-11-11 00:51:18 | `690fc470c03fd86df1ed5bb9404e11e828c2d8fb2a783b93d5217a19b58a28ac` | Keeper `publish_target` |
| 5,666,933 | 2026-11-11 00:51:48 | `53fc3da4c9c4e5df4f557f861c321a710634491ff8a90dc95673d4399236d5d8` | Keeper `execute`: Blend Supply |
| 5,667,115 | 2026-11-11 01:06:58 | `b886381be5ea06a3fb728df461d40ce528492cbeef53608d4d114e0b31ab4f10` | Keeper `publish_target` |
| 5,667,117 | 2026-11-11 01:07:08 | `1e17053f67227a32d38a4e29ae30de7672bc0b791a453df147654c7553441622` | Keeper `execute`: Blend Withdraw |
| 5,667,299 | 2026-11-11 01:22:18 | `cb6782c640cb59080773752297a350ed058b982fdc8302e577ea4b39a4f887d0` | Keeper `publish_target` |
| 5,667,301 | 2026-11-11 01:22:28 | `95d019c9b4f795315635b565ed2aed26d470c4fdc8ccd00afd282ca20ad65051` | Keeper `execute`: Blend Withdraw |
| 5,667,520 | 2026-11-11 01:40:43 | `93a00c8edab30124983c7a5e4883fdbc08089f64b4b566388a1beee605d14772` | Keeper `publish_target` |
| 5,667,522 | 2026-11-11 01:40:53 | `8e5ca1f0ba4905ac9854fe2faa4f8fb75d8dbde1e8efafb8fac61d12ca77009b` | Keeper `execute`: Blend Supply |
| 5,667,704 | 2026-11-11 01:56:03 | `524ed6cfd2fb4978fbf7117a0d8a873a0b8929528fe4c285e13b08a317b37005` | Keeper `publish_target` |
| 5,667,706 | 2026-11-11 01:56:13 | `b8d4a18c01aa452d88c9d23af1629434d56f0da1dd82438d86c5587e528e1946` | Keeper `execute`: Blend Withdraw |
| 5,667,889 | 2026-11-11 02:11:28 | `2789778690806b6e74639b63af537e124d8d196b2a12d111d061ddf3993bdd7e` | Keeper `publish_target` |
| 5,667,891 | 2026-11-11 02:11:38 | `c05a389d2bd1d10e15c66a10929b1879a94fdd7d2c14acc1ab4f22820fc14202` | Keeper `execute`: Blend Withdraw |
| 5,668,074 | 2026-11-11 02:26:53 | `b134f476abafa105f46748122aff2f45df5d5856af17732515fabe8c0173d821` | Keeper `publish_target` |
| 5,668,075 | 2026-11-11 02:26:58 | `07397a1bc7a488d2a456b1303448de38eea97e981439e96c9fe4b69187a2ab88` | Keeper `execute`: Blend Withdraw |
| 5,668,257 | 2026-11-11 02:42:08 | `da5c91e8ed96e5e133460aa4466c0eb351a83a29b600e9fcd36180b49c99e7e1` | Keeper `publish_target` |
| 5,668,259 | 2026-11-11 02:42:18 | `b9c8959e7d124181180a6e20a6bf186066871ce7574d6f210bec8076957ea050` | Keeper `execute`: Blend Supply |
| 5,668,441 | 2026-11-11 02:57:28 | `6fe4556e70f29afdc44e46f47cf4d6631daefcd22412214f5357293a739afd2e` | Keeper `publish_target` |
| 5,668,443 | 2026-11-11 02:57:38 | `72527dd1c0ec30da5fdf9ea7cf3bd2b2524ab3dea000c937fff3a91f23e00e90` | Keeper `execute`: Blend Supply |
| 5,668,660 | 2026-11-11 03:15:43 | `d552ada0d4f0abdec35cf04f0929f568afc7daf3389ea8bd73f69004b2281ff9` | Keeper `publish_target` |
| 5,668,662 | 2026-11-11 03:15:53 | `d11d5301c548cce7c53f596b47a417b9d2cc0e43c69f2259142f1de44f929837` | Keeper `execute`: Blend Supply |
| 5,668,844 | 2026-11-11 03:31:03 | `3bdc878171df31adbd6700436e8e02a82ff93b63e51e404a80cf4cc736988f41` | Keeper `publish_target` |
| 5,668,846 | 2026-11-11 03:31:13 | `4ece8564959d6969d39dbcb8bf0b89fba732735cbbe93b8bdd3a75d30eab95ed` | Keeper `execute`: Blend Withdraw |
| 5,669,028 | 2026-11-11 03:46:23 | `c88d3ea33a14297a8dfa1650f504241f13787a84501b6820c9085c51a760878c` | Keeper `publish_target` |
| 5,669,030 | 2026-11-11 03:46:33 | `d73e3a094e4691b3cfa1755383caad7e355b7e5b373651c89c94a4d8faee983a` | Keeper `execute`: Blend Supply |
| 5,669,212 | 2026-11-11 04:01:43 | `ed2f07b35462db779ed642c63d9a0ebc6d386ad4974c8b1be171c3fe6c9eb072` | Keeper `publish_target` |
| 5,669,214 | 2026-11-11 04:01:53 | `3c99dda0533676459c615f65d1111e913f54417ca32932e8efcae5c1eac5d034` | Keeper `execute`: Blend Withdraw |
| 5,669,396 | 2026-11-11 04:17:03 | `93b1236bfe6efef548fc73153b477e60d0d0bb08258bd0a02cfc9c29ab6c25bf` | Keeper `publish_target` |
| 5,669,578 | 2026-11-11 04:32:13 | `219839fb4268b21e852437393a69da17a234216c74e5470972cc52b7ff5262ba` | Keeper `publish_target` |
| 5,669,580 | 2026-11-11 04:32:23 | `8a3b07a2361bf03bc38187fa47450496cc4c7a81bf1930566bc02bdaf86253f4` | Keeper `execute`: Blend Withdraw |
| 5,669,762 | 2026-11-11 04:47:33 | `4d809b7b7ea972f06e78c658525724129904d595c5acf83da9eb5ef275a21be3` | Keeper `publish_target` |
| 5,669,979 | 2026-11-11 05:05:38 | `fb14101f87d4d002a6a4e7ee0276547ad19e3f8525da3e28e5b0f3b06f2c0ace` | Keeper `publish_target` |
| 5,669,981 | 2026-11-11 05:05:48 | `a30d23c2eb0cc29a1fef27b08e32361db277b1600a2f76c90937e79a58f67227` | Keeper `execute`: Blend Supply |
| 5,670,164 | 2026-11-11 05:21:03 | `4f5616acbe430698fd20e0defd99d2fa763e599d60061d477ea945c2165060e6` | Keeper `publish_target` |
| 5,670,166 | 2026-11-11 05:21:13 | `e2865c6c1801d2a44bea835991b249dd7d29b057d2da0312e2e7ca69cc75f1fa` | Keeper `execute`: Blend Supply |
| 5,670,349 | 2026-11-11 05:36:28 | `912a06af01c3817cc7951dce1449da167b3d72953ad7f4166a4f4b3bf9d6fdd5` | Keeper `publish_target` |
| 5,670,351 | 2026-11-11 05:36:38 | `2c81f8644485f0b37ef55809fbf255b396bcff69853517c4cfd8ba075b93f919` | Keeper `execute`: Blend Supply |
| 5,670,533 | 2026-11-11 05:51:48 | `596f9bdfd7e703ec4eb26eff59375394403eeb8dbdfda3c5802f17c8041caacc` | Keeper `publish_target` |
| 5,670,535 | 2026-11-11 05:51:58 | `431e069564370c7ee7546847c582f908eff9314d080bc1587cb1c9a50592415b` | Keeper `execute`: Blend Supply |
| 5,670,717 | 2026-11-11 06:07:08 | `dd9ded939e20634f6c02112d280f8ae8c36fd04caa3f3646bd274be1b4acdd61` | Keeper `publish_target` |
| 5,670,719 | 2026-11-11 06:07:18 | `58acf9008dc2850ff9ccc97ab15cba641a4f5d5ab6bfb043f8d29f964e69b243` | Keeper `execute`: Blend Supply |
| 5,670,902 | 2026-11-11 06:22:33 | `84adc06557a4e631bd94846c66f63128436d468f5f2c09db8300575e631bceaa` | Keeper `publish_target` |
| 5,670,903 | 2026-11-11 06:22:38 | `ec2b82619544ff42f1fb1fb46567b1513c5fca073e94b4443a09ddf17b869592` | Keeper `execute`: Blend Withdraw |
| 5,671,120 | 2026-11-11 06:40:43 | `cd04cb6d0b6c86a49e159645677ee9a817e06f4441c6d45635d0caaf94fe5101` | Keeper `publish_target` |
| 5,671,122 | 2026-11-11 06:40:53 | `169473292490ce0761d33219a358c9bf4cd2ccac8e295f33deda76bb87da7692` | Keeper `execute`: Blend Supply |
| 5,671,304 | 2026-11-11 06:56:03 | `db60cc3f338aa80238098b62f545aa2faa6e3f63747e0554e572dfbdf32b7958` | Keeper `publish_target` |
| 5,671,306 | 2026-11-11 06:56:13 | `c03a1adc2f5e23f036c4cf0652ba365a695106775c85cfa892ee020b8703dffe` | Keeper `execute`: Blend Withdraw |
| 5,671,488 | 2026-11-11 07:11:23 | `1216c5e7f5be0a768eebfb62cf16d63234b9db96f6f72650d33ac354b99e1c1e` | Keeper `publish_target` |
| 5,671,490 | 2026-11-11 07:11:33 | `7f185b8bc59ed1f7a3ef90915b25ba4f96aebae005d0b9bd2d7937ece81f49a4` | Keeper `execute`: Blend Withdraw |
| 5,671,672 | 2026-11-11 07:26:43 | `ecf97462d9a9e447788c2e3cb9a5bb2a269f64abbf731a8fd2224f9f0480587f` | Keeper `publish_target` |
| 5,671,674 | 2026-11-11 07:26:53 | `9729bfcb08decf0b209dc9f6adc821ac11754b41bac259c5bd7c7ef7298f0f66` | Keeper `execute`: Blend Supply |
| 5,671,857 | 2026-11-11 07:42:08 | `4bda3946b243a5ba0dbfbdad4d9ad7338f20327df65d518706f74aa103b4de61` | Keeper `publish_target` |
| 5,671,859 | 2026-11-11 07:42:18 | `4f4a87d3b134e4269bc7b092f6d40e07b7fc8ded3857bae7ce9c8a0208ae94a8` | Keeper `execute`: Blend Supply |
| 5,672,042 | 2026-11-11 07:57:33 | `28c0f1a3e27c7d12c84126e00ff59898b20d5c00d995f980c8d370e68d5964d6` | Keeper `publish_target` |
| 5,672,260 | 2026-11-11 08:15:43 | `0aa4a86d9289eb3851331046ab5fc2d67b0293cd8ff5765765507cb037c2c929` | Keeper `publish_target` |
| 5,672,262 | 2026-11-11 08:15:53 | `01a62d85992322ce3c0e1d94a401574f649061e4f51654172ba0f83a0944ce85` | Keeper `execute`: Blend Supply |
| 5,672,444 | 2026-11-11 08:31:03 | `6f96b0ac836e491dd4c32abf9cfedd515bb0245bcad17be484981277eed2392f` | Keeper `publish_target` |
| 5,672,449 | 2026-11-11 08:31:28 | `703bae64bf0a33a1c684df2584d89ad638a454a3427cd10799c153743c5b0b46` | Keeper `execute`: Blend Supply |
| 5,672,631 | 2026-11-11 08:46:38 | `97fce7782f0b6b7882148ababd05871e624ec4b9971fe5c8fa2611f1703d0e0e` | Keeper `publish_target` |
| 5,672,633 | 2026-11-11 08:46:48 | `7b86eb4d4d1b02537da80f9bc5f7694e007ef30ef3006c4f6644de8983694ae1` | Keeper `execute`: Blend Supply |
| 5,672,815 | 2026-11-11 09:01:58 | `f24a8cb3f0c5030b0cafcc66bb9ec2efc8949e5d15396764bc940c708c1d7940` | Keeper `publish_target` |
| 5,672,998 | 2026-11-11 09:17:13 | `f6d20453fc7e620f7c8e26a41cf1ddca9a7c5788dde76ddd80ef8593b9117072` | Keeper `publish_target` |
| 5,673,001 | 2026-11-11 09:17:28 | `e8a3db1789e2e75f00fd0e3fe2ffcdb9061650d368113acb0fe14e7effda5381` | Keeper `execute`: Blend Supply |
| 5,673,220 | 2026-11-11 09:35:43 | `52400ca5efde19f2588ede11d3f49883bbd7e50c431ebb3dc59514797fef14a2` | Keeper `publish_target` |
| 5,673,222 | 2026-11-11 09:35:53 | `bcb7142b1808aff99a0a821c8460997961aad9681a5f9d6e0a838b954d4773af` | Keeper `execute`: Blend Withdraw |
| 5,673,405 | 2026-11-11 09:51:08 | `3be3bd398687066dccedaca7645983b47ed71f299db8ed25d9b3d32ae5ed1be6` | Keeper `publish_target` |
| 5,673,589 | 2026-11-11 10:06:28 | `03f7aa25aea0c30eec1c9bbffcef16d6d470da09f05293721bd5692b3e9053c8` | Keeper `publish_target` |
| 5,673,591 | 2026-11-11 10:06:38 | `98e5e1d53708c9302715113beb065b917b29dce77871f675ed95405659d6e411` | Keeper `execute`: Blend Supply |
| 5,673,773 | 2026-11-11 10:21:48 | `38bcdb92219462b50d82373e94d31aeeb5ad557a143c56c68cd0c0e252ad6d78` | Keeper `publish_target` |
| 5,673,956 | 2026-11-11 10:37:03 | `8d4c94917bdbe1cfdce0fac75cf0ada19ffb43a6c6fec4873203035f76eb5f22` | Keeper `publish_target` |
| 5,673,958 | 2026-11-11 10:37:13 | `16bea64e70886447b8503f66bda839cd7cd381bbde17119079f37fa8cfa54981` | Keeper `execute`: Blend Supply |
| 5,674,140 | 2026-11-11 10:52:23 | `17db651cd97c062ad0bbbefa42b02e3c708489c80d29bb3d04a90b9be6ea4da1` | Keeper `publish_target` |
| 5,674,143 | 2026-11-11 10:52:38 | `49b53e6530cf99889a505be608652d77c60a2bbbe59e4f7bd6f24975ef1a2052` | Keeper `execute`: Blend Supply |
| 5,674,361 | 2026-11-11 11:10:48 | `e7d35097f4badb52581f81900e4384e1b84c044cad01100029b5e3a21dbfbb65` | Keeper `publish_target` |
| 5,674,364 | 2026-11-11 11:11:03 | `a2eaff5f79390c12064a22e63e49db9e06c2098fc7c210cf71bb45bac9bd1a2e` | Keeper `execute`: Blend Withdraw |
| 5,674,547 | 2026-11-11 11:26:18 | `c7fe8a31f493b55c6e78181ff4b98196634fd945d6fe0769ed089129eb8c93ee` | Keeper `publish_target` |
| 5,674,549 | 2026-11-11 11:26:28 | `1683cfa91372485ff22f2359d30a46713fe35a5d10a8782289e02299c1000ed1` | Keeper `execute`: Blend Supply |
| 5,674,731 | 2026-11-11 11:41:38 | `0e4177780c49a9899c53df81947138da83706b215e9e6de8e4a46a413593bf59` | Keeper `publish_target` |
| 5,674,733 | 2026-11-11 11:41:48 | `683e3db21d6e8d9a8f224638b19738fb40041e4e81b3a8752e5656c899b1bf68` | Keeper `execute`: Blend Withdraw |
| 5,674,915 | 2026-11-11 11:56:58 | `6c3b09e03dc74236d284a5688b97419f59f22c6a5d86a44b62f7bb043db32749` | Keeper `publish_target` |
| 5,674,917 | 2026-11-11 11:57:08 | `7dc1dfbe48106bdb5f0e70a7fa1c5545df2b02735e67d56fca7050f4272c38ba` | Keeper `execute`: Blend Supply |
| 5,675,099 | 2026-11-11 12:12:18 | `c1087a3dfcbc79cceb2d3fe9e6bce5b7a36f65fb311b67bda4633e6d4738d265` | Keeper `publish_target` |
| 5,675,102 | 2026-11-11 12:12:33 | `bfed6000b6bbe217d8b3955601af7f4d3da75f19b38d388d95c88bb7c5b9ab05` | Keeper `execute`: Blend Withdraw |
| 5,675,318 | 2026-11-11 12:30:33 | `acad45fdf2ab68a0f49c2a95303ebc3e85773b994365721596df206c886c9ea3` | Keeper `publish_target` |
| 5,675,320 | 2026-11-11 12:30:43 | `604b6f32226793fb996c7ae569d2f0a020513e5fa98eca17a96228f6b58864da` | Keeper `execute`: Blend Withdraw |
| 5,675,503 | 2026-11-11 12:45:58 | `de39d23717dd807e084a86021e4152e94b89218f91a63eb35b31b0a04452431e` | Keeper `publish_target` |
| 5,675,506 | 2026-11-11 12:46:13 | `31d079505a129765e14546bade60338f45cdf3ea7b729a861ed52eecb29144c1` | Keeper `execute`: Blend Supply |
| 5,675,689 | 2026-11-11 13:01:28 | `26ec02b905398e7dc129853627f97b588da906d193f113bf9f24d1a49606d216` | Keeper `publish_target` |
| 5,675,691 | 2026-11-11 13:01:38 | `3ec9caeb31816521c728dd092797b49941919499fa65339d4ede348fb881ef81` | Keeper `execute`: Blend Supply |

## t2-a77-resume-announce.json

| Ledger | Close (UTC) | Hash | Label |
| --- | --- | --- | --- |
| 5,675,326 | 2026-11-11 12:31:13 | `801123cb6a0c4d3cb9e80d211f0ebe4561d7b366c5cec27b583a23aea6f685ad` | Guard `announce(Resume)` from `etesia-admin` ([A77](../../../doc/wiki/decisions.md)); pending hash `987835d9…6792` |

## t2-d4-10-step1.json

| Ledger | Close (UTC) | Hash | Label |
| --- | --- | --- | --- |
| 5,675,700 | 2026-11-11 13:02:23 | `17cba5db39b69e1e1f3575af0016453fdf506d4c2016bb2e9c241ee97520f46a` | tRESERVE pair realign (+99.3 bps → 0) before the `flat` scenario |
| 5,675,860 | 2026-11-11 13:15:43 | `391912665a4264162548c2e5166f0d717662dedd652a44e7ce5e923b8b6641ae` | Keeper `publish_target` |
| 5,676,044 | 2026-11-11 13:31:03 | `af9e60ed48605d22e50c8c485408784ff369b54c586c71e8d4b0d7b01806394c` | Keeper `publish_target` |
| 5,676,228 | 2026-11-11 13:46:23 | `5d81478a90f756ce2c3b0d90969b242edeb9ddf3dfc4b3135485236a58401d2b` | Keeper `publish_target` |
| 5,676,411 | 2026-11-11 14:01:38 | `0f51abb643a2d50caba041a6d627476c542e1ce608271515fc3b7734230ad232` | Keeper `publish_target` |
| 5,676,594 | 2026-11-11 14:16:53 | `c799d8a0779ccd8d7e47590b53037dd4e1f6f47993490b347d4f2bf233915c2c` | Keeper `publish_target` |
| 5,676,778 | 2026-11-11 14:32:13 | `695139c58a92649596d3868fa16dfcb98b1011688a9b1a996e25bde07a43d6d8` | Keeper `publish_target` |
| 5,676,962 | 2026-11-11 14:47:33 | `eb7d7a5fd87e2b1a590aa4889d66f56bf88c8595555928f0dd9cb2dcd6f538d0` | Keeper `publish_target` |
| 5,677,180 | 2026-11-11 15:05:43 | `dcf0d35fc4a4074a564ad399734462567161d9cd14c9d1dfc7277b48f26d2242` | Keeper `publish_target` |
| 5,677,196 | 2026-11-11 15:07:03 | `d1f7ebbbc2f86451ffc5b276eb05a076bc4df41a2722120810d7a73e807f2c17` | Keeper `publish_target` |
| 5,677,199 | 2026-11-11 15:07:18 | `3b03a8eac8fe824c2bba7476f1ec0ed43b8a6e93d121ed9db25a67d3226f3de5` | `execute`: Swap, first A80 flat plan (cycle 63) |
| 5,677,210 | 2026-11-11 15:08:13 | `b4791c954f76a79d3295c9cc076f49059f3a13b25db62ce61c0fa06f9c337886` | tRESERVE pair realign after the first flat plan |
| 5,677,382 | 2026-11-11 15:22:33 | `e21aee230ec0232dede30a1b7727b1064df64742ee10e78b6475ab53134993b9` | Keeper `publish_target` |
| 5,677,384 | 2026-11-11 15:22:43 | `4a5e3b219f3308d119d3eae1f1a6e21d85f29c00e3e6ba922d759313f2787d66` | `execute`: Swap, second A80 flat plan (cycle 64) |
| 5,677,395 | 2026-11-11 15:23:38 | `3f2dca683f50aabaae2dcbd9c71492a060402014abeddbfffd5523305368c1a0` | tRESERVE pair realign after the second flat plan |
| 5,677,420 | 2026-11-11 15:25:43 | `6cfac5a8ca7c9018d5695c6d0cb63a8b3d67e5a013fe4fc72cf333b92b22be4c` | Keeper `publish_target` |
| 5,677,422 | 2026-11-11 15:25:53 | `db9b81f0abaa925d21fe421a7f017b132f4ad7c4c8e86165cd8178b9d5a91211` | `execute`: Swap, third A80 flat plan |

## t2-d4-10-step2.json

| Ledger | Close (UTC) | Hash | Label |
| --- | --- | --- | --- |
| 5,677,455 | 2026-11-11 15:28:38 | `7817685aea3accdd597b403f92a040494d57190a330b2edb4a401b1ddad543af` | XLM pair realign (236.21 USDC → 1,611.67 XLM) |
| 5,677,480 | 2026-11-11 15:30:43 | `65a1d8893268d50618b38dc723a18a5ab9c765b19c702a104b7fe1f2fdd69cde` | Keeper `publish_target` |
| 5,677,483 | 2026-11-11 15:30:58 | `8e6056c7beb50e0659d9b04f71a2d8d3e4562ae5d291fc0c3fbdce6cbc255c7d` | `execute`: Blend Withdraw, D4-10 step 2 |

## t2-d4-10-step2b.json

| Ledger | Close (UTC) | Hash | Label |
| --- | --- | --- | --- |
| 5,677,637 | 2026-11-11 15:43:48 | `79149abf4334d4a00d133409f8ec524a92a314ca912222294afb759d81f195b3` | XLM pair realign (−102.0 bps → 0; 47.83 USDC → 316.45 XLM) |
| 5,677,665 | 2026-11-11 15:46:08 | `fbea98ae6856838d359b90bdaacc6f7e7c5542561a5ef8797ca45637aaa69ad7` | Keeper `publish_target` |
| 5,677,668 | 2026-11-11 15:46:23 | `9ca779f1ea97dd6f0bbee629288a32f606eda76a1711c938d297a905ef893812` | `execute`: Swap 17.21 USDC→112.93 XLM through the v3 copy, D4-10 step 2 |
| 5,677,678 | 2026-11-11 15:47:13 | `b64c6e66fc073b5af12a0f7b3a857f64ed83a69622942477569cf01ba26458c3` | XLM pair realign (+68.0 bps → 0; 209.19 XLM → 31.60 USDC) |

## t2-faucet-a81.json

| Ledger | Close (UTC) | Hash | Label |
| --- | --- | --- | --- |
| 5,677,274 | 2026-11-11 15:13:33 | `46cd9c2fcfd7f9327d4b0c38f5c2aae1ad10737245823ff80b251102e8da2439` | Friendbot creates faucet key `GDJKKD…GLGS` |
| 5,677,283 | 2026-11-11 15:14:18 | `50d3d8b38ebfc329257b2f24248924b070cc4428becd5498264a48fd892eedfe` | Blend faucet claim, `GDJKKD…GLGS` |
| 5,677,285 | 2026-11-11 15:14:28 | `4acbf6862be6cdd9cac40f76d98faaddcf623e459bb62801eb110a84ede8bf87` | 1,000 USDC `GDJKKD…GLGS` → `etesia-realign` |
| 5,677,289 | 2026-11-11 15:14:48 | `1598f346fd02d7005323fc4046dbbb443360f5cd3834e9d9419a44f82083d5ca` | Friendbot creates faucet key `GCB3XC…BCV6` |
| 5,677,292 | 2026-11-11 15:15:03 | `e2e3e7814145e929dd797f28ebfb1f258c3668d8427896ee629ae222f9bbce9d` | Blend faucet claim, `GCB3XC…BCV6` |
| 5,677,294 | 2026-11-11 15:15:13 | `7d9177f05ed576d8d3376df83412e72005fe0e1950328df021ce9bcd24416c17` | 1,000 USDC `GCB3XC…BCV6` → `etesia-realign` |
| 5,677,295 | 2026-11-11 15:15:18 | `a0376dfe60b232dfb937d1a2180f8baa549d4bb0dec64348137c2e5caff8fe78` | Friendbot creates faucet key `GBPUJJ…EQIS` |
| 5,677,297 | 2026-11-11 15:15:28 | `7e2d1970c61795239c5901d04a79420e46c12f41600c8c4f6ac9f62c068ef332` | Blend faucet claim, `GBPUJJ…EQIS` |
| 5,677,299 | 2026-11-11 15:15:38 | `8d4663df2a82fabc4f1685688332bfd6a6fa06f3bac9565445cbee48d60a0b2c` | 1,000 USDC `GBPUJJ…EQIS` → `etesia-admin` |

## t2-d4-10-step2c-drill.json

D4-10 step 2 tail and the kill/restart drill (2026-11-11 15:44–16:11 UTC, ledgers 5,677,637–5,677,962): XLM realigns `79149abf…95b3` and `b64c6e66…58c3`; keeper publishes `11726d45…f218` (cycle 68, fault armed, skipped), `51c5cc00…95e1` (cycle 69, skipped), `240ee2f0…5565` (cycle 70); the dropped-response execute `312ead0e…8a19` (nonce 53, resolved as included in cycle 71); the restart's publish `dbb68ff3…f661` and execute `ff7bbe22…e44b` (nonce 54). Pre-reset deployment per A76; archived XDR is the record.

## t2-d5-user-deposit-1.json

First user wallet transaction on the hosted testnet vault page (`/stellar_tf_vault/testnet`, builder `0e49ac1`): `deposit_bounded` `633e4a6f…0406`, ledger 5,679,344 (2026-11-11 18:06:03 UTC), signed by the user's browser wallet `GA6LXX26…5CJP`: 10 USDC (100,000,000 base units) for 9.99155693881 shares, with the deposit-time fee-share mint of 65,016,424 to `etesia-admin`. Pre-reset deployment per A76.

## t2-d5-allocation-run-2026-11-12.json

First Builder allocation executed on testnet (hosted `/?network=testnet`, builder `d180e6b`, swaps quoted and built by the Soroswap Aggregator API), signed by the user's browser wallet `GA6LXX26…5CJP` (xBull, user-reported) and archived on 2026-11-12 at 08:53 UTC. Basket XLM + ETESIA-TF with the USTRY reserve, 100 USDC, 25% volatility budget: the gross allocation cap bound, so the reserve was 0 and the run had no tRESERVE trustline or swap leg. Pre-reset deployment per A76.

| Ledger | Close (UTC) | Hash | Label |
| --- | --- | --- | --- |
| 5,689,836 | 2026-11-12 08:40:23 | `bea80542f4db5237254614ba19a73b538d08ac358f19140d35a59e38776ae736` | Router `swap_exact_tokens_for_tokens` 15.6470783 USDC → 115.2191255 XLM (minimum 114.6430302) |
| 5,689,838 | 2026-11-12 08:40:33 | `e48eeda45a153b475f15f95c7d8787928e7aaa22dc43b6fc8ffd76cea43ee016` | Flagship `deposit_bounded` 81.851665 USDC → 81.331753576578 shares (minimum 80.927716236381) |

## t2-d5-rebalance-run-2026-11-12.json

First Builder rebalance executed on testnet (hosted `/?network=testnet`, builder `d25fe1f`, swaps quoted and built by the Soroswap Aggregator API), signed by the user's browser wallet `GA6LXX26…5CJP` (Varial, user-reported; screenshots) and archived on 2026-11-12 at 20:22 UTC. The run nets the account's holdings against the targets (A97): vault shares over target were withdrawn, and the USDC bought the XLM and tRESERVE under target. It is the first run with the tRESERVE trustline and swap legs. No liquidity top-up preceded it (no `etesia-liquidity` transaction after ledger 5,694,451, Horizon read 20:25 UTC). The share balance after the run is 70.898579957832 (`balance` read by simulation, 20:24 UTC). Labels are decoded from the archived envelopes and the return values in the metas. Pre-reset deployment per A76.

| Ledger | Close (UTC) | Hash | Label |
| --- | --- | --- | --- |
| 5,698,225 | 2026-11-12 20:19:29 | `c20b6c0bb3a2008d50882163b85959744f891a91f3b54a32200f50b6da159c3d` | `changeTrust` tRESERVE (issuer `etesia-realign` `GDRVYVG3…ZM3R`) |
| 5,698,226 | 2026-11-12 20:19:34 | `9af5108b89eead7a72f0b534da4795c91c334e8801745480bc4915f3beeb3d2a` | Flagship `withdraw_bounded` 10.5203409 USDC for 10.433173618746 shares (maximum 10.485339486549) |
| 5,698,228 | 2026-11-12 20:19:44 | `58fe6e58e70fde1b2bab2a6051fc338d16f9e89c3ea8ffba92647403f58345b8` | Router `swap_exact_tokens_for_tokens` 1.9870186 USDC → 18.6911632 XLM (minimum 18.5977077) |
| 5,698,229 | 2026-11-12 20:19:49 | `9eb66ca42951eba4789998b8b38b6079ec1a44a6fcd589193876631a8f16dce1` | Router `swap_exact_tokens_for_tokens` 8.6228498 USDC → 8.5050739 tRESERVE (minimum 8.4625486) |

## t2-keeper-cycles-2026-11-11-evening.json

Cycle numbers are derived, not read from the journal: from cycle 71 (`ff7bbe22…e44b`, 16:10:53) at the keeper's cadence of about 15 minutes, with the 2026-11-11 log anchors cycle 78 (before the 18:03 overrides deploy), cycle 80 (before the 18:21 unset deploy) and nonce 64 in cycle 84 (journal read). Cycles 79 and 81 sent no transaction. That they were the boot cycles of those two deploys is an inference; why they sent nothing is not read from the journal. Execute nonces are from the plan argument and the vault's `plan` event; actions from the meta's contract events. Whether the user wallet transactions count toward D5-07 is for the user to confirm. Pre-reset deployment per A76.

| Ledger | Close (UTC) | Hash | Label |
| --- | --- | --- | --- |
| 5,678,145 | 2026-11-11 16:26:08 | `95f32411c35ef5019f5cdbdc7745040c59bd21999cd84a2fd790dc7421872407` | Keeper `publish_target` (cycle 72) |
| 5,678,334 | 2026-11-11 16:41:53 | `5701882b901f4fb50a48e81c8d9d7c08b989cae486fc52903abcde610a53476a` | Keeper `publish_target` (cycle 73) |
| 5,678,337 | 2026-11-11 16:42:08 | `7481920e66105f3e07cb5e1cdd14aa5ea564e4c6781056e43a32ec9d4c9a3ff7` | Keeper `execute`: Blend Withdraw 0.0265 USDC (cycle 73, nonce 55) |
| 5,678,519 | 2026-11-11 16:57:18 | `7d5a5250693111082bdde9a645c9d23996a23720c1f4be2fc545d2de843fc9dc` | Keeper `publish_target` (cycle 74) |
| 5,678,521 | 2026-11-11 16:57:28 | `8bfc86fc6627e55b0daa2590b5a10035196f31913cf54a2aaa782377777c3765` | Keeper `execute`: Blend Supply 0.0157 USDC (cycle 74, nonce 56) |
| 5,678,740 | 2026-11-11 17:15:43 | `d4cc8bb535fa5413535731c716e9e92c4d06f470cd8ab1829dcf21b739c55472` | Keeper `publish_target` (cycle 75) |
| 5,678,743 | 2026-11-11 17:15:58 | `4713f33ee2c5dc7fb7dfb2c256e6e7555330d301c68c9a71cb19377efd890423` | Keeper `execute`: Blend Withdraw 0.0133 USDC (cycle 75, nonce 57) |
| 5,678,926 | 2026-11-11 17:31:13 | `e097c1f551060447423377bf54efc5150d2b43690289b82b6d8a206bdca9233a` | Keeper `publish_target` (cycle 76) |
| 5,678,929 | 2026-11-11 17:31:28 | `7a6418ce64d93ec2998622dd572ba672e83d6d8e0b573da8a5b5e0bd253371c5` | Keeper `execute`: Blend Supply 0.0106 USDC (cycle 76, nonce 58) |
| 5,678,987 | 2026-11-11 17:36:18 | `cc2f4ad935d56f15a60f41b253f462382166377b59e81d621d3ad1526d766949` | XLM pair realign (−25.5 bps → 0; 11.91 USDC → 78.76 XLM) before the D4-09 Turnover and Minimum-below-floor cases; logged without XDR in `d4-realign-pre-2026-12-16-reset.jsonl` |
| 5,679,112 | 2026-11-11 17:46:43 | `0714a9583a1c96d17adf0c867eb73101a59cfc163acf966ccf195c7ea214ddb3` | Keeper `publish_target` (cycle 77) |
| 5,679,114 | 2026-11-11 17:46:53 | `7939c7d606381c4e6eb9073a6e0c4e4d1c0df0b0e7a9fb0e2414e55ed7805b1f` | Keeper `execute`: Blend Withdraw 0.0036 USDC (cycle 77, nonce 59) |
| 5,679,297 | 2026-11-11 18:02:08 | `ee3dd2046c58c927114f8d138e3695b4b67f778adeac445d8b1186e046d31255` | Keeper `publish_target` (cycle 78) |
| 5,679,303 | 2026-11-11 18:02:38 | `19b2c486142c7f95a49985f4c3b3a905b715f90997bdfdad9ef1252e5d88242a` | Keeper `execute`: Blend Withdraw 0.0103 USDC (cycle 78, nonce 60) |
| 5,679,336 | 2026-11-11 18:05:23 | `05bca65c3e5a098ffb9243a0958ef6dbd212e6dcf80323242ccd0a94dd6fd5ab` | 100 USDC `etesia-realign` → user browser wallet `GA6LXX26…5CJP` |
| 5,679,521 | 2026-11-11 18:20:48 | `d978f934edbff43c8d440d1b157bc5f3d2a2ce89e23a3167a612629309719219` | Keeper `publish_target` (cycle 80) |
| 5,679,524 | 2026-11-11 18:21:03 | `8007d56ff376ce97ad70fe5540166683b4559213614b2e7d9a802a91e10fe4d7` | Keeper `execute`: Swap 12.32 USDC→81.21 XLM through the v3 copy (cycle 80, nonce 61) |
| 5,679,525 | 2026-11-11 18:21:08 | `1962623e2cdd0d72a8a8d57aea2c9429dc7dd144bafeb9db9367158a17bc04b3` | `withdraw_bounded` 5 USDC (4.996422328101 shares burned), user browser wallet `GA6LXX26…5CJP` |
| 5,679,535 | 2026-11-11 18:21:58 | `3bfee4b99b08b2b5a237ea8a09afd4e6679338bceb23ff277665351a64c84e5a` | `redeem_bounded` 4.995134610709 shares for 4.9987113 USDC, user browser wallet `GA6LXX26…5CJP`; with the withdraw, the burns equal the 9.99155693881 shares minted by `633e4a6f…0406` |
| 5,679,716 | 2026-11-11 18:37:03 | `98c5b12af962949227a274030ecc64aa5daf7759e8f126beb60cf4b28902da65` | Keeper `publish_target` (cycle 82) |
| 5,679,718 | 2026-11-11 18:37:13 | `7d546ab1ee7dad8a4c1fe2a32350ebb0c82a4c281c697815130c2b6dccea8993` | Keeper `execute`: Blend Withdraw 0.0354 USDC (cycle 82, nonce 62) |
| 5,679,901 | 2026-11-11 18:52:28 | `c6d862fda202ad6c2685deda43cbb73bbd7c07f601b5e12e577722f004494ced` | Keeper `publish_target` (cycle 83) |
| 5,679,903 | 2026-11-11 18:52:38 | `2398d3b3fcd9e5c75b6ca9c6e62dd2564d7cc069325d606913cedf18bfe756cd` | Keeper `execute`: Blend Supply 0.0045 USDC (cycle 83, nonce 63) |
| 5,680,120 | 2026-11-11 19:10:43 | `89a1630a90b80609a16bf5f7cc33c38f4ae448e8e26b6de00b42ab2accd0b173` | Keeper `publish_target` (cycle 84) |
| 5,680,122 | 2026-11-11 19:10:53 | `56781591c87336ac225d03e6ff9453548327d8a436a8e33859aeeeb1e69ae9a1` | Keeper `execute`: Blend Withdraw 0.0291 USDC (cycle 84, nonce 64) |
| 5,680,305 | 2026-11-11 19:26:08 | `ee19cf153196907b74d4e5e2a3248ea1e0db3c62dc6c0680004ed8fe824fc7a4` | Keeper `publish_target` (cycle 85) |
| 5,680,308 | 2026-11-11 19:26:23 | `7662521aa755e97dc903f1a95b3b3dbb3c85180b4f6ad05163c23d83f169ddf6` | Keeper `execute`: Blend Withdraw 0.0046 USDC (cycle 85, nonce 65) |
| 5,680,490 | 2026-11-11 19:41:33 | `2be172e0ba3e02e9d265301b36ba07bc394ababa13f9d08b8733667651d47017` | Keeper `publish_target` (cycle 86) |
| 5,680,492 | 2026-11-11 19:41:43 | `757e2f821296bafe63964cff6f66c19f2984b71b1462298f1b828a4a6e6783e3` | Keeper `execute`: Blend Supply 0.0071 USDC (cycle 86, nonce 66) |
| 5,680,675 | 2026-11-11 19:56:58 | `c64d86f4dff0e8a3da454a9858ef8e6c0e215e087ad2ed8857768b8a078fac7a` | Keeper `publish_target` (cycle 87) |
| 5,680,678 | 2026-11-11 19:57:13 | `f37c998f75051c06377b831c2339b7615d7d0a363fdc77e82b37bce7120dbbef` | Keeper `execute`: Blend Withdraw 0.0033 USDC (cycle 87, nonce 67) |

## t2-keeper-cycles-2026-11-12.json

Every tracked-account transaction after ledger 5,680,691 up to the Horizon read of 2026-11-12 14:15 UTC (latest ledger 5,693,851) not archived elsewhere: all 133 are flagship-executor `GDGCL46K…JT7N` calls to the flagship `CD73M44E…XFQH`, 69 `publish_target` and 64 `execute` (nonces 68–131, contiguous). The Horizon read found no transaction after ledger 5,680,691 for `etesia-admin`, the guard keys, the faucet keys `etesia-faucet-1`–`3`, the user wallets `GA5A…2XO5` and `GCF2…AVDW` or the v3 treasury other than as a fee recipient in the swap executes; the two user-wallet transactions of the Builder run are in `t2-d5-allocation-run-2026-11-12.json`, and the `etesia-realign` tRESERVE mint `bb02fc03cccc7805dbf501f3be4bf6299ca85d0157e45a589752cb751f21478b` (ledger 5,692,779) is left to the liquidity seed's archive with the `etesia-liquidity` and `etesia-faucet-4`-and-later transactions, which are not tracked here. No realign was sent in this range. Labels are decoded from the archived envelopes and metas: the function from the envelope's invocation; each execute's nonce from its plan argument, matched by the vault's `plan` event, and its target hash equal to the one emitted by the preceding `publish_target`; Supply and Withdraw amounts from the plan action, matched by the Blend pool's `supply` or `withdraw` event; swap amounts from the v3 copy's `swap` event. Cycle numbers are not assigned: no journal or log anchor was read for this range, and a cycle that sends nothing leaves no transaction. Five publishes have no execute after them (23:10, 02:06, 03:41, 05:01 and 06:36 UTC); why is not read from the journal. Two publishes are off the cadence (07:16:33 and 14:02:13 UTC); that they are boot cycles of quant deploys is an inference, not verified. Pre-reset deployment per A76.

| Ledger | Close (UTC) | Hash | Label |
| --- | --- | --- | --- |
| 5,680,861 | 2026-11-11 20:12:28 | `77b90a0158b18466a49a9efa623d2f6302e780e0ad02d18859d2687925dd8716` | Keeper `publish_target` |
| 5,680,863 | 2026-11-11 20:12:38 | `cba1ac18cf7770d82e75e61ac3ceb725003c78f6da0a30e019a1fdb74d744b3a` | Keeper `execute`: Blend Supply 0.0092 USDC (nonce 68) |
| 5,681,080 | 2026-11-11 20:30:43 | `d17797548576c4ef8da89b61aedac7741ec7d782f4ac6f8bc80963ec0542fead` | Keeper `publish_target` |
| 5,681,082 | 2026-11-11 20:30:53 | `855d44ec6336041a8bdd5d6437c8e5fd5dd693816b63bd669f41f3b97cbea1c6` | Keeper `execute`: Blend Withdraw 0.0074 USDC (nonce 69) |
| 5,681,265 | 2026-11-11 20:46:08 | `30e5f6e9d80e201b022ff07828fd6eb1c8d136d0c8ed018734e909e1236cfc12` | Keeper `publish_target` |
| 5,681,267 | 2026-11-11 20:46:18 | `e6d7176b9f2a91ae450e77bc755da1a28ae89b2034ef2b7a50609563432f895a` | Keeper `execute`: Blend Supply 0.0051 USDC (nonce 70) |
| 5,681,449 | 2026-11-11 21:01:28 | `d2a633b5d415015d5a2c4d3261a30926108c4194f74ecff82f3da55b5a5d06a3` | Keeper `publish_target` |
| 5,681,451 | 2026-11-11 21:01:38 | `a3d914b58bcf4c758bfb55c9715549b1ca4bd7e41d38ee475264e7bb6638a2a3` | Keeper `execute`: Blend Supply 0.0157 USDC (nonce 71) |
| 5,681,634 | 2026-11-11 21:16:53 | `ca9b2ae3f8f513e380134192fe8a53fdebdfc865e67cf366c01d29fc4eb2e49e` | Keeper `publish_target` |
| 5,681,636 | 2026-11-11 21:17:03 | `7c1e215b0b0a0f791c2e770f96afd69a829598c27543838c1845a0e9ed694a3d` | Keeper `execute`: Blend Supply 0.0107 USDC (nonce 72) |
| 5,681,818 | 2026-11-11 21:32:13 | `5d09cb9544011de1b9c84a8ffffed27fb04bedb0d48bc977725fe03adc715c90` | Keeper `publish_target` |
| 5,681,820 | 2026-11-11 21:32:23 | `91aca3158dbda10ae7fbdf014a2d63a0e37dadfb83e89918f8f6cb74d5c1cef1` | Keeper `execute`: Blend Withdraw 0.0109 USDC (nonce 73) |
| 5,682,041 | 2026-11-11 21:50:48 | `af27df9074d5f279b42ada88fe1f324fa141c11723e5ed842d91815022261bca` | Keeper `publish_target` |
| 5,682,044 | 2026-11-11 21:51:03 | `4daf3a2f37fc0c694809db88babcd3d2127e06d37136c0d1d1278ef82055ce60` | Keeper `execute`: Blend Withdraw 0.0217 USDC (nonce 74) |
| 5,682,227 | 2026-11-11 22:06:18 | `c946d898da68b9cdceecf4e9780438da200c3b18db4ae77a5e07d9f19cfe004b` | Keeper `publish_target` |
| 5,682,229 | 2026-11-11 22:06:28 | `c2a26def8a975b9d5e1ec49d61bd61256bd3876b5e882a5ad322395d66d77b3b` | Keeper `execute`: Blend Supply 0.0029 USDC (nonce 75) |
| 5,682,411 | 2026-11-11 22:21:38 | `55854d7ee3f90c565953f2d2a9823af72c2da6781e3cf64e16c721d5deee7e75` | Keeper `publish_target` |
| 5,682,413 | 2026-11-11 22:21:48 | `af293a4ed61f2c5097220269c57ae9d7766f00a22f82c20d846bbee67590bff8` | Keeper `execute`: Blend Supply 0.0160 USDC (nonce 76) |
| 5,682,596 | 2026-11-11 22:37:03 | `ea791e44f83279e4fa37ee60c772fc396d4b8f3cf96b4835c575b6d66d0c609c` | Keeper `publish_target` |
| 5,682,599 | 2026-11-11 22:37:18 | `0b3ca476e753281fb3a6baa1936708a36d13e2f191546add4ad2f313e575bb87` | Keeper `execute`: Blend Withdraw 0.0383 USDC (nonce 77) |
| 5,682,782 | 2026-11-11 22:52:33 | `4dade8aa3ca0a8cc9cf29309f6cc7e631190068a2e0b6c44624ee3336b92673a` | Keeper `publish_target` |
| 5,682,784 | 2026-11-11 22:52:43 | `f0524f405f331c43f2e83c736339f4d61115a03d3850637e1027dab080c7ea4b` | Keeper `execute`: Blend Withdraw 0.0537 USDC (nonce 78) |
| 5,683,000 | 2026-11-11 23:10:43 | `52cb8cfd8a5412356af4fab48d077773dbfb1302f1084d6b45ba841b4d4fdfc7` | Keeper `publish_target` |
| 5,683,190 | 2026-11-11 23:26:33 | `f20e32dbb7392abb2b04801e443277f64d68cedb160719709aa1f7cb05b9e71f` | Keeper `publish_target` |
| 5,683,193 | 2026-11-11 23:26:48 | `ef207b2b47c27bdb0baded9eea3960564d44714b1dd78404a5c7cd90db910f6e` | Keeper `execute`: Blend Withdraw 0.0337 USDC (nonce 79) |
| 5,683,375 | 2026-11-11 23:41:58 | `928648450d5c374429b16c1bc5b23e47e886630795329e2ffb64c59917e46499` | Keeper `publish_target` |
| 5,683,377 | 2026-11-11 23:42:08 | `5f4d9a57ee61603f4dea38701df95266d467cebc2065f78108732d92a15f016f` | Keeper `execute`: Blend Supply 0.0348 USDC (nonce 80) |
| 5,683,560 | 2026-11-11 23:57:23 | `1ac8cbed2eb5c9a0b5693ed1412a19e0b2c07c5ac4a3248721b4023bc578d5c7` | Keeper `publish_target` |
| 5,683,562 | 2026-11-11 23:57:33 | `4650688cf52355fb4495b27647629501497a5514fc02c8c2960840f300354077` | Keeper `execute`: Blend Supply 0.0008 USDC (nonce 81) |
| 5,683,780 | 2026-11-12 00:15:43 | `577255249cecd39a2f6eee32d78af02145190ffe338a1b0e228b046207bf3199` | Keeper `publish_target` |
| 5,683,783 | 2026-11-12 00:15:58 | `c61d9e627a936595f57fe9dc4c9b2c1f5c44f0679f352afd159690a757123c02` | Keeper `execute`: Blend Supply 0.0029 USDC (nonce 82) |
| 5,683,965 | 2026-11-12 00:31:08 | `2da8c657674fa921d536e11d85e99f393a62ef091c62b45d4f4bd6360b33dd36` | Keeper `publish_target` |
| 5,683,967 | 2026-11-12 00:31:18 | `f2c4fa6ee814fe66f194173b6a1666cbcda5df7547d51df986396d4be1e71e70` | Keeper `execute`: Blend Supply 0.0290 USDC (nonce 83) |
| 5,684,149 | 2026-11-12 00:46:28 | `e326c9017515e24b37d02434888620e087365fa61a390611719e83c65bbe6e46` | Keeper `publish_target` |
| 5,684,151 | 2026-11-12 00:46:38 | `659cc933e07312b8e0398286901e150378420bcbb3fa4008db3f2117060d22ed` | Keeper `execute`: Blend Withdraw 0.0027 USDC (nonce 84) |
| 5,684,334 | 2026-11-12 01:01:53 | `e1636e5017315c9b00829a82cbd585e9ae477e995cbc3c5826cf8c4b37499dba` | Keeper `publish_target` |
| 5,684,336 | 2026-11-12 01:02:03 | `7c22279e95751d011c0d3d69fa8204f691d48ecc60c92563607d95f46f3c49a2` | Keeper `execute`: Blend Supply 0.0047 USDC (nonce 85) |
| 5,684,519 | 2026-11-12 01:17:18 | `41c5a5163467a95a22ba5f5255e3092c557f3743e7c9a9aa65d813fc2a2cb99b` | Keeper `publish_target` |
| 5,684,521 | 2026-11-12 01:17:28 | `bc6831bbd0eafc1893788ab790fa38de5302bae09755f516f2199f81dffc4b52` | Keeper `execute`: Blend Withdraw 0.0202 USDC (nonce 86) |
| 5,684,740 | 2026-11-12 01:35:43 | `3b96c4636477a9424fb14d73aaaab17ba174c82f12654e2a07a21b523030afaf` | Keeper `publish_target` |
| 5,684,742 | 2026-11-12 01:35:53 | `f4fb4fcd6c94947a4cfba0c09e836f3857542d74ae9c9f441a90670e769497e2` | Keeper `execute`: Blend Supply 0.0121 USDC (nonce 87) |
| 5,684,924 | 2026-11-12 01:51:03 | `d41d30bd7636d383d81c31178a9a4e35fe7b7648e0adafbeddb71d9ac822ee70` | Keeper `publish_target` |
| 5,684,926 | 2026-11-12 01:51:13 | `52c5c31c2c3d99c661bdee2d2b85705daf8d7e6aa8bed6b6a38b88a6405f74fa` | Keeper `execute`: Blend Withdraw 0.0144 USDC (nonce 88) |
| 5,685,109 | 2026-11-12 02:06:28 | `b51966e761724f01ecaab36bc97a7e99084362294657bb9efed49cdaab33fe46` | Keeper `publish_target` |
| 5,685,293 | 2026-11-12 02:21:48 | `0e938d59434b8558a7fe22976ff4dbce707cfd4a64bed487f8bcab770471b1da` | Keeper `publish_target` |
| 5,685,296 | 2026-11-12 02:22:03 | `b9a578771c47abf3097e60edba6294408602eef1465c52bf5806c206bd5bfd4d` | Keeper `execute`: Blend Supply 0.0156 USDC (nonce 89) |
| 5,685,479 | 2026-11-12 02:37:18 | `972365c44d852ecf0b3895921f66109e1a48961f9d7a40d93a07694ffd12ee34` | Keeper `publish_target` |
| 5,685,481 | 2026-11-12 02:37:28 | `2fef7a431e3f46eafc34957d2212a04f66136053caea165e6d745e8a501586f1` | Keeper `execute`: Blend Supply 0.0083 USDC (nonce 90) |
| 5,685,699 | 2026-11-12 02:55:38 | `3b562e0f8ba88d8929c52dedeed90c2fca4da33fc2a6f087a8630446533865aa` | Keeper `publish_target` |
| 5,685,701 | 2026-11-12 02:55:48 | `e45a2d87cbe617f393a61bf24c2d19221ed5bcbd6ab21293408e2f97f7d2fbd0` | Keeper `execute`: Blend Withdraw 0.0184 USDC (nonce 91) |
| 5,685,884 | 2026-11-12 03:11:03 | `6f82bf2993b7ec09d8ffd1d7badb5f18ec64b7e8fb7726834f8faf363e0e7679` | Keeper `publish_target` |
| 5,685,887 | 2026-11-12 03:11:18 | `8b3f98aa808f734b7f447913d0121009c679b2398a3af17ee107efbf05a61656` | Keeper `execute`: Blend Withdraw 0.0134 USDC (nonce 92) |
| 5,686,069 | 2026-11-12 03:26:28 | `eccfd239c811adf3069477eb3fef49425a735fa55855af68f7ab0bcdef32b259` | Keeper `publish_target` |
| 5,686,071 | 2026-11-12 03:26:38 | `85af09ced6703d199979f93fd21bdae26c2cd97cb80234b76e9cc8871202788c` | Keeper `execute`: Blend Withdraw 0.0145 USDC (nonce 93) |
| 5,686,253 | 2026-11-12 03:41:48 | `537f16f91332274c0c15566bc3575460dbbcaafe662cbd0ef3b6551f1b4a1ca2` | Keeper `publish_target` |
| 5,686,436 | 2026-11-12 03:57:03 | `06b23add4c7a3e8afc3e6b1b03c1de91cd7dc2fe851e3eb2325ca41184c84c07` | Keeper `publish_target` |
| 5,686,439 | 2026-11-12 03:57:18 | `41d0d3a2a7408cd8c82aca5e7f9f929843cd43fb549de38f55bd7b000417342f` | Keeper `execute`: Swap 95.81 USDC→713.81 XLM through the v3 copy (nonce 94) |
| 5,686,621 | 2026-11-12 04:12:28 | `2de6b6fad6d52939b158132815e96d1b3e410301e2e8d75f249e791ab502aa89` | Keeper `publish_target` |
| 5,686,623 | 2026-11-12 04:12:38 | `b8f4a44c277a749e30e0b23c260696d6bf68e3aed52dd34f562d90eefd98bd53` | Keeper `execute`: Blend Supply 0.7491 USDC (nonce 95) |
| 5,686,839 | 2026-11-12 04:30:38 | `96d9e618e568c417bb8d98ce9684f7c30b93c4d907f0185ae744928d98c07fdb` | Keeper `publish_target` |
| 5,686,842 | 2026-11-12 04:30:53 | `2ba964e67b271048b2fbc617be5a4ef8bb7c1cabd39bc7295bd2d954ac25ba8a` | Keeper `execute`: Blend Withdraw 0.0164 USDC (nonce 96) |
| 5,687,024 | 2026-11-12 04:46:03 | `fdf12dae7033bfd1cb9eee6fe044d9049faa3ba3d54d6ca29aaec3fcfff00256` | Keeper `publish_target` |
| 5,687,026 | 2026-11-12 04:46:13 | `ebb7af827179dc5222a1c8a8791540cae4a7ed92d075938b07582485bbb22811` | Keeper `execute`: Blend Withdraw 0.0174 USDC (nonce 97) |
| 5,687,208 | 2026-11-12 05:01:23 | `6901d489be99150ed590d735c94fc6d33bd4eb5d35416672ef4ad843e21e8ed9` | Keeper `publish_target` |
| 5,687,391 | 2026-11-12 05:16:38 | `e8ad494e8436189f5a6faa157d12b5d045db1843c0797102da91cdbe50b95032` | Keeper `publish_target` |
| 5,687,393 | 2026-11-12 05:16:48 | `d55026857c0e6fd58f3ea0ea6301b59f760569fd0a722aae65dc34ca184e5ece` | Keeper `execute`: Blend Withdraw 0.0448 USDC (nonce 98) |
| 5,687,576 | 2026-11-12 05:32:03 | `a900b2245a07f11fe9666b7ffb8ffdb469c192eba6609f434638b0b5ab855510` | Keeper `publish_target` |
| 5,687,578 | 2026-11-12 05:32:13 | `6e019009635f0460f5a0b7ac489564549de15a043fcfccc8e98b457f2f059d5e` | Keeper `execute`: Blend Supply 0.0432 USDC (nonce 99) |
| 5,687,760 | 2026-11-12 05:47:23 | `12b16a077e514361b9104479760b616c30f700431bdaf256b7587380b7a23594` | Keeper `publish_target` |
| 5,687,762 | 2026-11-12 05:47:33 | `3eb85c0d78856c04d5e53001e8d37c4ab464a69c34a395550543334b6b969051` | Keeper `execute`: Blend Withdraw 0.0042 USDC (nonce 100) |
| 5,687,979 | 2026-11-12 06:05:38 | `03e94e1304db250e5eacbacb78b22974a7440960f9f4eb38eb63877f12310660` | Keeper `publish_target` |
| 5,687,981 | 2026-11-12 06:05:48 | `4b9bc1532876e22e5876c01afe26ae78baf2873f24e2dfa74bba6030d8ea6432` | Keeper `execute`: Blend Supply 0.0001 USDC (nonce 101) |
| 5,688,163 | 2026-11-12 06:20:58 | `ee1a169f55f88e7a41e6bbf4e38251f5bda4875f5c01b44f5effbedb0ee42117` | Keeper `publish_target` |
| 5,688,165 | 2026-11-12 06:21:08 | `d61f5046fe654b6a6da15587269f220624d3f34d97c9f02774dfdc9caee6016c` | Keeper `execute`: Blend Withdraw 0.0117 USDC (nonce 102) |
| 5,688,348 | 2026-11-12 06:36:23 | `8e104f4e46cf459c7a53830c8d352fd35a124764b173e6276d3223445c3d5cb8` | Keeper `publish_target` |
| 5,688,537 | 2026-11-12 06:52:08 | `0e34d9ca3bd41f58d7a52f907d4302982bfc019d42f881624ea0e5cc8e557f52` | Keeper `publish_target` |
| 5,688,539 | 2026-11-12 06:52:18 | `f65b146267d37bb6a0c061eb7b35866988169a43c236c8c0c1301725a46127bf` | Keeper `execute`: Blend Withdraw 0.0301 USDC (nonce 103) |
| 5,688,722 | 2026-11-12 07:07:33 | `b51a23f3a0c3c3f89120789d9b0da29388d3a6ac458a0616102dccaaeacb28e1` | Keeper `publish_target` |
| 5,688,724 | 2026-11-12 07:07:43 | `a5303dfb556a980fe1fecdf2bc0707212646fec8d3de1170f3c1b7368f0198d3` | Keeper `execute`: Blend Withdraw 0.0022 USDC (nonce 104) |
| 5,688,830 | 2026-11-12 07:16:33 | `73123b7768e765bfd86ed3497b25f7734081f4ba48874171a8b31faa30eab053` | Keeper `publish_target`, 9.0 min after the previous publish (off the 15-minute cadence; cause not read from the journal) |
| 5,688,832 | 2026-11-12 07:16:43 | `a4105f63a5fcc3566eb9af548666499c859800a2f9250495b209164066c53533` | Keeper `execute`: Blend Withdraw 0.0144 USDC (nonce 105) |
| 5,689,015 | 2026-11-12 07:31:58 | `69ce4822b17a1fbb5e087cf364594f0a632ec338e521302f2d645c3810acce21` | Keeper `publish_target` |
| 5,689,017 | 2026-11-12 07:32:08 | `2c060a235401622d7a94416e5784276dc516def147aeb3d6bb31249df1300970` | Keeper `execute`: Blend Supply 0.0019 USDC (nonce 106) |
| 5,689,199 | 2026-11-12 07:47:18 | `51c2b9d6f03e2d4d046883644cf6a908e1dc3140d1acf96a44aea4d4fb33c5b6` | Keeper `publish_target` |
| 5,689,201 | 2026-11-12 07:47:28 | `4d2b93961f97641e187b97e3ebd122ce1c57ae5d94b3e120e24681b3c99d9ed5` | Keeper `execute`: Blend Withdraw 0.0044 USDC (nonce 107) |
| 5,689,419 | 2026-11-12 08:05:38 | `89afbf7213686f89337ab7edd9cd55f2ccc0518ae1d11cfbf4a85e5b7fc268a8` | Keeper `publish_target` |
| 5,689,421 | 2026-11-12 08:05:48 | `2b609a3fc0a3844d98d375ce8e600225552a6cec1fa486ea80fc67daa5f373d9` | Keeper `execute`: Blend Supply 0.0083 USDC (nonce 108) |
| 5,689,604 | 2026-11-12 08:21:03 | `608dedad3f62118f995828ac7e06ae96042d616b8d2aa647a17980254d07de92` | Keeper `publish_target` |
| 5,689,606 | 2026-11-12 08:21:13 | `5bb9fae0c66686dc74778cb2196bdcb2d85652edd8043796fd42aa09b809d6b0` | Keeper `execute`: Blend Withdraw 0.0122 USDC (nonce 109) |
| 5,689,788 | 2026-11-12 08:36:23 | `05d482d65e3e0fce4568b3f372dde8f36d8d2fc08eac59cb3e26ce07ab49510c` | Keeper `publish_target` |
| 5,689,790 | 2026-11-12 08:36:33 | `5aa3c0cad3d2f7833293e11b65a52f7aa84e3de7feca5266d6ef3d71b40443d7` | Keeper `execute`: Blend Withdraw 0.0089 USDC (nonce 110) |
| 5,689,973 | 2026-11-12 08:51:48 | `e7f833b747bb9ac8c96e3366f484e30651aaf541c09acacc785e2e1f5dfea6c4` | Keeper `publish_target` |
| 5,689,976 | 2026-11-12 08:52:03 | `f093083785dce6de8738712247a250376252db26c5217528f755c1d4c6e2767a` | Keeper `execute`: Swap 5.44 USDC→39.92 XLM through the v3 copy (nonce 111) |
| 5,690,158 | 2026-11-12 09:07:13 | `a174a3f8a833efd551858c0636069a45357c2d58b7484dee3318090a46988622` | Keeper `publish_target` |
| 5,690,161 | 2026-11-12 09:07:28 | `65d6b9998aea466ee51899127050b799b9b069dbe62f487f913145b9bffea7a1` | Keeper `execute`: Blend Supply 6.7781 USDC (nonce 112) |
| 5,690,381 | 2026-11-12 09:25:48 | `28bb8db35630d4f3a970872d50caf3683ee225fed43cd42b75e9dd4a54691ac4` | Keeper `publish_target` |
| 5,690,383 | 2026-11-12 09:25:58 | `431786e0c4574bc7a1781feaf50e767ec6b7f9843fcaa18db7ba761bb553aeaa` | Keeper `execute`: Blend Supply 0.0227 USDC (nonce 113) |
| 5,690,565 | 2026-11-12 09:41:08 | `f4ee9266ed559425de66eff05f9f1d66abcff9c79ad026ce02a81e009ad4b819` | Keeper `publish_target` |
| 5,690,567 | 2026-11-12 09:41:18 | `88767777b24c3bc91d41bab3920efe227f8f11c595e209d16502f2e167aef71e` | Keeper `execute`: Blend Supply 0.0527 USDC (nonce 114) |
| 5,690,750 | 2026-11-12 09:56:33 | `c2be6346bdf73c86d583502d3c488408816a9d0833fa2c5195f32faced6f5e5b` | Keeper `publish_target` |
| 5,690,753 | 2026-11-12 09:56:48 | `7751178834773c1ce6775d7334cc0cd937403dee3ce8bae87afd919eccd17f62` | Keeper `execute`: Blend Supply 0.0054 USDC (nonce 115) |
| 5,690,936 | 2026-11-12 10:12:03 | `fac2290e347af753d8aea5d08c037b430046381dc4397d70f8810e893eaa1823` | Keeper `publish_target` |
| 5,690,938 | 2026-11-12 10:12:13 | `299cebb5d03183f5790f715208011897e87ce3959809a2421f5f1cb2f73785ba` | Keeper `execute`: Blend Supply 0.0048 USDC (nonce 116) |
| 5,691,121 | 2026-11-12 10:27:28 | `f51b480f31ab30b620eb789f3be25860cc541792072c356e44544015fb17b43d` | Keeper `publish_target` |
| 5,691,124 | 2026-11-12 10:27:43 | `618936d5dd68bad4317d7ce6b2435953520153ffb03265c67f087195e6c58d98` | Keeper `execute`: Blend Supply 0.0284 USDC (nonce 117) |
| 5,691,340 | 2026-11-12 10:45:43 | `47c595a1a0d03aea82371e167a16e28455d7d54e225c4c2a8c85312bcb5d0a04` | Keeper `publish_target` |
| 5,691,342 | 2026-11-12 10:45:53 | `9ecb0cce857dcf19f5fdfe1bfeca8b0da5d3febba20999bdc7b8637934faf73c` | Keeper `execute`: Blend Withdraw 0.0524 USDC (nonce 118) |
| 5,691,524 | 2026-11-12 11:01:03 | `5ebab133534c1970e7b3164d768fd14f3532c37c0ab97025ce09aefa138ae958` | Keeper `publish_target` |
| 5,691,526 | 2026-11-12 11:01:13 | `2831e5f81b5542d1871ddfc4b9d54b510447cf1074ab56c2a27bc5fea2567692` | Keeper `execute`: Blend Withdraw 0.0188 USDC (nonce 119) |
| 5,691,708 | 2026-11-12 11:16:23 | `63fdcf31008d7b21405e6af142480a88925afb95e4a0b340ed8ccecc7fdf6e2d` | Keeper `publish_target` |
| 5,691,710 | 2026-11-12 11:16:33 | `ffeccec61f612aa5d838023e5b42257f9fd506e235f7e1935de849d15f0e68a9` | Keeper `execute`: Blend Withdraw 0.0080 USDC (nonce 120) |
| 5,691,892 | 2026-11-12 11:31:43 | `789199239609b3cbf4f5cb647eb9d1062019840e8223fe8ad8027997cb2057b3` | Keeper `publish_target` |
| 5,691,894 | 2026-11-12 11:31:53 | `2e81be97547eeb689cb3d05040c2aa90df2910e7eeb0a281ed592153fb2ed6c8` | Keeper `execute`: Blend Withdraw 0.0570 USDC (nonce 121) |
| 5,692,076 | 2026-11-12 11:47:03 | `afb690c6606fdac7bc608533a11d8b6cd46df81591cbc1ac1721c2912d21de9e` | Keeper `publish_target` |
| 5,692,078 | 2026-11-12 11:47:13 | `f36919505e9a10f4c1000e15284408b912a70068ca182a6705f68f4c044488f0` | Keeper `execute`: Blend Withdraw 0.0344 USDC (nonce 122) |
| 5,692,260 | 2026-11-12 12:02:23 | `7b233fde464792e2c22e3bc5b65f05a7f3301e732538556ad1c91394d9e093e8` | Keeper `publish_target` |
| 5,692,262 | 2026-11-12 12:02:33 | `824c30dee3e45e9414ed02b36e36a41fb06d3e331470e7bf7ff3647dc7cfb1d9` | Keeper `execute`: Blend Withdraw 0.0420 USDC (nonce 123) |
| 5,692,480 | 2026-11-12 12:20:43 | `ce0680d2b8e28c57cd3fd32b18981c506cdf657ff7b98dcb5ad518ba302dd260` | Keeper `publish_target` |
| 5,692,482 | 2026-11-12 12:20:53 | `fb47f2034d0bb25f335296a0b1e4f85cb7747e5f6c6fa4655fea0d7ea725bc62` | Keeper `execute`: Swap 0.34 USDC→2.50 XLM through the v3 copy (nonce 124) |
| 5,692,664 | 2026-11-12 12:36:03 | `602adf0d674c4c16f2da4b27be233a1964c2ba857942db18cc727bac861952b9` | Keeper `publish_target` |
| 5,692,666 | 2026-11-12 12:36:13 | `be800c20ce5495e9c42b2a8f2c6bb811c29f23b024ea4c45dbfee15561c8941d` | Keeper `execute`: Swap 0.11 USDC→0.80 XLM through the v3 copy (nonce 125) |
| 5,692,848 | 2026-11-12 12:51:23 | `d62021c73db20f56da7ea2b8978d0cec97fc1d9875d1228347150ae8200f8af5` | Keeper `publish_target` |
| 5,692,850 | 2026-11-12 12:51:33 | `df6b6a299f56ec5bfe0862123bfd079cbb6fd2972617ba88ce8bad8380159f78` | Keeper `execute`: Swap 0.06 USDC→0.46 XLM through the v3 copy (nonce 126) |
| 5,693,032 | 2026-11-12 13:06:43 | `0e4e6f1bc5ac0be2b37db4b1d97d6e69cd357d8b8fd66cb2f6cceeef9c6ffee3` | Keeper `publish_target` |
| 5,693,034 | 2026-11-12 13:06:53 | `a73734bf7f75e042d4233246e6c706bfc3134ef39fc0d33d12688a75e23aa031` | Keeper `execute`: Swap 0.05 USDC→0.39 XLM through the v3 copy (nonce 127) |
| 5,693,216 | 2026-11-12 13:22:03 | `5cf2c56218ad4fbba4d9ab9ed39ff664a7ca3752e125fe57a4d783f97631da0f` | Keeper `publish_target` |
| 5,693,218 | 2026-11-12 13:22:13 | `3e36f80d027223735895d867893840c6bf593928e4dbe50cd7709088d24076e3` | Keeper `execute`: Blend Withdraw 0.0993 USDC (nonce 128) |
| 5,693,400 | 2026-11-12 13:37:23 | `71603458a5547c1665bad14debfa9541b0a47fd8532f1ad9364c6b09de30f144` | Keeper `publish_target` |
| 5,693,402 | 2026-11-12 13:37:33 | `4d32f21ff6a6bb41ae33743d0b2640d79c697ddfcbb1a985816fba91e49f6d39` | Keeper `execute`: Swap 0.49 USDC→3.63 XLM through the v3 copy (nonce 129) |
| 5,693,620 | 2026-11-12 13:55:43 | `ffeeef0bbeb9577227e88159cb5915383036b9fafa659f6ba94da3bf655cd106` | Keeper `publish_target` |
| 5,693,622 | 2026-11-12 13:55:53 | `d2ee816f0fe35f4dce8f4f70e77a154499da57c6fb12eb029f9411bd0409bd87` | Keeper `execute`: Swap 0.20 USDC→1.48 XLM through the v3 copy (nonce 130) |
| 5,693,698 | 2026-11-12 14:02:13 | `f3a6cc95d1af01ec9ed7ce4bbebeaf135fc6c4085f50cc1908eab3e3cf10b731` | Keeper `publish_target`, 6.5 min after the previous publish (off the 15-minute cadence; cause not read from the journal) |
| 5,693,700 | 2026-11-12 14:02:23 | `111a9c35847abf1fdad8413cb97152eaf7bd11ab60e0ca4a4733428e9b605c24` | Keeper `execute`: Blend Supply 0.0648 USDC (nonce 131) |

## t2-liquidity-setup-2026-11-12.json

A99 testnet liquidity float and seed, run with `scripts/testnet-deploy.py liquidity` (vault `14c2a48`; `account`, `mint`, `claims` and `seed --yes`) and two `realign` runs. Labels are from the script's journals [`d4-liquidity-pre-2026-12-16-reset.jsonl`](d4-liquidity-pre-2026-12-16-reset.jsonl) (the `included` records) and [`d4-realign-pre-2026-12-16-reset.jsonl`](d4-realign-pre-2026-12-16-reset.jsonl) (its last four records), and the account creation from its envelope. Each Blend faucet claim (envelope sourced by the faucet issuer `GATALT…5V56`, signed by the fresh key) gave 1,000 USDC, 5,000 BLND, 0.5 wETH and 0.05 wBTC; each forward-and-merge paid the 1,000 USDC to `etesia-liquidity`, returned BLND, wETH and wBTC to the issuer, removed the four trustlines and merged the key into `etesia-liquidity`. After the claims the float held 10,000 USDC, 50,000 tRESERVE and about 110,000 XLM. The realigns brought the XLM pair to the Reflector mid (0.14324 USDC per XLM) and the tRESERVE pair to parity before the seed. The journal's reads after each seed: both pairs at 9,900.00 USDC, and a 100 USDC trade at 99.70 bps (0.997%) impact on each. The float after the seed (Horizon testnet, read 2026-11-12 15:12 UTC at ledger 5,694,535, no later transaction): 289.21 USDC, 41,100.06 tRESERVE, 104,339.11 XLM. The file lists the realigns after the seeds; the rows below are in ledger order. The ten faucet keys' friendbot creations are not archived (above). Pre-reset deployment per A76.

| Ledger | Close (UTC) | Hash | Label |
| --- | --- | --- | --- |
| 5,692,774 | 2026-11-12 12:45:13 | `5e4bd4e0786d55410913661e30855c57c465eeb41746f779497ac3454eea4f1c` | Friendbot creates `etesia-liquidity` `GBRMID…FQEP` (10,000 XLM; from the envelope, the journal's `account` line has no hash) |
| 5,692,775 | 2026-11-12 12:45:18 | `5e95eec6b4959d1157904054a280f28936a6007b139e1453c0d6d007cbc517b4` | `etesia-liquidity` USDC trustline |
| 5,692,776 | 2026-11-12 12:45:23 | `59b74e740964005bf231dd239511f37a3e35b1a594b9a537c263c29a10b8dccc` | `etesia-liquidity` tRESERVE trustline |
| 5,692,779 | 2026-11-12 12:45:38 | `bb02fc03cccc7805dbf501f3be4bf6299ca85d0157e45a589752cb751f21478b` | 50,000 tRESERVE minted by `etesia-realign` (issuer) to `etesia-liquidity` |
| 5,692,786 | 2026-11-12 12:46:13 | `c0effcd67eaf7dd5c63a2c5343c65679e816f8a9a52cc9199d000fde532cd9fb` | Blend faucet claim, `etesia-faucet-4` `GDI7Z6…RJP4` |
| 5,692,788 | 2026-11-12 12:46:23 | `4fafde30d1d8fdbfe7ae1b8f58cd2637baa15f7973b62adabf2ec4ae31662002` | Forward-and-merge, `etesia-faucet-4` → `etesia-liquidity` (1,000 USDC) |
| 5,692,969 | 2026-11-12 13:01:28 | `9f62b2760f966a6858bd04b6b8560c8d40134a570c94c5478ee5f8fa3b42abeb` | Blend faucet claim, `etesia-faucet-5` `GB3BMC…S5OM` |
| 5,692,971 | 2026-11-12 13:01:38 | `0cfe4b270a4693cff203ebda949a484fbb1ed44c25a596595d8a5f2bb3ebf9d3` | Forward-and-merge, `etesia-faucet-5` → `etesia-liquidity` (1,000 USDC) |
| 5,693,152 | 2026-11-12 13:16:43 | `282783a9317d77bc5704353873f159de719fc8b56fdcd959bdf383c335ac0f53` | Blend faucet claim, `etesia-faucet-6` `GBZBKD…XPLZ` |
| 5,693,154 | 2026-11-12 13:16:53 | `d865c57b894d425361855458ea9d89b7d059e3490ea7646117cd245ba0ba23dc` | Forward-and-merge, `etesia-faucet-6` → `etesia-liquidity` (1,000 USDC) |
| 5,693,335 | 2026-11-12 13:31:58 | `20b4f00dcbc806d9e271181292ce2b61f7298bcfbe2d7f6b5143ae0ae3b80cbf` | Blend faucet claim, `etesia-faucet-7` `GAPPSN…R2VF` |
| 5,693,337 | 2026-11-12 13:32:08 | `aa6b6d56db40f0ea57c190f873e71daee62d1842918bbc58d39f31dc7a2c4a2b` | Forward-and-merge, `etesia-faucet-7` → `etesia-liquidity` (1,000 USDC) |
| 5,693,518 | 2026-11-12 13:47:13 | `780ecf1c91565d5b2f66b7d5f43a1c74b410acd2fea682449e9c3806a8929b7a` | Blend faucet claim, `etesia-faucet-8` `GBS6JL…VLJF` |
| 5,693,519 | 2026-11-12 13:47:18 | `c0f07ab500c50972838b5e14bd10e69d80d1da874aa1d6377a089e4e52202065` | Forward-and-merge, `etesia-faucet-8` → `etesia-liquidity` (1,000 USDC) |
| 5,693,701 | 2026-11-12 14:02:28 | `604ae314e9617a5753470dd217872ed98c7c8934db658e9b32b840e1c8c2e3a4` | Blend faucet claim, `etesia-faucet-9` `GCI5EI…D3MJ` |
| 5,693,702 | 2026-11-12 14:02:33 | `dd93e26f153c641c900de28458137455ae363eb0f3eb9256394a5fb68b11f909` | Forward-and-merge, `etesia-faucet-9` → `etesia-liquidity` (1,000 USDC) |
| 5,693,884 | 2026-11-12 14:17:44 | `148b49ce6964dc77bcb9428bc95fce97e84a1f6b65c41aa5b355d227b7d1a902` | Blend faucet claim, `etesia-faucet-10` `GAR2H5…6V6Q` |
| 5,693,885 | 2026-11-12 14:17:49 | `bbe1d5ac5f56dd591ae2b2ae973ceaa3af8cc044377f6e1aa23ea89e4c5865d4` | Forward-and-merge, `etesia-faucet-10` → `etesia-liquidity` (1,000 USDC) |
| 5,694,067 | 2026-11-12 14:32:59 | `8b67611641137836077ec47828724b57bb7ca5f158f060e9de9bb5fe877bc3ef` | Blend faucet claim, `etesia-faucet-11` `GCNQDD…VUSW` |
| 5,694,069 | 2026-11-12 14:33:09 | `05ac5af00d6f124ae36500d8dcf6fadc2c59f30b7538640fa23170f1aad7ff96` | Forward-and-merge, `etesia-faucet-11` → `etesia-liquidity` (1,000 USDC) |
| 5,694,250 | 2026-11-12 14:48:14 | `980948bdc53cd3ecce6343c68bb41c282854de155841cc83f7dcb0c63bdd9979` | Blend faucet claim, `etesia-faucet-12` `GBWXPH…PU2V` |
| 5,694,252 | 2026-11-12 14:48:24 | `1b0fe7912c8f6bfffdbab2d07879dceb62cc235c1e3f050b0cdbb9c615f81633` | Forward-and-merge, `etesia-faucet-12` → `etesia-liquidity` (1,000 USDC) |
| 5,694,433 | 2026-11-12 15:03:29 | `e8563d70082e161e12095217defd9a2f54e5ff2290b073138e5352e563a2a705` | Blend faucet claim, `etesia-faucet-13` `GA24J6…XIAQ` |
| 5,694,435 | 2026-11-12 15:03:39 | `e67989bd3f714825d4702545603ab30263b7ad027daee9c61dd5fbae60eb5f8d` | Forward-and-merge, `etesia-faucet-13` → `etesia-liquidity` (1,000 USDC) |
| 5,694,443 | 2026-11-12 15:04:19 | `fc1ad1994655afb971feae93c0806a8b1b94efac0016c458702ad96cea70793b` | XLM pair realign by `etesia-realign` (−515.4 bps → 0; 237.66 USDC → 1,698.62 XLM) |
| 5,694,445 | 2026-11-12 15:04:29 | `dda0573f3e69ab9d120107957243700344d14588fc1bbf51207c88702ec7360e` | tRESERVE pair realign by `etesia-realign` (+99.3 bps → 0; 4.94 tRESERVE → 4.95 USDC) |
| 5,694,449 | 2026-11-12 15:04:49 | `6245231c625445a32222afce2cd938dc71ebb0bbba88710d87a9c1bfb4fbeaa4` | XLM/USDC seed, Soroswap router `add_liquidity` from `etesia-liquidity`: 810.85 USDC + 5,660.83 XLM for 2,112.01 LP; reserves after 9,900.00 USDC / 69,115.65 XLM |
| 5,694,451 | 2026-11-12 15:04:59 | `a1f66b7279423572e9417d73ffd2a7d1975172ae39950d1ee5a924a668c68848` | tRESERVE/USDC seed, Soroswap router `add_liquidity` from `etesia-liquidity`: 8,899.94 USDC + 8,899.94 tRESERVE for 8,899.41 LP; reserves after 9,900.00 USDC / 9,900.00 tRESERVE |


## t2-liquidity-faucet-keys-2026-11-12.json

The Friendbot `create_account` of each fresh faucet key used for the A99 claims (`testnet-deploy.py liquidity claims 10 --yes`, vault `14c2a48`). Each hash is the key's first operation on Horizon testnet; key order follows the script's names. Recorded 2026-11-12 15:18 UTC with `record-evidence.py --testnet`, 10 of 10 `SUCCESS`. Pre-reset deployment per A76.

| Ledger | Close (UTC) | Hash | Label |
| --- | --- | --- | --- |
| 5,692,784 | 2026-11-12 12:46:03 | `0861da559e25fe0afeb1c34803b27845df4f1a8db3a86faf7a38e18e38f80ed5` | Friendbot `create_account` of `etesia-faucet-4` (10,000 XLM); the key's claim and forward/merge are in `t2-liquidity-setup-2026-11-12.json` |
| 5,692,967 | 2026-11-12 13:01:18 | `3307ac810c4de5e88ed2dc881d620bd45cce0f92a1cbe9ca02cb3c7eb80ae14a` | Friendbot `create_account` of `etesia-faucet-5` (10,000 XLM); the key's claim and forward/merge are in `t2-liquidity-setup-2026-11-12.json` |
| 5,693,150 | 2026-11-12 13:16:33 | `d98a7bac49ab5444938c5619a3236e1d62f7e86cda03dbc4ab0f3f5a00867b5d` | Friendbot `create_account` of `etesia-faucet-6` (10,000 XLM); the key's claim and forward/merge are in `t2-liquidity-setup-2026-11-12.json` |
| 5,693,333 | 2026-11-12 13:31:48 | `943b0fdab3a67c94ac04e63d9ef901c78e393d9672772755f297cd3632d001bb` | Friendbot `create_account` of `etesia-faucet-7` (10,000 XLM); the key's claim and forward/merge are in `t2-liquidity-setup-2026-11-12.json` |
| 5,693,516 | 2026-11-12 13:47:03 | `6012cea8bad09683fd476fcec49c18010967476a1df64d248cd4c38c7fe8bff2` | Friendbot `create_account` of `etesia-faucet-8` (10,000 XLM); the key's claim and forward/merge are in `t2-liquidity-setup-2026-11-12.json` |
| 5,693,699 | 2026-11-12 14:02:18 | `83dabe76f6ae6f4c9fec44e45161ec791a5c89d83a9747938f85e608cfd915b8` | Friendbot `create_account` of `etesia-faucet-9` (10,000 XLM); the key's claim and forward/merge are in `t2-liquidity-setup-2026-11-12.json` |
| 5,693,882 | 2026-11-12 14:17:34 | `2246049ad468614f6c3189c51acf1b1dd2974261e32d1aaa16430f9232fbd732` | Friendbot `create_account` of `etesia-faucet-10` (10,000 XLM); the key's claim and forward/merge are in `t2-liquidity-setup-2026-11-12.json` |
| 5,694,065 | 2026-11-12 14:32:49 | `6dbadef3e9ced8d4d4def5e0aea2bf564c0ddbf64fbdd1f49391af136ac762f0` | Friendbot `create_account` of `etesia-faucet-11` (10,000 XLM); the key's claim and forward/merge are in `t2-liquidity-setup-2026-11-12.json` |
| 5,694,248 | 2026-11-12 14:48:04 | `3cbcf1c0ba7213273b42157ab1909f748c1beb114f0a7558a628072f89addb5c` | Friendbot `create_account` of `etesia-faucet-12` (10,000 XLM); the key's claim and forward/merge are in `t2-liquidity-setup-2026-11-12.json` |
| 5,694,431 | 2026-11-12 15:03:19 | `308fbc2958509cfd8f70c77170aae99a02966542cca75dd2853e20fb3287673a` | Friendbot `create_account` of `etesia-faucet-13` (10,000 XLM); the key's claim and forward/merge are in `t2-liquidity-setup-2026-11-12.json` |

## t2-liquidity-remove-2026-11-12.json

D6-04 row 13, `liquidity remove`, exercised with a small real removal on the user's answer “Small real removal” (A101, 2026-11-12). Sent by the agent over `railway ssh` from the quant service: `python -m etesia_quant.liquidity remove --data-dir /data --pair CC47HAL6…555UA --liquidity 100000000`, first without `--yes` (plan: minimums 100,488,660 USDC units and 99,328,285 tRESERVE units, 0.5% below pro rata), then with `--yes`. Signed by `etesia-liquidity` `GBRMIDTC…FQEP`. Sized at 10 LP units, not the 1% of the account's LP the agent first proposed: the pair held 9,957.75 USDC and 9,842.76 tRESERVE (RPC `get_reserves`, 21:3x UTC), where a 100 USDC trade was at about 99.1 bps, and a 1% removal would have taken it past monitor A15's 100 bps depth threshold. Pre-reset deployment per A76.

| Ledger | Close (UTC) | Hash | Label |
| --- | --- | --- | --- |
| 5,699,152 | 2026-11-12 21:36:44 | `6948549cbaa4fe0ec270b2ea8c1b14783b711a80ef5f31f0b1f09cf061b0b811` | Router `remove_liquidity` 10 LP units of tRESERVE/USDC → 10.0589250 USDC + 9.9427713 tRESERVE to `etesia-liquidity` (return value) |

## t2-d6-04-basket-exit-2026-11-12.json

D6-04 row 6, the basket exit with the backend off, run by the agent on the user's go-ahead ("continue what you are doing then do 1-5 when you can tonight", 2026-11-12) with a fresh Etesia test key `etesia-exit-drill` `GBQF33UD26K7D5P4JNJ2F7WUVDZFK46LVNJ64YDTPCWHHQR25KHZHLLK`, not the user's wallet (the D5-07 run 2 with the user's wallet in the Builder stays separate). Setup with the keeper on: Friendbot account, USDC trustline, 20 USDC from `etesia-admin`, `deposit_bounded` of 20 USDC. Then the keeper and the indexer were turned off (`ETESIA_KEEPER_ENABLED=0`, `ETESIA_INDEXER_ENABLED=0`; Railway deployment `722ac145`, created 22:50:43 UTC; last keeper heartbeat cycle 189 at 22:42:40 UTC, no unresolved attempt) and the exit ran through the contract directly (`stellar contract invoke`), with bounds 0.5% from the previews. Labels from the CLI's return values and events. Pre-reset deployment per A76.

| Ledger | Close (UTC) | Hash | Label |
| --- | --- | --- | --- |
| 5,699,983 | 2026-11-12 22:45:59 | `365c5c0e42ad96e6eb49d7a7e4460286f5a57e513410b748ac0566a1edde1615` | Friendbot `create_account` of `etesia-exit-drill` |
| 5,699,984 | 2026-11-12 22:46:04 | `96a585fbe1fbf3376bcda1e088013411bb046dd3e2e65c390e51ec40058c0e08` | `changeTrust` USDC |
| 5,699,985 | 2026-11-12 22:46:09 | `363eb5f23a7117c2e9d051ffa9fbac74984fb8c234c44b39255118b9c0e7fc24` | `etesia-admin` payment 20 USDC |
| 5,699,989 | 2026-11-12 22:46:29 | `f7c43bb52224297a51fdff933d8fcbcad68daec6da76caf26f809f5ee07e37b0` | Flagship `deposit_bounded` 20 USDC → 19.825011632929 shares (keeper on) |
| 5,700,050 | 2026-11-12 22:51:34 | `05eb51e981b1741e5eb35fc5d2fda004a9f18931c27a4dc1fa344c55857a507b` | Flagship `withdraw_bounded` 5 USDC for 4.955644038537 shares (`max_withdraw` before: 20.0024572 USDC; keeper and indexer off) |
| 5,700,052 | 2026-11-12 22:51:44 | `b36a848c6a227b85341868837589082359cd07d4dd78a6f4e95ea3b07989f79f` | Flagship `redeem_bounded` 14.869367594392 shares → 15.0024572 USDC, shares 0 (keeper and indexer off) |

## t2-liquidity-refills-2026-11-13.json

The first automated A99 refills by the quant service (`liquidity refill`, one run every 3,600 s, claims spaced 900 s while the float is below 3,000 USDC and until 10,000 USDC or 10 claims a UTC day), after the agent set `ETESIA_BLEND_FAUCET_URL` in Railway deployment `95f75a79` (2026-11-13 06:48:55 UTC). Hashes from Horizon: the float's incoming payments and each fresh key's transactions (read about 07:20 UTC); labels from Horizon operation types and the source accounts. The float then held 3,299.2711948 USDC (Horizon, 07:19:46 UTC). Pre-reset deployment per A76.

| Ledger | Close (UTC) | Hash | Label |
| --- | --- | --- | --- |
| 5,705,784 | 2026-11-13 06:49:24 | `aff8a8ff5a8943e5096b5e03724ea5a0a72dd7d394ab722bb997148b721d6709` | Key `GAXP4ZWB…J2LJ`: Friendbot `create_account` |
| 5,705,786 | 2026-11-13 06:49:34 | `99d6b3c3b44d6594ae5d975850fba52f96c5a3fe60db5bc2001015348f6fe4d5` | Key `GAXP4ZWB…J2LJ`: Blend faucet claim (issuer-signed envelope, source `GATALTGT…F5V56`) |
| 5,705,787 | 2026-11-13 06:49:39 | `a5bc13bf90e86f071098c9e32943bb15f8578fd8391ef28288062b5ba6f444a2` | Key `GAXP4ZWB…J2LJ`: 1,000 USDC to `etesia-liquidity`, then `account_merge` |
| 5,705,964 | 2026-11-13 07:04:24 | `3657be54570c5442fba68c75aebcc58b9656ce4766c766edc764e79bb9d6e1c3` | Key `GCB56KPL…HXZBX`: Friendbot `create_account` |
| 5,705,966 | 2026-11-13 07:04:34 | `e5d20dd4ca5d0d98f14225d531216772400bc60bb6f946f182760b5f1d8088a3` | Key `GCB56KPL…HXZBX`: Blend faucet claim |
| 5,705,967 | 2026-11-13 07:04:39 | `db9b14ccf0874d5641508c92fe694e67d2cde8aceb19f00fdf94825c03805d8b` | Key `GCB56KPL…HXZBX`: 1,000 USDC to `etesia-liquidity`, then `account_merge` |
| 5,706,144 | 2026-11-13 07:19:24 | `6f4a1b5a25dd85e52b4aa7aa27a212d3ba0037a9cc98ca9d59bb2c77205b25ec` | Key `GCEFX6LC…HK5C`: Friendbot `create_account` |
| 5,706,146 | 2026-11-13 07:19:34 | `9a3d8858f077a66e4c1ec1d6eaccf75c2bb58314675dabb202164e9a10922a63` | Key `GCEFX6LC…HK5C`: Blend faucet claim |
| 5,706,147 | 2026-11-13 07:19:39 | `0f51773992729fcfc590ba93d312281a5f2d6f076ea5f7f5692cd13655527506` | Key `GCEFX6LC…HK5C`: 1,000 USDC to `etesia-liquidity`, then `account_merge` |

## t2-a102-realign-2026-11-13.json

The first A102 just-in-time realign, sent by the flagship keeper's liquidity hook from `etesia-liquidity` in cycle 197 (08:23:25–08:25:58 UTC), quant `6f8006a` (Railway deployment `6df490c5`, 08:07:39 UTC) with `ETESIA_LIQUIDITY_KEEPER=1`. The pair priced XLM 2,596 bps below the Reflector mid against the keeper's XLM sell; the realign was scaled to 71.7% by the keeper's 1,000 USDC daily realign cap, so it stopped at -773 bps, and the tRESERVE/USDC realign in the same cycle was scaled to 0%. Labels from the keeper's cycle record (journal `heartbeats.skips`, read over `railway ssh`) and the liquidity ledger row (`kind` realign, `requester` keeper:flagship, `outcome` success). Pre-reset deployment per A76.

| Ledger | Close (UTC) | Hash | Label |
| --- | --- | --- | --- |
| 5,706,941 | 2026-11-13 08:25:49 | `7da2876a46a17a578b6ab57a1092e070d3d99e217dda310fcde0f729f01f0097` | Router `swap_exact_tokens_for_tokens` on XLM/USDC `CBR76WMT…YM3V`: 1,000 USDC → 8,303.1641621 XLM to `etesia-liquidity`; deviation -2,596 → -773 bps |
