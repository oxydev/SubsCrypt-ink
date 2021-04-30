#![cfg_attr(not(feature = "std"), no_std)]

pub mod utils {
    use crate::subscrypt::subscrypt::Subscrypt;
    use ink_env::AccountId as Account;
    use ink_env::{call, test};
    const DEFAULT_GAS_LIMIT: u128 = 1_000_000;
    use ink_env::hash::{HashOutput, Sha2x256};

    
    /// This function will set the `caller` and `callee` of transaction with endowment amount of
    /// `value`
    pub fn set_caller(callee: Account, from: Account, value: u128) {
        test::push_execution_context::<ink_env::DefaultEnvironment>(
            from,
            callee,
            DEFAULT_GAS_LIMIT,
            value,
            test::CallData::new(call::Selector::new([0x00; 4])),
        );
    }
    /// This function will set the account balance of `callee` to `value`
    pub fn set_account_balance(account: Account, value: u128) {
        ink_env::test::set_account_balance::<ink_env::DefaultEnvironment>(account, value)
            .expect("Cannot set account balance");
    }

    /// This function will do the provider registration routines
    pub fn subscrypt_provider_register_routine(
        subscrypt: &mut Subscrypt,
        account: Account,
        durations: Vec<u64>,
        active_session_limits: Vec<u128>,
        prices: Vec<u128>,
        max_refund_permille_policies: Vec<u128>,
        username: String,
        plan_charastristics: Vec<Vec<String>>,
    ) {
        let p: String = "pass_phrase".to_string();
        let encodable = [p];
        let mut output = <Sha2x256 as HashOutput>::Type::default(); // 256-bit buffer
        ink_env::hash_encoded::<Sha2x256, _>(&encodable, &mut output);

        subscrypt.provider_register(
            durations.clone(),
            active_session_limits.clone(),
            prices.clone(),
            max_refund_permille_policies.clone(),
            account,
            username,
            output,
            plan_charastristics,
        );
        for i in 0..durations.len() {
            assert_eq!(
                subscrypt
                    .providers
                    .get(&account)
                    .unwrap()
                    .plans
                    .get(i)
                    .unwrap()
                    .duration,
                durations[i]
            );
            assert_eq!(
                subscrypt
                    .providers
                    .get(&account)
                    .unwrap()
                    .plans
                    .get(i)
                    .unwrap()
                    .active_session_limit,
                active_session_limits[i]
            );
            assert_eq!(
                subscrypt
                    .providers
                    .get(&account)
                    .unwrap()
                    .plans
                    .get(i)
                    .unwrap()
                    .price,
                prices[i]
            );
            assert_eq!(
                subscrypt
                    .providers
                    .get(&account)
                    .unwrap()
                    .plans
                    .get(i)
                    .unwrap()
                    .max_refund_permille_policy,
                max_refund_permille_policies[i]
            );
        }
        assert_eq!(
            subscrypt.providers.get(&account).unwrap().money_address,
            account
        );
    }
    /// This function will do the edit plan of provider routines
    pub fn subscrypt_edit_plan_routine(
        subscrypt: &mut Subscrypt,
        account: Account,
        plan_index: u128,
        duration: u64,
        active: u128,
        price: u128,
        max_refund: u128,
        disabled: bool,
    ) {
        subscrypt.edit_plan(plan_index, duration, active, price, max_refund, disabled);
        assert_eq!(
            subscrypt
                .providers
                .get(&account)
                .unwrap()
                .plans
                .get(1)
                .unwrap()
                .active_session_limit,
            active
        );
        assert_eq!(
            subscrypt
                .providers
                .get(&account)
                .unwrap()
                .plans
                .get(1)
                .unwrap()
                .duration,
            duration
        );
        assert_eq!(
            subscrypt
                .providers
                .get(&account)
                .unwrap()
                .plans
                .get(1)
                .unwrap()
                .price,
            price
        );
        assert_eq!(
            subscrypt
                .providers
                .get(&account)
                .unwrap()
                .plans
                .get(1)
                .unwrap()
                .max_refund_permille_policy,
            max_refund
        );
        assert_eq!(
            subscrypt.providers.get(&account).unwrap().money_address,
            account
        );
    }
    /// This function will do the add plan of provider routines
    pub fn subscrypt_add_plan_routine(
        subscrypt: &mut Subscrypt,
        account: Account,
        durations: Vec<u64>,
        active_session_limits: Vec<u128>,
        prices: Vec<u128>,
        max_refund_permille_policies: Vec<u128>,
        plan_charastristics: Vec<Vec<String>>,
    ) {
        subscrypt.add_plan(
            durations.clone(),
            active_session_limits.clone(),
            prices.clone(),
            max_refund_permille_policies.clone(),
            plan_charastristics,
        );
        assert_eq!(
            subscrypt
                .providers
                .get(&account)
                .unwrap()
                .plans
                .get(2)
                .unwrap()
                .active_session_limit,
            active_session_limits[0]
        );
        assert_eq!(
            subscrypt
                .providers
                .get(&account)
                .unwrap()
                .plans
                .get(2)
                .unwrap()
                .duration,
            durations[0]
        );
        assert_eq!(
            subscrypt
                .providers
                .get(&account)
                .unwrap()
                .plans
                .get(2)
                .unwrap()
                .price,
            prices[0]
        );
        assert_eq!(
            subscrypt
                .providers
                .get(&account)
                .unwrap()
                .plans
                .get(2)
                .unwrap()
                .max_refund_permille_policy,
            max_refund_permille_policies[0]
        );
        assert_eq!(
            subscrypt.providers.get(&account).unwrap().money_address,
            account
        );
    }
}
