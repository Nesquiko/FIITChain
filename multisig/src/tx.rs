use rsa::{
    pkcs1v15::{Signature, SigningKey, VerifyingKey},
    signature::Signer,
    traits::PublicKeyParts,
};

use sha2::{Digest, Sha256};

pub type Hash = [u8; 32];

pub struct UnsignedTx {
    inputs: Vec<UnsignedInput>,
    outputs: Vec<Output>,
}

impl UnsignedTx {
    pub fn new() -> Self {
        Self {
            inputs: vec![],
            outputs: vec![],
        }
    }

    pub fn finalize(self, signers: Vec<&SigningKey<Sha256>>) -> Tx {
        let raw_tx = self.raw_tx();
        let mut inputs = vec![];
        for input in self.inputs.iter() {
            let mut signatures = vec![];
            for signer in signers.iter() {
                let signature = signer.sign(&raw_tx);
                signatures.push(signature);
            }
            inputs.push(Input {
                output_tx_hash: input.output_tx_hash,
                output_idx: input.output_idx,
                signatures,
            });
        }

        let mut hasher = Sha256::new();
        hasher.update(raw_tx);
        let hash = hasher.finalize().into();
        Tx {
            hash,
            inputs,
            outputs: self.outputs,
        }
    }

    /// Returns representation of this transaction in bytes
    fn raw_tx(&self) -> Vec<u8> {
        let mut tx = vec![];
        for input in self.inputs.iter() {
            tx.extend(input.output_tx_hash);
            tx.push(input.output_idx);
        }
        for output in self.outputs.iter() {
            tx.extend(output.value.to_be_bytes());
            for verifying_key in output.verifiers.iter() {
                tx.extend(verifying_key.as_ref().e().to_bytes_be());
                tx.extend(verifying_key.as_ref().n().to_bytes_be());
            }
        }
        tx
    }

    pub fn add_input(&mut self, output_tx_hash: Hash, output_idx: u8) {
        self.inputs.push(UnsignedInput {
            output_tx_hash,
            output_idx,
        })
    }

    pub fn add_output(
        &mut self,
        value: u32,
        verifiers: Vec<&VerifyingKey<Sha256>>,
        threshold: usize,
    ) {
        let verifiers = verifiers.into_iter().map(|v| v.clone()).collect();
        self.outputs.push({
            Output {
                value,
                verifiers,
                threshold,
            }
        });
    }
}

#[derive(Debug, Clone)]
pub struct Tx {
    hash: Hash,
    inputs: Vec<Input>,
    outputs: Vec<Output>,
}

impl Tx {
    pub fn coinbase(value: u32, address: Vec<&VerifyingKey<Sha256>>, threshold: usize) -> Self {
        let mut unsigned = UnsignedTx::new();
        unsigned.add_output(value, address, threshold);
        // coinbase txs don't have inputs, so no signers are needed
        unsigned.finalize(vec![])
    }

    pub fn hash(&self) -> [u8; 32] {
        self.hash
    }

    pub fn output(&self, idx: u8) -> Option<&Output> {
        self.outputs.get(usize::from(idx))
    }

    pub fn output_len(&self) -> usize {
        self.outputs.len()
    }

    pub fn inputs(&self) -> &Vec<Input> {
        &self.inputs
    }

    pub fn outputs(&self) -> &Vec<Output> {
        &self.outputs
    }

    /// Returns representation of this transaction in bytes
    pub fn raw_tx(&self) -> Vec<u8> {
        let mut tx = vec![];
        for input in self.inputs.iter() {
            tx.extend(input.output_tx_hash);
            tx.push(input.output_idx);
        }
        for output in self.outputs.iter() {
            tx.extend(output.value.to_be_bytes());
            for verifying_key in output.verifiers.iter() {
                tx.extend(verifying_key.as_ref().e().to_bytes_be());
                tx.extend(verifying_key.as_ref().n().to_bytes_be());
            }
        }
        tx
    }
}

#[derive(Debug, Clone)]
pub struct UnsignedInput {
    /// Hash of tx, of which output is transformed into this input
    output_tx_hash: Hash,
    /// Index of the output in tx
    output_idx: u8,
}

#[derive(Debug, Clone)]
pub struct Input {
    /// Hash of tx, of which output is transformed into this input
    output_tx_hash: Hash,
    /// Index of the output in tx
    output_idx: u8,
    /// Signature created by signing whole current transaction with
    /// private key corresponding to the output's public key
    signatures: Vec<Signature>,
}

impl Input {
    pub fn new(output_tx_hash: Hash, output_idx: u8, signatures: Vec<Signature>) -> Self {
        Self {
            output_tx_hash,
            output_idx,
            signatures,
        }
    }

    pub fn output_tx_hash(&self) -> [u8; 32] {
        self.output_tx_hash
    }

    pub fn output_idx(&self) -> u8 {
        self.output_idx
    }

    pub fn signatures(&self) -> &Vec<Signature> {
        &self.signatures
    }
}

#[derive(Debug, Clone)]
pub struct Output {
    /// Value of this output
    value: u32,
    /// List of multisig owners of this Output
    verifiers: Vec<VerifyingKey<Sha256>>,
    /// How many owners must are needed to unlock this Output
    threshold: usize,
}

impl Output {
    pub fn new(value: u32, verifier: VerifyingKey<Sha256>) -> Self {
        Self {
            value,
            verifiers: vec![verifier],
            threshold: 1,
        }
    }

    pub fn multisig(value: u32, verifiers: Vec<VerifyingKey<Sha256>>, threshold: usize) -> Self {
        Self {
            value,
            verifiers,
            threshold,
        }
    }

    pub fn value(&self) -> u32 {
        self.value
    }

    pub fn verifiers(&self) -> &Vec<VerifyingKey<Sha256>> {
        &self.verifiers
    }

    pub fn threshold(&self) -> usize {
        self.threshold
    }
}
