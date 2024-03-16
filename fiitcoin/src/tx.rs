use core::fmt;

use rsa::{
    pkcs1v15::{SigningKey, VerifyingKey},
    signature::{SignatureEncoding, Signer},
    traits::PublicKeyParts,
};
use sha2::{Digest, Sha256};

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

#[derive(Debug, Clone)]
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

#[derive(Debug)]
pub struct UnsignedTx {
    inputs: Vec<Input>,
    outputs: Vec<Output>,
}

impl UnsignedTx {
    pub fn new() -> Self {
        Self {
            inputs: vec![],
            outputs: vec![],
        }
    }

    pub fn sing_inputs_and_finalize(
        mut self,
        sender_sk: &SigningKey<Sha256>,
    ) -> Result<Tx, TxError> {
        let mut signatures = vec![];
        for idx in 0..self.inputs.len() {
            let idx = match idx.try_into() {
                Ok(i) => i,
                Err(_) => return Err(TxError::DownCastFromUsize(idx)),
            };

            let raw_tx_one_input = raw_tx_from_one_input(&self.inputs, &self.outputs, idx)?;
            let signature = sender_sk.sign(&raw_tx_one_input).to_bytes();
            signatures.push(signature);
        }
        let signatures_len = signatures.len();
        for (idx, sig) in signatures.into_iter().enumerate() {
            match self.inputs.get_mut(usize::from(idx)) {
                Some(input) => input.signature = Some(sig),
                None => return Err(TxError::InputIndexOutOfBounds(idx, signatures_len)),
            }
        }

        self.finalize()
    }

    /// Finalizes this transaction by internally hashing its contents and returning finalized Tx
    pub fn finalize(self) -> Result<Tx, TxError> {
        let tx_bytes = raw_tx(&self.inputs, &self.outputs)?;
        let mut hasher = Sha256::new();
        hasher.update(tx_bytes);
        Ok(Tx {
            hash: hasher.finalize().into(),
            inputs: self.inputs,
            outputs: self.outputs,
        })
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

#[derive(Debug, Clone)]
pub struct Tx {
    hash: [u8; 32],
    inputs: Vec<Input>,
    outputs: Vec<Output>,
}

impl Tx {
    pub fn coinbase(value: u32, address: &VerifyingKey<Sha256>) -> Self {
        let mut unsigned = UnsignedTx::new();
        unsigned.add_output(value, address);
        // the unwrap is safe, because coinbase doesn't have any input,
        // so no need to sign any
        unsigned.finalize().unwrap()
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

    /// # DO NOT USE, don't use this function outside tests!
    pub fn force_signature_on_input(&mut self, idx: u8, signature: Box<[u8]>) {
        let input = self.inputs.get_mut(usize::from(idx)).unwrap();
        input.signature = Some(signature);
    }
}

pub fn raw_tx_from_one_input(
    inputs: &Vec<Input>,
    outputs: &Vec<Output>,
    idx: u8,
) -> Result<Vec<u8>, TxError> {
    let input = match inputs.get(usize::from(idx)) {
        Some(inp) => inp,
        None => {
            return Err(TxError::InputIndexOutOfBounds(
                usize::from(idx),
                inputs.len(),
            ))
        }
    };

    let mut tx = vec![];
    tx.extend(input.output_tx_hash);
    tx.push(input.output_idx);

    for output in outputs.iter() {
        tx.extend(output.value.to_be_bytes());
        tx.extend(output.verifying_key.as_ref().e().to_bytes_be());
        tx.extend(output.verifying_key.as_ref().n().to_bytes_be());
    }

    Ok(tx)
}

/// Returns representation of this transaction in bytes
pub fn raw_tx(inputs: &Vec<Input>, outputs: &Vec<Output>) -> Result<Vec<u8>, TxError> {
    let mut tx = vec![];

    for input in inputs.iter() {
        match &input.signature {
            Some(sig) => {
                tx.extend(input.output_tx_hash);
                tx.push(input.output_idx);
                tx.extend(sig.iter());
            }
            None => return Err(TxError::UnsignedInput(input.clone())),
        }
    }
    for output in outputs.iter() {
        tx.extend(output.value.to_be_bytes());
        tx.extend(output.verifying_key.as_ref().e().to_bytes_be());
        tx.extend(output.verifying_key.as_ref().n().to_bytes_be());
    }

    Ok(tx)
}

#[derive(Debug)]
pub enum TxError {
    UnsignedInput(Input),
    InputIndexOutOfBounds(usize, usize),
    DownCastFromUsize(usize),
}

impl fmt::Display for TxError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TxError::UnsignedInput(input) => write!(f, "unsigned input {:?}", input),
            TxError::InputIndexOutOfBounds(idx, max) => {
                write!(f, "tried to access idx {}, max is {}", idx, max)
            }
            TxError::DownCastFromUsize(u) => write!(f, "failed to downcast usize {} to u8", u),
        }
    }
}
impl std::error::Error for TxError {}
