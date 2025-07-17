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


Questions for the Team??

- should we be running 2 servers.. 1 internal and 1 'public'
    - intenral could handle internal requests like 'encrypt' aka generate random [u8;32] pointer
    - health checks
    - forwards liqdations 
    - basicaly we sperate the servers into one that needs to listen for user requests and internal

- how to handle client side encryption requests
    - zama uses zkpok.. zk proof for valid encrpytion w/o leaking plaintext input
    - i think arcium doesn use zk and if the amount u tried to encrypt, u kinda just fuck urself
    - current poc poc doesnt handle this, user inputs is public.. deal w this later
    - pub key encryption client side cud also work, we just forwards the pointer we generate??
