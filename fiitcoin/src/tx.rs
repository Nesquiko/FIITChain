use core::fmt;

use rsa::{
    pkcs1v15::{SigningKey, VerifyingKey},
    signature::{SignatureEncoding, Signer},
    traits::PublicKeyParts,
};
use sha2::{Digest, Sha256};

pub struct Incomplete;
pub type Finalized = [u8; 32];

#[derive(Debug)]
pub struct Tx<S> {
    hash: S,
    inputs: Vec<Input>,
    outputs: Vec<Output>,
}

#[derive(Debug, Clone)]
pub struct Input {
    /// Hash of tx, of which output is transformed into this input
    output_tx_hash: [u8; 32],
    /// Index of the output in tx
    output_idx: u8,
    /// Signature created by signing whole current transaction with
    /// private key corresponding to the output's public key
    signature: Option<Box<[u8]>>,
}

impl Input {
    pub fn output_tx_hash(&self) -> [u8; 32] {
        self.output_tx_hash
    }

    pub fn output_idx(&self) -> u8 {
        self.output_idx
    }

    pub fn signature(&self) -> Option<&Box<[u8]>> {
        self.signature.as_ref()
    }
}

#[derive(Debug)]
pub struct Output {
    /// Value of this output in FIITCoins
    value: u32,
    verifying_key: VerifyingKey<Sha256>,
}

impl Output {
    pub fn verifying_key(&self) -> &VerifyingKey<Sha256> {
        &self.verifying_key
    }

    pub fn value(&self) -> u32 {
        self.value
    }
}

impl<S> Tx<S> {
    /// Returns representation of this transaction in bytes
    pub fn raw_tx(&self) -> Result<Vec<u8>, TxError> {
        let mut tx = vec![];

        for input in self.inputs.iter() {
            match &input.signature {
                Some(sig) => {
                    tx.extend(input.output_tx_hash);
                    tx.push(input.output_idx);
                    tx.extend(sig.iter());
                }
                None => return Err(TxError::UnsignedInput(input.clone())),
            }
        }
        for output in self.outputs.iter() {
            tx.extend(output.value.to_be_bytes());
            tx.extend(output.verifying_key.as_ref().e().to_bytes_be());
            tx.extend(output.verifying_key.as_ref().n().to_bytes_be());
        }

        Ok(tx)
    }

    pub fn raw_tx_from_one_input(&self, idx: u8) -> Result<Vec<u8>, TxError> {
        let input = match self.inputs.get(usize::from(idx)) {
            Some(inp) => inp,
            None => return Err(TxError::InputIndexOutOfBounds(idx)),
        };

        let mut tx = vec![];
        tx.extend(input.output_tx_hash);
        tx.push(input.output_idx);

        for output in self.outputs.iter() {
            tx.extend(output.value.to_be_bytes());
            tx.extend(output.verifying_key.as_ref().e().to_bytes_be());
            tx.extend(output.verifying_key.as_ref().n().to_bytes_be());
        }

        Ok(tx)
    }
}

impl Tx<Incomplete> {
    pub fn new() -> Self {
        Self {
            hash: Incomplete {},
            inputs: vec![],
            outputs: vec![],
        }
    }

    pub fn sing_input_and_finalize(
        mut self,
        idx: u8,
        sender_sk: &SigningKey<Sha256>,
    ) -> Result<Tx<Finalized>, TxError> {
        self.sign_input(idx, sender_sk)?;
        self.finalize()
    }

    /// Finalizes this transaction by internally hashing its contents and returning finalized Tx
    pub fn finalize(self) -> Result<Tx<Finalized>, TxError> {
        let tx_bytes = self.raw_tx()?;
        let mut hasher = Sha256::new();
        hasher.update(tx_bytes);
        Ok(Tx {
            hash: hasher.finalize().into(),
            inputs: self.inputs,
            outputs: self.outputs,
        })
    }

    /// Creates signature from input on idx and all outputs, and sets signature
    /// on that Input to the created one
    pub fn sign_input(&mut self, idx: u8, sk: &SigningKey<Sha256>) -> Result<(), TxError> {
        let raw_tx_one_input = self.raw_tx_from_one_input(idx)?;
        let signature = sk.sign(&raw_tx_one_input).to_bytes();

        match self.inputs.get_mut(usize::from(idx)) {
            Some(input) => input.signature = Some(signature),
            None => return Err(TxError::InputIndexOutOfBounds(idx)),
        }

        Ok(())
    }

    pub fn add_input(&mut self, output_tx_hash: [u8; 32], output_idx: u8) {
        self.inputs.push(Input {
            output_tx_hash,
            output_idx,
            signature: None,
        })
    }

    pub fn add_output(&mut self, value: u32, receiver_verifying_key: &VerifyingKey<Sha256>) {
        self.outputs.push(Output {
            value,
            verifying_key: receiver_verifying_key.clone(),
        });
    }
}

impl Tx<Finalized> {
    pub fn hash(&self) -> [u8; 32] {
        self.hash
    }

    pub fn output(&self, idx: u8) -> Option<&Output> {
        self.outputs.get(usize::from(idx))
    }

    pub fn inputs(&self) -> &Vec<Input> {
        &self.inputs
    }

    pub fn outputs(&self) -> &Vec<Output> {
        &self.outputs
    }
}

#[derive(Debug)]
pub enum TxError {
    UnsignedInput(Input),
    InputIndexOutOfBounds(u8),
}

impl fmt::Display for TxError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TxError::UnsignedInput(input) => write!(f, "unsigned input {:?}", input),
            TxError::InputIndexOutOfBounds(idx) => write!(f, "index {} out of bounds", idx),
        }
    }
}

impl std::error::Error for TxError {}
