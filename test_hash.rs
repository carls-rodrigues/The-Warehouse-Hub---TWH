use bcrypt::{hash, verify, DEFAULT_COST};

fn main() {
    let password = "password";
    let hash_result = hash(password, DEFAULT_COST).unwrap();
    println!("Generated hash: {}", hash_result);
    
    let stored_hash = "$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LewdBPjYQmO7K3Q7e";
    let verify_result = verify(password, stored_hash).unwrap();
    println!("Verification result: {}", verify_result);
}
