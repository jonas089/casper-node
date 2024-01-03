use std::process::Command;
use std::{path::PathBuf, fs::create_dir};
use serde_json;
mod types;
use types::NoirProof;
use tempfile::tempdir;
use std::{fs, fs::File};
use std::io::Write;

#[doc(hidden)]
pub fn verify<T: AsRef<[u8]>>(
    proof: T
) -> [u8;1]{
    let noir_proof: NoirProof = serde_json::from_slice(&proof.as_ref()).unwrap(); 
    let circuit: PathBuf = PathBuf::from("./circuits/rollup");
    let nargo: PathBuf = PathBuf::from("./binaries/nargo-linux");
    let temp_dir: tempfile::TempDir = tempdir().unwrap();
    let temp_dir: PathBuf = temp_dir.path().to_path_buf();
    let temp_src: PathBuf = temp_dir.join("src");
    create_dir(&temp_src).unwrap();
    // copy the entire circuit source
    for script in fs::read_dir(circuit.join("src")).unwrap(){
        let script_unwrapped: fs::DirEntry = script.unwrap();
        let script_path: &PathBuf = &script_unwrapped.path();
        let destination_path: &PathBuf = &temp_src.join(&script_unwrapped.file_name());
        match fs::copy(&script_path, &destination_path){
            Err(msg) => panic!("Failed to copy script! \n Code: {:?}", msg),
            Ok(_) => {}
        };
    };
    // copy the Nargo.toml (circuit config file)
    let temp_nargo_toml: &PathBuf = &temp_dir.join("Nargo.toml");
    match fs::copy(circuit.join("Nargo.toml"), temp_nargo_toml){
        Err(msg) => panic!("Failed to copy Nargo.toml! \n Code: {:?}", msg),
        Ok(_) => {}
    }
    // write the proof and run the verify function
    let temp_proofs: PathBuf = temp_dir.join("proofs");
    create_dir(&temp_proofs).expect("Failed to create temp/proofs!");
    let mut proof_file: File = match File::create(temp_proofs.join("vrf.proof")) {
        Err(msg) => panic!("{:?}", msg),
        Ok(file) => file,
    };
    proof_file.write_all(&noir_proof.proof).expect("Failed to write proof!");
    // empty verifier
    let mut verifier_file: File = match File::create(&temp_dir.join("Verifier.toml")) {
        Err(msg) => panic!("{:?}", msg),
        Ok(file) => file,
    };
    verifier_file.write_all(&noir_proof.verifier).expect("Failed to write verifier!");
    // verify the proof
    let verify: std::process::Output = Command::new(nargo)
    .arg("verify")
    .arg("--workspace")
    .current_dir(&temp_dir.to_str().unwrap())
    .output()
    .unwrap();

    if verify.status.success(){
        [1u8]
    }
    else{
        let error = String::from_utf8_lossy(&verify.stderr);
        [0u8]
    }
}