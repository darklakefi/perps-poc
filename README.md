Darklake Liqdation Engine POC of a POC xD

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

Food for Thought:

- Why do we have an in-momory 'cache'
    - AFAIK general practice for FHE coprocessing requires the actualy Compresseed and serlized Ciphertexts to be stored in a DB (maybe even written to a DA)
    - The cache allows us to refrence ciphertexts directly in their un-compressed state (FheUint64 in this case) wihtout having to deal with compression and deserlization 
    - Useful for ciphertexts that don't need to persist. 
    - Again the idea is that everytime we need to write or update a ciphertext, we spin up some sort of async thread/worker to handle the db write process (compress and serlize)

