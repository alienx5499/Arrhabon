//! Signing wallet transactions with a local keypair: the demo attestor and the devnet CLI.

use anchor_client::Signer;
use anchor_client::compat::solana_transaction::Transaction;
use arrhabon::state::id;
use arrhabon_app::PortError;
use arrhabon_app::ports::{AttestorSigner, PortResult, SignedTx, UnsignedTx};
use arrhabon_domain::AccountId;
use solana_keypair::Keypair;

/// Adds `signer`'s signature to an unsigned transaction built for it.
pub fn sign_tx(tx: &UnsignedTx, signer: &Keypair) -> PortResult<SignedTx> {
    let mut decoded: Transaction = bincode::deserialize(&tx.bytes)
        .map_err(|_| PortError::rejected("not a serialized legacy transaction"))?;
    let blockhash = decoded.message.recent_blockhash;
    decoded
        .try_partial_sign(&[signer], blockhash)
        .map_err(|e| PortError::rejected(e.to_string()))?;
    let bytes = bincode::serialize(&decoded).expect("a Transaction always serializes");
    Ok(SignedTx(bytes))
}

/// The clearly labelled demo attestor: a project keypair, never a production signer.
pub struct KeypairAttestor(pub Keypair);

impl AttestorSigner for KeypairAttestor {
    fn attestor(&self) -> AccountId {
        id(&self.0.pubkey())
    }

    async fn sign(&self, tx: UnsignedTx) -> PortResult<SignedTx> {
        sign_tx(&tx, &self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::instructions::{deny_visa, unsigned_tx};
    use anchor_client::compat::solana_transaction::Hash;

    #[test]
    fn signing_fills_the_required_signature() {
        let attestor = Keypair::new();
        let ix = deny_visa(crate::accounts::tests::pk(9), attestor.pubkey());
        let bytes = unsigned_tx(&[ix], &attestor.pubkey(), Hash::new_from_array([1; 32]));
        let tx = UnsignedTx {
            bytes,
            last_valid_block_height: 10,
        };
        let signed = sign_tx(&tx, &attestor).unwrap();
        let decoded: Transaction = bincode::deserialize(&signed.0).unwrap();
        assert!(decoded.is_signed());
        assert_eq!(
            decoded.signatures[0],
            attestor.sign_message(&decoded.message_data())
        );
    }

    #[test]
    fn a_stranger_cannot_sign() {
        let attestor = Keypair::new();
        let ix = deny_visa(crate::accounts::tests::pk(9), attestor.pubkey());
        let bytes = unsigned_tx(&[ix], &attestor.pubkey(), Hash::new_from_array([1; 32]));
        let tx = UnsignedTx {
            bytes,
            last_valid_block_height: 10,
        };
        assert!(sign_tx(&tx, &Keypair::new()).is_err());
    }
}
