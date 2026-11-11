# Tranche 2 testnet receipt archive

Envelope, result, meta and diagnostic-event XDR of every Tranche 2 testnet transaction not already archived with its XDR, recorded before SDF testnet RPC retention (120,960 ledgers, about 7 days) drops them. All are in the pre-2026-12-16-reset epoch ([A76](../../../doc/wiki/decisions.md)).

- **Command:** `python3 scripts/record-evidence.py --testnet docs/evidence/<file> <hash>...`, one run per file, on 2026-11-11 (15:38 UTC; `t2-d4-10-step2b.json` at 15:49 UTC). Each run asserts the RPC passphrase and `SUCCESS` for every hash.
- **Endpoint:** `https://soroban-testnet.stellar.org`. **Manifest:** `deploy/testnet/pre-2026-12-16-reset.json`, SHA-256 `cfa30d48adb6c5559dfbed446a4a0763c67ed0672305b18910e493b1920c2220`.
- **Hashes:** resolved in full from Horizon operations of the flagship executor `GDGCL46K…JT7N`, `etesia-admin` `GAQKH3PL…DUVY`, `etesia-realign` `GDRVYVG3…ZM3R`, `etesia-v3-treasury`, the guard keys and the faucet wallets, from ledger 5,650,000 (failed included). The only failed one is the D4-02 failed swap, archived elsewhere (below).
- **Check:** every archived envelope hashes (testnet passphrase) to its recorded hash, and every result and meta XDR decodes (stellar-sdk, 166 of 166). Execute actions below are read from the meta's contract events.

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

**Archived elsewhere, not repeated:** D4-02 direct proof `e58ed4546ea00938b83f512ba8fec968e3b5e994e0010d6fb4420f2900991e35` and failed swap `4c55ed60dc50e7eaf29d6a0cd9bfe47d2aaf085e8eb641a236826103637c37bf` (FAILED by design) in `d4-testnet-probes.json` keys `direct_proof` and `failed_swap`; flagship deploy `35ca57046571f19315aa19fbb20f864328c0d04735d8034dac03f77b153b614d`, D4-03 publish `d89fe72583d47db84521c19c30a087bd2928da9a7f4efc028fd8dfd25e05630b`, D4-04 publish `f76c3c1a0e6eba91a3c7b12af3e5b8bc011f4e420bc0b012570ba122d68d09a5`, D4-05 publish `d24e9f3363fb107038be5c477f80f895109dfe12ddc40ce0f12261844eb380f9` and execute `7b7bf088079423e92fc5c3df2f31683fd11a8b4ae408a08bb81a0bd4203fd408` in `fixtures/d5-04-testnet/00`–`05`.

**Not archived:** the third-party v3 copy deploy `a383fe94…c91e` (2026-10-16, before Tranche 2 and outside retention). No D4-09 guard case has been sent yet. Records are written once, so later D4-10 step 2 or keeper transactions go in a new file, recorded before about 2026-11-18 (retention).

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
