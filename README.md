Darklake Liqdation Engine POC of a POC xD

Basic User Flow 

1. User Creates an Account
2. User Deposits into thier Account
3. User opens Perp Position
    - Background health checks 
    - If Underwater -> trigger liqudation 
    - Funding Rate Adjustment
4. User closes position


Server Architecture

- Cache Layer, for now just storing some simple mappings in memory. Ideally have concurrent writes to a db
- future: add DB  
- Endpoints for user action
- Constant Health Checks based on open user positions
- 