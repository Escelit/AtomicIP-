#[cfg(test)]
mod multi_signer_tests {
    use ip_registry::{IpRegistry, IpRegistryClient};
    use soroban_sdk::{
        testutils::Address as _,
        token::StellarAssetClient,
        Address, Bytes, BytesN, Env, Vec,
    };

    use crate::{AtomicSwap, AtomicSwapClient, SwapStatus};

    // ── Helpers ───────────────────────────────────────────────────────────────

    fn setup_registry(env: &Env, owner: &Address) -> (Address, u64, BytesN<32>, BytesN<32>) {
        let registry_id = env.register(IpRegistry, ());
        let registry = IpRegistryClient::new(env, &registry_id);

        let secret = BytesN::from_array(env, &[0xAAu8; 32]);
        let blinding = BytesN::from_array(env, &[0xBBu8; 32]);

        let mut preimage = Bytes::new(env);
        preimage.append(&Bytes::from(secret.clone()));
        preimage.append(&Bytes::from(blinding.clone()));
        let commitment_hash: BytesN<32> = env.crypto().sha256(&preimage).into();

        let ip_id = registry.commit_ip(owner, &commitment_hash);
        (registry_id, ip_id, secret, blinding)
    }

    fn setup_token(env: &Env, admin: &Address, recipient: &Address, amount: i128) -> Address {
        let token_id = env
            .register_stellar_asset_contract_v2(admin.clone())
            .address();
        StellarAssetClient::new(env, &token_id).mint(recipient, &amount);
        token_id
    }

    fn setup_contract(env: &Env, registry_id: &Address) -> AtomicSwapClient {
        let contract_id = env.register(AtomicSwap, ());
        let client = AtomicSwapClient::new(env, &contract_id);
        client.initialize(registry_id);
        client
    }

    // ── Tests ─────────────────────────────────────────────────────────────────

    #[test]
    fn test_reveal_blocked_until_all_signers_sign() {
        let env = Env::default();
        env.mock_all_auths();

        let seller = Address::generate(&env);
        let co_signer = Address::generate(&env);
        let buyer = Address::generate(&env);

        let (registry_id, ip_id, secret, blinding) = setup_registry(&env, &seller);
        let token_id = setup_token(&env, &seller, &buyer, 1_000_000);
        let client = setup_contract(&env, &registry_id);

        let mut signers = Vec::new(&env);
        signers.push_back(seller.clone());
        signers.push_back(co_signer.clone());

        let swap_id = client.initiate_swap_with_signers(
            &token_id, &ip_id, &seller, &1000i128, &buyer, &signers,
        );
        client.accept_swap(&swap_id);

        // Only seller has signed — reveal must fail
        client.sign_swap_reveal(&swap_id, &seller);
        let result = client.try_reveal_key(&swap_id, &seller, &secret, &blinding);
        assert!(result.is_err(), "reveal must fail when not all signers have signed");
    }

    #[test]
    fn test_reveal_succeeds_after_all_signers_sign() {
        let env = Env::default();
        env.mock_all_auths();

        let seller = Address::generate(&env);
        let co_signer = Address::generate(&env);
        let buyer = Address::generate(&env);

        let (registry_id, ip_id, secret, blinding) = setup_registry(&env, &seller);
        let token_id = setup_token(&env, &seller, &buyer, 1_000_000);
        let client = setup_contract(&env, &registry_id);

        let mut signers = Vec::new(&env);
        signers.push_back(seller.clone());
        signers.push_back(co_signer.clone());

        let swap_id = client.initiate_swap_with_signers(
            &token_id, &ip_id, &seller, &1000i128, &buyer, &signers,
        );
        client.accept_swap(&swap_id);

        client.sign_swap_reveal(&swap_id, &seller);
        client.sign_swap_reveal(&swap_id, &co_signer);

        // All signed — reveal must succeed
        client.reveal_key(&swap_id, &seller, &secret, &blinding);

        let swap = client.get_swap(&swap_id).unwrap();
        assert_eq!(swap.status, SwapStatus::Completed);
    }

    #[test]
    fn test_non_required_signer_cannot_sign() {
        let env = Env::default();
        env.mock_all_auths();

        let seller = Address::generate(&env);
        let co_signer = Address::generate(&env);
        let outsider = Address::generate(&env);
        let buyer = Address::generate(&env);

        let (registry_id, ip_id, _, _) = setup_registry(&env, &seller);
        let token_id = setup_token(&env, &seller, &buyer, 1_000_000);
        let client = setup_contract(&env, &registry_id);

        let mut signers = Vec::new(&env);
        signers.push_back(seller.clone());
        signers.push_back(co_signer.clone());

        let swap_id = client.initiate_swap_with_signers(
            &token_id, &ip_id, &seller, &1000i128, &buyer, &signers,
        );
        client.accept_swap(&swap_id);

        let result = client.try_sign_swap_reveal(&swap_id, &outsider);
        assert!(result.is_err(), "outsider must not be able to sign");
    }

    #[test]
    fn test_duplicate_signature_rejected() {
        let env = Env::default();
        env.mock_all_auths();

        let seller = Address::generate(&env);
        let co_signer = Address::generate(&env);
        let buyer = Address::generate(&env);

        let (registry_id, ip_id, _, _) = setup_registry(&env, &seller);
        let token_id = setup_token(&env, &seller, &buyer, 1_000_000);
        let client = setup_contract(&env, &registry_id);

        let mut signers = Vec::new(&env);
        signers.push_back(seller.clone());
        signers.push_back(co_signer.clone());

        let swap_id = client.initiate_swap_with_signers(
            &token_id, &ip_id, &seller, &1000i128, &buyer, &signers,
        );
        client.accept_swap(&swap_id);

        client.sign_swap_reveal(&swap_id, &seller);
        let result = client.try_sign_swap_reveal(&swap_id, &seller);
        assert!(result.is_err(), "duplicate signature must be rejected");
    }

    #[test]
    fn test_three_signers_all_must_sign() {
        let env = Env::default();
        env.mock_all_auths();

        let seller = Address::generate(&env);
        let signer2 = Address::generate(&env);
        let signer3 = Address::generate(&env);
        let buyer = Address::generate(&env);

        let (registry_id, ip_id, secret, blinding) = setup_registry(&env, &seller);
        let token_id = setup_token(&env, &seller, &buyer, 1_000_000);
        let client = setup_contract(&env, &registry_id);

        let mut signers = Vec::new(&env);
        signers.push_back(seller.clone());
        signers.push_back(signer2.clone());
        signers.push_back(signer3.clone());

        let swap_id = client.initiate_swap_with_signers(
            &token_id, &ip_id, &seller, &1000i128, &buyer, &signers,
        );
        client.accept_swap(&swap_id);

        // Two of three signed — still blocked
        client.sign_swap_reveal(&swap_id, &seller);
        client.sign_swap_reveal(&swap_id, &signer2);
        assert!(
            client.try_reveal_key(&swap_id, &seller, &secret, &blinding).is_err(),
            "reveal must fail with only 2 of 3 signatures"
        );

        // Third signs — now unblocked
        client.sign_swap_reveal(&swap_id, &signer3);
        client.reveal_key(&swap_id, &seller, &secret, &blinding);

        let swap = client.get_swap(&swap_id).unwrap();
        assert_eq!(swap.status, SwapStatus::Completed);
    }

    #[test]
    fn test_sign_on_pending_swap_rejected() {
        let env = Env::default();
        env.mock_all_auths();

        let seller = Address::generate(&env);
        let co_signer = Address::generate(&env);
        let buyer = Address::generate(&env);

        let (registry_id, ip_id, _, _) = setup_registry(&env, &seller);
        let token_id = setup_token(&env, &seller, &buyer, 1_000_000);
        let client = setup_contract(&env, &registry_id);

        let mut signers = Vec::new(&env);
        signers.push_back(seller.clone());
        signers.push_back(co_signer.clone());

        let swap_id = client.initiate_swap_with_signers(
            &token_id, &ip_id, &seller, &1000i128, &buyer, &signers,
        );

        // Swap is still Pending — sign must fail (must be Accepted first)
        let result = client.try_sign_swap_reveal(&swap_id, &seller);
        assert!(result.is_err(), "sign must fail on a Pending swap");
    }

    #[test]
    fn test_empty_signers_list_rejected() {
        let env = Env::default();
        env.mock_all_auths();

        let seller = Address::generate(&env);
        let buyer = Address::generate(&env);

        let (registry_id, ip_id, _, _) = setup_registry(&env, &seller);
        let token_id = setup_token(&env, &seller, &buyer, 1_000_000);
        let client = setup_contract(&env, &registry_id);

        let empty_signers: Vec<Address> = Vec::new(&env);
        let result = client.try_initiate_swap_with_signers(
            &token_id, &ip_id, &seller, &1000i128, &buyer, &empty_signers,
        );
        assert!(result.is_err(), "empty signers list must be rejected");
    }

    // ── Batch sign tests ──────────────────────────────────────────────────────

    #[test]
    fn test_batch_sign_swap_reveal_signs_all() {
        let env = Env::default();
        env.mock_all_auths();

        let seller = Address::generate(&env);
        let co_signer = Address::generate(&env);
        let buyer = Address::generate(&env);

        let (registry_id, ip_id1, secret1, blinding1) = setup_registry(&env, &seller);
        let token_id = setup_token(&env, &seller, &buyer, 2_000_000);
        let client = setup_contract(&env, &registry_id);

        // Create a second IP for the second swap
        let registry_client = ip_registry::IpRegistryClient::new(&env, &registry_id);
        let secret2 = BytesN::from_array(&env, &[0xCCu8; 32]);
        let blinding2 = BytesN::from_array(&env, &[0xDDu8; 32]);
        let mut preimage2 = Bytes::new(&env);
        preimage2.append(&Bytes::from(secret2.clone()));
        preimage2.append(&Bytes::from(blinding2.clone()));
        let hash2: BytesN<32> = env.crypto().sha256(&preimage2).into();
        let ip_id2 = registry_client.commit_ip(&seller, &hash2);

        let mut signers = Vec::new(&env);
        signers.push_back(seller.clone());
        signers.push_back(co_signer.clone());

        let swap_id1 = client.initiate_swap_with_signers(
            &token_id, &ip_id1, &seller, &1000i128, &buyer, &signers,
        );
        let swap_id2 = client.initiate_swap_with_signers(
            &token_id, &ip_id2, &seller, &1000i128, &buyer, &signers,
        );

        client.accept_swap(&swap_id1);
        client.accept_swap(&swap_id2);

        // Batch sign both swaps as co_signer
        let mut batch_ids = Vec::new(&env);
        batch_ids.push_back(swap_id1);
        batch_ids.push_back(swap_id2);
        client.batch_sign_swap_reveal(&batch_ids, &co_signer);

        // Verify signature status: 1 of 2 signed for each
        let (signed1, required1) = client.get_swap_signature_status(&swap_id1);
        assert_eq!(signed1, 1, "swap1 should have 1 signature");
        assert_eq!(required1, 2, "swap1 requires 2 signatures");

        let (signed2, required2) = client.get_swap_signature_status(&swap_id2);
        assert_eq!(signed2, 1, "swap2 should have 1 signature");
        assert_eq!(required2, 2, "swap2 requires 2 signatures");

        // Seller also signs both
        client.batch_sign_swap_reveal(&batch_ids, &seller);

        let (signed1_after, _) = client.get_swap_signature_status(&swap_id1);
        assert_eq!(signed1_after, 2, "swap1 should have 2 signatures after seller signs");

        // Now reveal should succeed for both
        client.reveal_key(&swap_id1, &seller, &secret1, &blinding1);
        client.reveal_key(&swap_id2, &seller, &secret2, &blinding2);

        let swap1 = client.get_swap(&swap_id1).unwrap();
        let swap2 = client.get_swap(&swap_id2).unwrap();
        assert_eq!(swap1.status, SwapStatus::Completed);
        assert_eq!(swap2.status, SwapStatus::Completed);
    }

    #[test]
    fn test_batch_sign_duplicate_rejected() {
        let env = Env::default();
        env.mock_all_auths();

        let seller = Address::generate(&env);
        let co_signer = Address::generate(&env);
        let buyer = Address::generate(&env);

        let (registry_id, ip_id, _, _) = setup_registry(&env, &seller);
        let token_id = setup_token(&env, &seller, &buyer, 1_000_000);
        let client = setup_contract(&env, &registry_id);

        let mut signers = Vec::new(&env);
        signers.push_back(seller.clone());
        signers.push_back(co_signer.clone());

        let swap_id = client.initiate_swap_with_signers(
            &token_id, &ip_id, &seller, &1000i128, &buyer, &signers,
        );
        client.accept_swap(&swap_id);

        let mut batch_ids = Vec::new(&env);
        batch_ids.push_back(swap_id);

        client.batch_sign_swap_reveal(&batch_ids, &seller);
        // Second call with same signer must fail
        let result = client.try_batch_sign_swap_reveal(&batch_ids, &seller);
        assert!(result.is_err(), "duplicate batch signature must be rejected");
    }

    #[test]
    fn test_batch_sign_non_required_signer_rejected() {
        let env = Env::default();
        env.mock_all_auths();

        let seller = Address::generate(&env);
        let co_signer = Address::generate(&env);
        let outsider = Address::generate(&env);
        let buyer = Address::generate(&env);

        let (registry_id, ip_id, _, _) = setup_registry(&env, &seller);
        let token_id = setup_token(&env, &seller, &buyer, 1_000_000);
        let client = setup_contract(&env, &registry_id);

        let mut signers = Vec::new(&env);
        signers.push_back(seller.clone());
        signers.push_back(co_signer.clone());

        let swap_id = client.initiate_swap_with_signers(
            &token_id, &ip_id, &seller, &1000i128, &buyer, &signers,
        );
        client.accept_swap(&swap_id);

        let mut batch_ids = Vec::new(&env);
        batch_ids.push_back(swap_id);

        let result = client.try_batch_sign_swap_reveal(&batch_ids, &outsider);
        assert!(result.is_err(), "outsider must not be able to batch sign");
    }

    #[test]
    fn test_batch_reveal_blocked_without_all_signatures() {
        let env = Env::default();
        env.mock_all_auths();

        let seller = Address::generate(&env);
        let co_signer = Address::generate(&env);
        let buyer = Address::generate(&env);

        let (registry_id, ip_id, secret, blinding) = setup_registry(&env, &seller);
        let token_id = setup_token(&env, &seller, &buyer, 2_000_000);
        let client = setup_contract(&env, &registry_id);

        let registry_client = ip_registry::IpRegistryClient::new(&env, &registry_id);
        let secret2 = BytesN::from_array(&env, &[0xCCu8; 32]);
        let blinding2 = BytesN::from_array(&env, &[0xDDu8; 32]);
        let mut preimage2 = Bytes::new(&env);
        preimage2.append(&Bytes::from(secret2.clone()));
        preimage2.append(&Bytes::from(blinding2.clone()));
        let hash2: BytesN<32> = env.crypto().sha256(&preimage2).into();
        let ip_id2 = registry_client.commit_ip(&seller, &hash2);

        let mut signers = Vec::new(&env);
        signers.push_back(seller.clone());
        signers.push_back(co_signer.clone());

        let swap_id1 = client.initiate_swap_with_signers(
            &token_id, &ip_id, &seller, &1000i128, &buyer, &signers,
        );
        let swap_id2 = client.initiate_swap_with_signers(
            &token_id, &ip_id2, &seller, &1000i128, &buyer, &signers,
        );

        client.accept_swap(&swap_id1);
        client.accept_swap(&swap_id2);

        // Only seller signs — co_signer has not signed
        client.sign_swap_reveal(&swap_id1, &seller);
        client.sign_swap_reveal(&swap_id2, &seller);

        let mut batch_ids = Vec::new(&env);
        batch_ids.push_back(swap_id1);
        batch_ids.push_back(swap_id2);

        let mut secrets = Vec::new(&env);
        secrets.push_back(secret);
        secrets.push_back(secret2);

        let mut blindings = Vec::new(&env);
        blindings.push_back(blinding);
        blindings.push_back(blinding2);

        // batch_reveal_keys must fail because co_signer hasn't signed
        let result = client.try_batch_reveal_keys(&batch_ids, &secrets, &blindings, &seller);
        assert!(result.is_err(), "batch_reveal_keys must fail when not all signers have signed");
    }

    #[test]
    fn test_get_swap_signature_status_no_signers() {
        let env = Env::default();
        env.mock_all_auths();

        let seller = Address::generate(&env);
        let buyer = Address::generate(&env);

        let (registry_id, ip_id, _, _) = setup_registry(&env, &seller);
        let token_id = setup_token(&env, &seller, &buyer, 1_000_000);
        let client = setup_contract(&env, &registry_id);

        let swap_id = client.initiate_swap(
            &token_id, &ip_id, &seller, &1000i128, &buyer, &0u32, &None, &0i128, &false,
        );

        let (signed, required) = client.get_swap_signature_status(&swap_id);
        assert_eq!(signed, 0, "no signers means 0 signed");
        assert_eq!(required, 0, "no signers means 0 required");
    }
}
