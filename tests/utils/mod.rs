#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::new_without_default)]
#![allow(unused_mut)]

pub mod utils {
    use ink_env::{call, test};
    use ink_env::{AccountId as Account};
    use crate::subscrypt::subscrypt::Subscrypt;

    pub fn set_caller(callee: Account, from: Account, value: u128) {
        test::push_execution_context::<ink_env::DefaultEnvironment>(
            from,
            callee,
            100,
            value,
            test::CallData::new(call::Selector::new([0x00; 4])),
        );
    }

    pub fn set_account_balance(callee: Account, value: u128) {
        ink_env::test::set_account_balance::<ink_env::DefaultEnvironment>(
            callee, value,
        )
            .expect("Cannot set account balance");
    }

    pub fn subscrypt_provider_register_scenario1(subscrypt: &mut Subscrypt, account: Account, durations: Vec<u64>, active_session_limits: Vec<u128>, prices: Vec<u128>, max_refund_percent_policies: Vec<u128>) {
        subscrypt.provider_register(
            durations.clone(),
            active_session_limits.clone(),
            prices.clone(),
            max_refund_percent_policies.clone(),
            account);
        for i in 0..durations.len() {
            assert_eq!(subscrypt.providers.get(&account).unwrap().plans.get(i).unwrap().duration, durations[i]);
            assert_eq!(subscrypt.providers.get(&account).unwrap().plans.get(i).unwrap().active_session_limit, active_session_limits[i]);
            assert_eq!(subscrypt.providers.get(&account).unwrap().plans.get(i).unwrap().price, prices[i]);
            assert_eq!(subscrypt.providers.get(&account).unwrap().plans.get(i).unwrap().max_refund_percent_policy, max_refund_percent_policies[i]);
        }
        assert_eq!(subscrypt.providers.get(&account).unwrap().money_address, account);
    }

    pub fn subscrypt_edit_plan_scenario1(subscrypt: &mut Subscrypt, account: Account, plan_index: u128, duration: u64, active: u128, price: u128, max_refund: u128, disabled: bool) {
        subscrypt.edit_plan(
            plan_index, duration, active, price, max_refund, disabled,
        );
        assert_eq!(subscrypt.providers.get(&account).unwrap().plans.get(1).unwrap().active_session_limit, active);
        assert_eq!(subscrypt.providers.get(&account).unwrap().plans.get(1).unwrap().duration, duration);
        assert_eq!(subscrypt.providers.get(&account).unwrap().plans.get(1).unwrap().price, price);
        assert_eq!(subscrypt.providers.get(&account).unwrap().plans.get(1).unwrap().max_refund_percent_policy, max_refund);
        assert_eq!(subscrypt.providers.get(&account).unwrap().money_address, account);
    }

    pub fn subscrypt_add_plan_scenario1(subscrypt: &mut Subscrypt, account: Account, durations: Vec<u64>, active_session_limits: Vec<u128>, prices: Vec<u128>, max_refund_percent_policies: Vec<u128>) {
        subscrypt.add_plan(
            durations.clone(),
            active_session_limits.clone(),
            prices.clone(),
            max_refund_percent_policies.clone(),
        );
        assert_eq!(subscrypt.providers.get(&account).unwrap().plans.get(2).unwrap().active_session_limit, active_session_limits[0]);
        assert_eq!(subscrypt.providers.get(&account).unwrap().plans.get(2).unwrap().duration, durations[0]);
        assert_eq!(subscrypt.providers.get(&account).unwrap().plans.get(2).unwrap().price, prices[0]);
        assert_eq!(subscrypt.providers.get(&account).unwrap().plans.get(2).unwrap().max_refund_percent_policy, max_refund_percent_policies[0]);
        assert_eq!(subscrypt.providers.get(&account).unwrap().money_address, account);
    }
}