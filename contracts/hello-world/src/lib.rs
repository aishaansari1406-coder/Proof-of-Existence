/* #![no_std]
use soroban_sdk::{contract, contractimpl, vec, Env, String, Vec};

#[contract]
pub struct Contract;

// This is a sample contract. Replace this placeholder with your own contract logic.
// A corresponding test example is available in `test.rs`.
//
// For comprehensive examples, visit <https://github.com/stellar/soroban-examples>.
// The repository includes use cases for the Stellar ecosystem, such as data storage on
// the blockchain, token swaps, liquidity pools, and more.
//
// Refer to the official documentation:
// <https://developers.stellar.org/docs/build/smart-contracts/overview>.
#[contractimpl]
impl Contract {
    pub fn hello(env: Env, to: String) -> Vec<String> {
        vec![&env, String::from_str(&env, "Hello"), to]
    }
}


mod test; */
#![allow(non_snake_case)]
#![no_std]
use soroban_sdk::{contract, contracttype, contractimpl, log, Env, Symbol, String, symbol_short};

// Structure to store document proof information
#[contracttype]
#[derive(Clone)]
pub struct DocumentProof {
    pub doc_hash: String,        // SHA-256 hash of the document
    pub owner: String,            // Owner/creator address
    pub timestamp: u64,           // Timestamp when document was registered
    pub description: String,      // Optional description of the document
}

// Structure to track overall statistics
#[contracttype]
#[derive(Clone)]
pub struct ProofStats {
    pub total_documents: u64,     // Total number of documents registered
}

// Symbol for storing overall statistics
const STATS: Symbol = symbol_short!("STATS");

// Mapping document hash to its proof record
#[contracttype]
pub enum ProofBook {
    Proof(String)  // Key is the document hash
}

#[contract]
pub struct ProofOfExistenceContract;

#[contractimpl]
impl ProofOfExistenceContract {
    
    /// Register a new document proof with its hash
    /// Returns true if successfully registered, panics if already exists
    pub fn register_document(
        env: Env, 
        doc_hash: String, 
        owner: String, 
        description: String
    ) -> bool {
        
        // Check if document already exists
        let existing_proof = Self::get_proof(env.clone(), doc_hash.clone());
        
        if existing_proof.timestamp != 0 {
            log!(&env, "Document already registered at timestamp: {}", existing_proof.timestamp);
            panic!("Document hash already exists!");
        }
        
        // Get current timestamp from ledger
        let timestamp = env.ledger().timestamp();
        
        // Create new document proof
        let proof = DocumentProof {
            doc_hash: doc_hash.clone(),
            owner: owner.clone(),
            timestamp,
            description,
        };
        
        // Update statistics
        let mut stats = Self::get_stats(env.clone());
        stats.total_documents += 1;
        
        // Store the proof
        env.storage().instance().set(&ProofBook::Proof(doc_hash.clone()), &proof);
        
        // Store updated statistics
        env.storage().instance().set(&STATS, &stats);
        
        // Extend TTL for storage
        env.storage().instance().extend_ttl(17280, 17280); // ~30 days
        
        log!(&env, "Document registered successfully. Hash: {}, Timestamp: {}", doc_hash, timestamp);
        
        true
    }
    
    /// Verify if a document exists and retrieve its proof
    /// Returns DocumentProof with timestamp 0 if not found
    pub fn verify_document(env: Env, doc_hash: String) -> DocumentProof {
        let proof = Self::get_proof(env.clone(), doc_hash.clone());
        
        if proof.timestamp != 0 {
            log!(&env, "Document verified! Registered at: {}", proof.timestamp);
        } else {
            log!(&env, "Document not found in registry");
        }
        
        proof
    }
    
    /// Get proof details for a specific document hash
    pub fn get_proof(env: Env, doc_hash: String) -> DocumentProof {
        let key = ProofBook::Proof(doc_hash.clone());
        
        env.storage().instance().get(&key).unwrap_or(DocumentProof {
            doc_hash: String::from_str(&env, "NOT_FOUND"),
            owner: String::from_str(&env, ""),
            timestamp: 0,
            description: String::from_str(&env, ""),
        })
    }
    
    /// Get overall statistics of the proof system
    pub fn get_stats(env: Env) -> ProofStats {
        env.storage().instance().get(&STATS).unwrap_or(ProofStats {
            total_documents: 0,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::Env;

    #[test]
    fn test_register_and_verify() {
        let env = Env::default();
        let contract_id = env.register_contract(None, ProofOfExistenceContract);
        let client = ProofOfExistenceContractClient::new(&env, &contract_id);

        let doc_hash = String::from_str(&env, "abc123def456");
        let owner = String::from_str(&env, "owner_address_123");
        let description = String::from_str(&env, "Important document");

        // Register document
        let result = client.register_document(&doc_hash, &owner, &description);
        assert_eq!(result, true);

        // Verify document
        let proof = client.verify_document(&doc_hash);
        assert!(proof.timestamp > 0);
        assert_eq!(proof.owner, owner);
    }
}