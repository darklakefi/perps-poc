Darklake Liquidation Engine ZK POC

Basic User Flow 

1. User Creates an Account
2. User Deposits into thier Account
3. User opens Perp Position
    - Background health checks 
    - If Underwater -> trigger liqudation 
    - Funding Rate Adjustment
4. User closes position
5. Withdraw

Server Architecture

- Cache Layer, for now just storing some simple mappings in memory. Ideally have concurrent writes to a db
- future: add DB  
- Endpoints for user action
- Constant Health Checks based on open user positions


Food for Thought:

- Why do we have an in-momory 'cache'
    - In the ZK version the cache holds user balance commitments and open positions, not encrypted state.
    - The long-term version should persist commitments, positions, and proof metadata in a DB or DA-backed store.
    - This POC keeps that state in memory so the proof-backed flows are easy to test end to end.

- We (I) need to think about how to optimize the health check/funding engine
    - should we make it so that smaller assets have a tighter spread, as price flucations lead to many attacks

- Handling quanitity adjustments
    - intial poc of poc just giga happy path assums that 1 unit quantity. ex: 30k notional and entry price of 30k
    - ideally it should be optimized for whatever is the least amount of compute. maybe quanitity adjustments can just be done in plaintext. 
