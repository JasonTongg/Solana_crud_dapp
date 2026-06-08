#[cfg(test)]
mod tests {
    use crate::ID as PROGRAM_ID;
    use litesvm::LiteSVM;
    use solana_sdk::{
        instruction::{AccountMeta, Instruction},
        pubkey::Pubkey,
        signature::Keypair,
        signer::Signer,
        transaction::Transaction,
    };
    use solana_sdk::pubkey;
    use anchor_lang::AccountDeserialize;
    use crate::JouralEntryState;
    
    const SYSTEM_PROGRAM_ID: Pubkey = pubkey!("11111111111111111111111111111111");
    const LAMPORTS_PER_SOL: u64 = 1_000_000_000;

    fn send_tx(svm: &mut LiteSVM, user: &Keypair, ix: Instruction) -> bool {
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&user.pubkey()),
            &[user],
            svm.latest_blockhash(),
        );
        svm.send_transaction(tx).is_ok()
    }

    fn setup() -> (LiteSVM, Keypair, Keypair) {
        let mut svm = LiteSVM::new();
        let program_bytes = include_bytes!("../../../target/deploy/crud.so");
        svm.add_program(PROGRAM_ID, program_bytes).unwrap();

        let user = Keypair::new();
        let attacker = Keypair::new();
        svm.airdrop(&user.pubkey(), 10 * LAMPORTS_PER_SOL).unwrap();
        svm.airdrop(&attacker.pubkey(), 10 * LAMPORTS_PER_SOL).unwrap();

        (svm, user, attacker)
    }

    fn get_journal_entry_pda(title: String, owner: Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[title.as_bytes(), owner.as_ref()],
            &PROGRAM_ID,
        )
    }

    fn create_initialize_journal_entry_ix(
        signer: &Pubkey,
        pda: &Pubkey,
        title: &str,
        message: &str,
    ) -> Instruction {
        let discriminator: [u8; 8] = [
            48,
            65,
            201,
            186,
            25,
            41,
            127,
            0
        ];

        let mut data = discriminator.to_vec();

        let title_bytes = title.as_bytes();
        data.extend_from_slice(&(title_bytes.len() as u32).to_le_bytes());
        data.extend_from_slice(title_bytes);

        let message_bytes = message.as_bytes();
        data.extend_from_slice(&(message_bytes.len() as u32).to_le_bytes());
        data.extend_from_slice(message_bytes);

        Instruction {
            program_id: PROGRAM_ID,
            accounts: vec![
                AccountMeta::new(*pda, false),
                AccountMeta::new(*signer, true),
                AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
            ],
            data,
        }
    }

    fn update_journal_entry_ix(
        signer: &Pubkey,
        pda: &Pubkey,
        title: &str,
        message: &str,
    ) -> Instruction {
        let discriminator: [u8; 8] = [
            113,
            164,
            49,
            62,
            43,
            83,
            194,
            172
        ];

        let mut data = discriminator.to_vec();

        let title_bytes = title.as_bytes();
        data.extend_from_slice(&(title_bytes.len() as u32).to_le_bytes());
        data.extend_from_slice(title_bytes);

        let message_bytes = message.as_bytes();
        data.extend_from_slice(&(message_bytes.len() as u32).to_le_bytes());
        data.extend_from_slice(message_bytes);

        Instruction {
            program_id: PROGRAM_ID,
            accounts: vec![
                AccountMeta::new(*pda, false),
                AccountMeta::new(*signer, true),
                AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
            ],
            data,
        }
    }

    fn delete_journal_entry_ix(
        signer: &Pubkey,
        pda: &Pubkey,
        title: &str
    ) -> Instruction {
        let discriminator: [u8; 8] = [
            156,
            50,
            93,
            5,
            157,
            97,
            188,
            114
        ];

        let mut data = discriminator.to_vec();

        let title_bytes = title.as_bytes();
        data.extend_from_slice(&(title_bytes.len() as u32).to_le_bytes());
        data.extend_from_slice(title_bytes);

        Instruction {
            program_id: PROGRAM_ID,
            accounts: vec![
                AccountMeta::new(*pda, false),
                AccountMeta::new(*signer, true)
            ],
            data,
        }
    }

    fn account_exists(svm: &LiteSVM, pda: &Pubkey) -> bool {
        svm.get_account(pda).is_some()
    }

    fn read_journal_entry(svm: &LiteSVM, pda: &Pubkey) -> JouralEntryState {
        let account = svm.get_account(pda).unwrap();
        JouralEntryState::try_deserialize(&mut &account.data[..]).unwrap()
    }

    #[test]
    fn success_create() {
        let (mut svm, user, _) = setup();
        let (pda,_) = get_journal_entry_pda("title".to_string(), user.pubkey());
        
        let ix = create_initialize_journal_entry_ix(&user.pubkey(), &pda, "title", "message");

        assert!(send_tx(&mut svm, &user, ix));

        let created_journal = read_journal_entry(&svm, &pda);

        assert_eq!(created_journal.owner, user.pubkey());
        assert_eq!(created_journal.title, "title");
        assert_eq!(created_journal.message, "message");
    }
    
    #[test]
    fn failed_create_message_to_long() {
        let (mut svm, user, _) = setup();
        let (pda,_) = get_journal_entry_pda("title".to_string(), user.pubkey());
        
        let ix = create_initialize_journal_entry_ix(&user.pubkey(), &pda, "title", "a".repeat(129).as_ref());

        assert!(!send_tx(&mut svm, &user, ix));
    }
    
    #[test]
    fn failed_create_title_to_long() {
        let (mut svm, user, _) = setup();
        let long_title = "a".repeat(33);
        let (pda,_) = get_journal_entry_pda(long_title.clone(), user.pubkey());

        let ix = create_initialize_journal_entry_ix(&user.pubkey(), &pda, &long_title, "message");

        assert!(!send_tx(&mut svm, &user, ix));
    }

    #[test]
    fn success_update() {
        let (mut svm, user, _) = setup();
        let (pda,_) = get_journal_entry_pda("title".to_string(), user.pubkey());

        let before_lamport = svm.get_account(&user.pubkey()).unwrap().lamports;
        
        let ix = create_initialize_journal_entry_ix(&user.pubkey(), &pda, "title", "message");

        assert!(send_tx(&mut svm, &user, ix));

        let after_create_lamport = svm.get_account(&user.pubkey()).unwrap().lamports;

        let created_journal = read_journal_entry(&svm, &pda);

        assert_eq!(created_journal.owner, user.pubkey());
        assert_eq!(created_journal.title, "title");
        assert_eq!(created_journal.message, "message");

        let ix_update = update_journal_entry_ix(&user.pubkey(), &pda, "title", "message123");

        assert!(send_tx(&mut svm, &user, ix_update));

        let after_update_lamport = svm.get_account(&user.pubkey()).unwrap().lamports;

        let created_journal2 = read_journal_entry(&svm, &pda);

        assert_eq!(created_journal2.owner, user.pubkey());
        assert_eq!(created_journal2.title, "title");
        assert_eq!(created_journal2.message, "message123");
        assert!(after_create_lamport < before_lamport);
        assert!(after_update_lamport < after_create_lamport);
    }
    
    #[test]
    fn failed_update_message_to_long() {
        let (mut svm, user, _) = setup();
        let (pda,_) = get_journal_entry_pda("title".to_string(), user.pubkey());

        let before_lamport = svm.get_account(&user.pubkey()).unwrap().lamports;
        
        let ix = create_initialize_journal_entry_ix(&user.pubkey(), &pda, "title", "message");

        assert!(send_tx(&mut svm, &user, ix));

        let after_create_lamport = svm.get_account(&user.pubkey()).unwrap().lamports;

        let created_journal = read_journal_entry(&svm, &pda);

        assert_eq!(created_journal.owner, user.pubkey());
        assert_eq!(created_journal.title, "title");
        assert_eq!(created_journal.message, "message");

        let ix_update = update_journal_entry_ix(&user.pubkey(), &pda, "title", "a".repeat(129).as_ref());

        assert!(!send_tx(&mut svm, &user, ix_update));

        let after_update_lamport = svm.get_account(&user.pubkey()).unwrap().lamports;

        let created_journal2 = read_journal_entry(&svm, &pda);

        assert_eq!(created_journal2.owner, user.pubkey());
        assert_eq!(created_journal2.title, "title");
        assert_eq!(created_journal2.message, "message");
        assert!(after_create_lamport < before_lamport);
        assert!(after_update_lamport < after_create_lamport);
    }
    
    #[test]
    fn failed_update_wrong_owner() {
        let (mut svm, user, attacker) = setup();
        let (pda,_) = get_journal_entry_pda("title".to_string(), user.pubkey());
        
        let ix = create_initialize_journal_entry_ix(&user.pubkey(), &pda, "title", "message");

        assert!(send_tx(&mut svm, &user, ix));

        let created_journal = read_journal_entry(&svm, &pda);

        assert_eq!(created_journal.owner, user.pubkey());
        assert_eq!(created_journal.title, "title");
        assert_eq!(created_journal.message, "message");

        let ix_update = update_journal_entry_ix(&attacker.pubkey(), &pda, "title", "message123");

        assert!(!send_tx(&mut svm, &attacker, ix_update));

        let created_journal2 = read_journal_entry(&svm, &pda);

        assert_eq!(created_journal2.owner, user.pubkey());
        assert_eq!(created_journal2.title, "title");
        assert_eq!(created_journal2.message, "message");
    }

    #[test]
    fn success_delete() {
        let (mut svm, user, _) = setup();
        let (pda,_) = get_journal_entry_pda("title".to_string(), user.pubkey());

        let before_lamport = svm.get_account(&user.pubkey()).unwrap().lamports;
        
        let ix = create_initialize_journal_entry_ix(&user.pubkey(), &pda, "title", "message");

        assert!(send_tx(&mut svm, &user, ix));

        let after_create_lamport = svm.get_account(&user.pubkey()).unwrap().lamports;

        let created_journal = read_journal_entry(&svm, &pda);

        assert_eq!(created_journal.owner, user.pubkey());
        assert_eq!(created_journal.title, "title");
        assert_eq!(created_journal.message, "message");

        let ix_delete = delete_journal_entry_ix(&user.pubkey(), &pda, "title");

        assert!(send_tx(&mut svm, &user, ix_delete));

        let after_delete_lamport = svm.get_account(&user.pubkey()).unwrap().lamports;

        assert!(!account_exists(&svm, &pda));
        assert!(after_create_lamport < before_lamport);
        assert!(after_delete_lamport > after_create_lamport);
    }

    #[test]
    fn failed_delete_wrong_owner() {
        let (mut svm, user, attacker) = setup();
        let (pda,_) = get_journal_entry_pda("title".to_string(), user.pubkey());
        
        let ix = create_initialize_journal_entry_ix(&user.pubkey(), &pda, "title", "message");

        assert!(send_tx(&mut svm, &user, ix));

        let created_journal = read_journal_entry(&svm, &pda);

        assert_eq!(created_journal.owner, user.pubkey());
        assert_eq!(created_journal.title, "title");
        assert_eq!(created_journal.message, "message");

        let ix_delete = delete_journal_entry_ix(&attacker.pubkey(), &pda, "title");

        assert!(!send_tx(&mut svm, &attacker, ix_delete));
        assert!(account_exists(&svm, &pda));
    }

    #[test]
    fn failed_create_duplicate() {
        let (mut svm, user, _) = setup();
        let (pda,_) = get_journal_entry_pda("title".to_string(), user.pubkey());
        
        let ix = create_initialize_journal_entry_ix(&user.pubkey(), &pda, "title", "message");

        assert!(send_tx(&mut svm, &user, ix));

        let created_journal = read_journal_entry(&svm, &pda);

        assert_eq!(created_journal.owner, user.pubkey());
        assert_eq!(created_journal.title, "title");
        assert_eq!(created_journal.message, "message");

        let ix2 = create_initialize_journal_entry_ix(&user.pubkey(), &pda, "title", "message2");

        assert!(!send_tx(&mut svm, &user, ix2));

        let created_journal = read_journal_entry(&svm, &pda);

        assert_eq!(created_journal.owner, user.pubkey());
        assert_eq!(created_journal.title, "title");
        assert_eq!(created_journal.message, "message");
    }
}