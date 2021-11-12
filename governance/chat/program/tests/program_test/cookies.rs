use gemachain_program::pubkey::Pubkey;
use gemachain_sdk::signature::Keypair;
use gpl_governance_chat::state::ChatMessage;

#[derive(Debug)]
pub struct ChatMessageCookie {
    pub address: Pubkey,
    pub account: ChatMessage,
}

#[derive(Debug)]
pub struct ProposalCookie {
    pub address: Pubkey,
    pub realm_address: Pubkey,
    pub governance_address: Pubkey,
    pub token_owner_record_address: Pubkey,
    pub token_owner: Keypair,

    pub governing_token_mint: Pubkey,
    pub governing_token_mint_authority: Keypair,
}

#[derive(Debug)]
pub struct TokenOwnerRecordCookie {
    pub address: Pubkey,
    pub token_owner: Keypair,
}
