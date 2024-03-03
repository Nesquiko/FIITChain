use std::usize;

use rsa::{
    pkcs1v15::SigningKey,
    signature::{SignatureEncoding, Signer},
    traits::PublicKeyParts,
    RsaPublicKey,
};
use sha2::{Digest, Sha256};

pub struct Incomplete;
pub type Finalized = [u8; 32];

#[derive(Debug)]
pub struct Tx<'a, S> {
    hash: S,
    inputs: Vec<Input>,
    outputs: Vec<Output<'a>>,
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

#[derive(Debug)]
pub struct Output<'a> {
    /// Value of this output in FIITCoins
    value: u64,
    rec_pub_key: &'a RsaPublicKey,
}

impl<'a> Tx<'a, Incomplete> {
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
        sk: &SigningKey<Sha256>,
    ) -> Result<Tx<'a, Finalized>, TxError> {
        self.sign_input(idx, sk)?;
        self.finalize()
    }

    /// Finalizes this transaction by internally hashing its contents and returning finalized Tx
    pub fn finalize(self) -> Result<Tx<'a, Finalized>, TxError> {
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
            tx.extend(output.rec_pub_key.e().to_bytes_be());
            tx.extend(output.rec_pub_key.n().to_bytes_be());
        }

        Ok(tx)
    }

    pub fn add_input(&mut self, output_tx_hash: [u8; 32], output_idx: u8) {
        self.inputs.push(Input {
            output_tx_hash,
            output_idx,
            signature: None,
        })
    }

    pub fn add_output(&mut self, value: u64, rec_pub_key: &'a RsaPublicKey) {
        self.outputs.push(Output { value, rec_pub_key });
    }

    fn raw_tx_from_one_input(&self, idx: u8) -> Result<Vec<u8>, TxError> {
        let input: &Input;
        match self.inputs.get(usize::from(idx)) {
            Some(inp) => input = inp,
            None => return Err(TxError::InputIndexOutOfBounds(idx)),
        }

        let mut tx = vec![];
        match &input.signature {
            Some(sig) => {
                tx.extend(input.output_tx_hash);
                tx.push(input.output_idx);
                tx.extend(sig.iter());
            }
            None => return Err(TxError::UnsignedInput(input.clone())),
        }

        for output in self.outputs.iter() {
            tx.extend(output.value.to_be_bytes());
            tx.extend(output.rec_pub_key.e().to_bytes_be());
            tx.extend(output.rec_pub_key.n().to_bytes_be());
        }

        Ok(tx)
    }
}

impl<'a> Tx<'a, Finalized> {
    pub fn hash(&self) -> [u8; 32] {
        self.hash
    }

    pub fn output(&self, idx: u8) -> Option<&Output> {
        self.outputs.get(usize::from(idx))
    }

    pub fn inputs(&self) -> &Vec<Input> {
        &self.inputs
    }
}

#[derive(Debug)]
pub enum TxError {
    UnsignedInput(Input),
    InputIndexOutOfBounds(u8),
}
