# Tranche 2 changes note

This note lists where the delivery of Deliverables 4–6 differs from the award text. It accompanies the Tranche 2 submission.

| Award wording (Deliverable 4) | What was delivered | Reason |
| --- | --- | --- |
| "swap execution routed through the Soroswap routing API (aggregating Soroban and classic liquidity), with Phoenix as a secondary path" | Swaps are quoted by the Soroswap Aggregator API and executed through Soroswap's Aggregator v3 contract, which takes the place of the separate Phoenix path. The vault can split one swap across up to 15 routes of up to 4 hops. On mainnet Aggregator v3 routes across Soroswap, Aquarius, SushiSwap V3 and Comet; this tranche enables Soroswap pools, and further venues can be added behind the same guardrails. | One aggregator integration gives multi-route execution and keeps every hop under the vault's onchain guardrails. The Soroswap API's venues are Soroswap, Aquarius, SushiSwap V3, Comet and the classic DEX; Phoenix is not among them. |
