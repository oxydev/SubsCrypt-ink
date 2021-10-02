#![cfg_attr(not(feature = "std"), no_std)]

#[path = "../src/lib.rs"]
mod subscrypt;

mod utils;

#[cfg(test)]
pub mod tests {
    use crate::subscrypt::subscrypt::LinkedList;
    use crate::subscrypt::subscrypt::PlanConsts;
    use crate::subscrypt::subscrypt::Subscrypt;
    use crate::utils::utils::{
        set_account_balance, set_caller, subscrypt_add_plan_routine, subscrypt_edit_plan_routine,
        subscrypt_provider_register_routine,
    };
    use ink_env::hash::{HashOutput, Sha2x256};
    use ink_lang as ink;

    #[ink::test]
    fn constructor_works() {
        let subscrypt = Subscrypt::new();
        assert_eq!(subscrypt.provider_register_fee, 100);
    }

    #[ink::test]
    fn default_works() {
        let subscrypt = Subscrypt::default();
        assert_eq!(subscrypt.provider_register_fee, 100);
    }

    #[ink::test]
    fn linked_list_works() {
        let linked = LinkedList::new();
        assert_eq!(linked.back, 0);
    }

    #[ink::test]
    fn linked_list_default_works() {
        let linked = LinkedList::default();
        assert_eq!(linked.back, 0);
    }

    /// Simple scenario that `alice` register as a provider
    /// `alice` has two plans. One is daily and other is monthly.
    /// `alice` also pays 100 because of the policy of the registering in contract.
    #[ink::test]
    fn provider_register_works() {
        let mut subscrypt = Subscrypt::new();
        let accounts = ink_env::test::default_accounts::<ink_env::DefaultEnvironment>()
            .expect("Cannot get accounts");
        let callee =
            ink_env::test::get_current_contract_account_id::<ink_env::DefaultEnvironment>()
                .expect("Cannot get contract id");

        set_caller(callee, accounts.alice, 100);
        subscrypt_provider_register_routine(
            &mut subscrypt,
            accounts.alice,
            vec![60 * 60 * 24, 60 * 60 * 24 * 30],
            vec![10000, 50000],
            vec![50, 100],
            "alice".to_string(),
            vec![vec!["key".to_string()], vec!["key".to_string()]],
        );
        assert_eq!(subscrypt.get_plan_length(accounts.alice), 2);
    }

    /// Simple scenario that `alice` tries to register as a provider but it fails because of
    /// insufficient payment of staking value of policy of contract.
    #[ink::test]
    #[should_panic(expected = "You have to pay a minimum amount to register in the contract!")]
    fn provider_register_fails_insufficient_payment() {
        let mut subscrypt = Subscrypt::new();
        let accounts = ink_env::test::default_accounts::<ink_env::DefaultEnvironment>()
            .expect("Cannot get accounts");
        let callee =
            ink_env::test::get_current_contract_account_id::<ink_env::DefaultEnvironment>()
                .expect("Cannot get contract id");
        set_caller(callee, accounts.alice, 90);
        subscrypt_provider_register_routine(
            &mut subscrypt,
            accounts.alice,
            vec![60 * 60 * 24, 60 * 60 * 24 * 30],
            vec![10000, 50000],
            vec![50, 100],
            "alice".to_string(),
            vec![vec!["key".to_string()], vec!["key".to_string()]],
        );
    }

    /// Simple scenario that `alice` tries to register as a provider but it fails because of
    /// wrong args(length of vectors of plan configs are not equal).
    #[ink::test]
    #[should_panic(expected = "Wrong Number of Args")]
    fn provider_register_fails_wrong_arguments() {
        let mut subscrypt = Subscrypt::new();
        let accounts = ink_env::test::default_accounts::<ink_env::DefaultEnvironment>()
            .expect("Cannot get accounts");
        let callee =
            ink_env::test::get_current_contract_account_id::<ink_env::DefaultEnvironment>()
                .expect("Cannot get contract id");
        set_caller(callee, accounts.alice, 100);
        subscrypt_provider_register_routine(
            &mut subscrypt,
            accounts.alice,
            vec![60 * 60 * 24],
            vec![10000, 50000],
            vec![50, 100],
            "alice".to_string(),
            vec![vec!["key".to_string()], vec!["key".to_string()]],
        );
    }
    /// Simple scenario that `alice` edit a plan as a provider
    /// `alice` has two plans. One is daily and other is monthly.
    /// `alice` changes the monthly plan configs to different configs
    #[ink::test]
    fn edit_plan_works() {
        let mut subscrypt = Subscrypt::new();
        let accounts = ink_env::test::default_accounts::<ink_env::DefaultEnvironment>()
            .expect("Cannot get accounts");
        let callee =
            ink_env::test::get_current_contract_account_id::<ink_env::DefaultEnvironment>()
                .expect("Cannot get contract id");

        set_caller(callee, accounts.alice, 100);
        subscrypt_provider_register_routine(
            &mut subscrypt,
            accounts.alice,
            vec![60 * 60 * 24, 60 * 60 * 24 * 30],
            vec![10000, 50000],
            vec![50, 100],
            "alice".to_string(),
            vec![vec!["key".to_string()], vec!["key".to_string()]],
        );
        subscrypt_edit_plan_routine(
            &mut subscrypt,
            accounts.alice,
            1,
            60 * 60 * 24 * 10,
            100000,
            500,
            false,
        );
    }
    /// Simple scenario that `alice` tries to edit a plan as a provider
    /// `alice` has two plans. One is daily and other is monthly.
    /// `alice` tries to change config of her third plan which doesn't exist so it will fail
    #[ink::test]
    #[should_panic(expected = "please select a valid plan")]
    fn edit_plan_fails_invalid_plan_index() {
        let mut subscrypt = Subscrypt::new();
        let accounts = ink_env::test::default_accounts::<ink_env::DefaultEnvironment>()
            .expect("Cannot get accounts");
        let callee =
            ink_env::test::get_current_contract_account_id::<ink_env::DefaultEnvironment>()
                .expect("Cannot get contract id");
        set_caller(callee, accounts.alice, 100);
        subscrypt_provider_register_routine(
            &mut subscrypt,
            accounts.alice,
            vec![60 * 60 * 24, 60 * 60 * 24 * 30],
            vec![10000, 50000],
            vec![50, 100],
            "alice".to_string(),
            vec![vec!["key".to_string()], vec!["key".to_string()]],
        );
        subscrypt_edit_plan_routine(
            &mut subscrypt,
            accounts.alice,
            2,
            60 * 60 * 24 * 10,
            100000,
            500,
            false,
        );
    }
    /// Simple scenario that `alice` adds a plan as a provider
    /// `alice` has two plans. One is daily and other is monthly.
    /// `alice` adds a third plan with 10 days long and 50% refund policy
    #[ink::test]
    fn add_plan_works() {
        let mut subscrypt = Subscrypt::new();
        let accounts = ink_env::test::default_accounts::<ink_env::DefaultEnvironment>()
            .expect("Cannot get accounts");
        let callee =
            ink_env::test::get_current_contract_account_id::<ink_env::DefaultEnvironment>()
                .expect("Cannot get contract id");

        set_caller(callee, accounts.alice, 100);
        subscrypt_provider_register_routine(
            &mut subscrypt,
            accounts.alice,
            vec![60 * 60 * 24, 60 * 60 * 24 * 30],
            vec![10000, 50000],
            vec![50, 100],
            "alice".to_string(),
            vec![vec!["key".to_string()], vec!["key".to_string()]],
        );
        subscrypt_add_plan_routine(
            &mut subscrypt,
            accounts.alice,
            vec![60 * 60 * 24 * 10],
            vec![100000],
            vec![500],
            vec![vec!["key".to_string()]],
        )
    }
    /// Simple scenario that `alice` tries to add a plan as a provider
    /// `alice` has two plans. One is daily and other is monthly.
    /// `alice` tries to add more plans but obviously she is doing it wrong
    #[ink::test]
    #[should_panic(expected = "Wrong Number of Args")]
    fn add_plan_fails_wrong_arguments() {
        let mut subscrypt = Subscrypt::new();
        let accounts = ink_env::test::default_accounts::<ink_env::DefaultEnvironment>()
            .expect("Cannot get accounts");
        let callee =
            ink_env::test::get_current_contract_account_id::<ink_env::DefaultEnvironment>()
                .expect("Cannot get contract id");
        set_caller(callee, accounts.alice, 100);
        subscrypt_provider_register_routine(
            &mut subscrypt,
            accounts.alice,
            vec![60 * 60 * 24, 60 * 60 * 24 * 30],
            vec![10000, 50000],
            vec![50, 100],
            "alice".to_string(),
            vec![vec!["key".to_string()], vec!["key".to_string()]],
        );
        subscrypt_add_plan_routine(
            &mut subscrypt,
            accounts.alice,
            vec![60 * 60 * 24 * 10],
            vec![100000, 100000],
            vec![500],
            vec![vec!["key".to_string()]],
        );
    }

    /// Simple scenario that `alice` tries to add characteristic
    /// `alice` has two plans. One is daily and other is monthly.
    /// `alice` tries to add more plans but obviously she is doing it wrong
    #[ink::test]
    fn add_characteristic_for_plan_works() {
        let mut subscrypt = Subscrypt::new();
        let accounts = ink_env::test::default_accounts::<ink_env::DefaultEnvironment>()
            .expect("Cannot get accounts");
        let callee =
            ink_env::test::get_current_contract_account_id::<ink_env::DefaultEnvironment>()
                .expect("Cannot get contract id");
        set_caller(callee, accounts.alice, 100);
        subscrypt_provider_register_routine(
            &mut subscrypt,
            accounts.alice,
            vec![60 * 60 * 24, 60 * 60 * 24 * 30],
            vec![10000, 50000],
            vec![50, 100],
            "alice".to_string(),
            vec![vec!["key".to_string()], vec!["key".to_string()]],
        );
        subscrypt.add_characteristic_for_plan(0, vec!["key2".to_string(), "key3".to_string()]);
        assert_eq!(
            subscrypt.get_plan_characteristics(accounts.alice, 0),
            vec!["key".to_string(), "key2".to_string(), "key3".to_string()]
        )
    }

    /// Simple scenario that `alice` disables a plan as a provider
    /// `alice` has two plans. One is daily and other is monthly.
    /// `alice` disables and enables its plan
    #[ink::test]
    fn change_disable_works() {
        let mut subscrypt = Subscrypt::new();
        let accounts = ink_env::test::default_accounts::<ink_env::DefaultEnvironment>()
            .expect("Cannot get accounts");
        let callee =
            ink_env::test::get_current_contract_account_id::<ink_env::DefaultEnvironment>()
                .expect("Cannot get contract id");

        set_account_balance(callee, 100);
        set_caller(callee, accounts.alice, 100);
        subscrypt_provider_register_routine(
            &mut subscrypt,
            accounts.alice,
            vec![60 * 60 * 24, 60 * 60 * 24 * 30],
            vec![10000, 50000],
            vec![50, 100],
            "alice".to_string(),
            vec![vec!["key".to_string()], vec!["key".to_string()]],
        );

        subscrypt.change_disable(1);
        assert_eq!(
            subscrypt
                .providers
                .get(&accounts.alice)
                .unwrap()
                .plans
                .get(1)
                .unwrap()
                .disabled,
            true
        );

        subscrypt.change_disable(1);
        assert_eq!(
            subscrypt
                .providers
                .get(&accounts.alice)
                .unwrap()
                .plans
                .get(1)
                .unwrap()
                .disabled,
            false
        );
    }

    /// Simple scenario that `alice` register as a provider and `bob` will subscribe to her second plan
    /// then alice will change her subscrypt_pass
    /// `alice` has two plans. One is daily and other is monthly.
    /// `alice` also pays 100 because of the policy of the registering in contract.
    /// `bob` pays 50000 for her second plan price
    #[ink::test]
    fn set_subscrypt_pass_works() {
        let mut subscrypt = Subscrypt::new();
        let accounts = ink_env::test::default_accounts::<ink_env::DefaultEnvironment>()
            .expect("Cannot get accounts");
        let callee =
            ink_env::test::get_current_contract_account_id::<ink_env::DefaultEnvironment>()
                .expect("Cannot get contract id");

        set_account_balance(callee, 50100);
        set_caller(callee, accounts.alice, 100);
        subscrypt_provider_register_routine(
            &mut subscrypt,
            accounts.alice,
            vec![60 * 60 * 24, 60 * 60 * 24 * 30],
            vec![10000, 50000],
            vec![50, 100],
            "alice".to_string(),
            vec![vec!["key".to_string()], vec!["key".to_string()]],
        );

        set_caller(callee, accounts.bob, 50000);

        let p: String = "pass_phrase".to_string();
        let encodable = [p];
        let mut output = <Sha2x256 as HashOutput>::Type::default(); // 256-bit buffer
        ink_env::hash_encoded::<Sha2x256, _>(&encodable, &mut output);

        subscrypt.subscribe(
            accounts.alice,
            1,
            output,
            "bob".to_string(),
            vec!["nothing important".to_string()],
        );
        assert_eq!(
            subscrypt
                .users
                .get(&accounts.bob)
                .unwrap()
                .list_of_providers
                .get(0)
                .unwrap(),
            &accounts.alice
        );
        subscrypt
            .retrieve_whole_data_with_username("bob".to_string(), "pass_phrase".parse().unwrap());

        let p: String = "new_pass_phrase".to_string();
        let encodable = [p];
        let mut output = <Sha2x256 as HashOutput>::Type::default(); // 256-bit buffer
        ink_env::hash_encoded::<Sha2x256, _>(&encodable, &mut output);

        subscrypt.set_user_subscrypt_pass(output);
        subscrypt.retrieve_whole_data_with_username(
            "bob".to_string(),
            "new_pass_phrase".parse().unwrap(),
        );
    }

    /// Simple scenario that `alice` register as a provider and `bob` will subscribe to her second plan
    /// `alice` has two plans. One is daily and other is monthly.
    /// `alice` also pays 100 because of the policy of the registering in contract.
    /// `bob` pays 50000 for her second plan price
    #[ink::test]
    fn subscribe_works() {
        let mut subscrypt = Subscrypt::new();
        let accounts = ink_env::test::default_accounts::<ink_env::DefaultEnvironment>()
            .expect("Cannot get accounts");
        let callee =
            ink_env::test::get_current_contract_account_id::<ink_env::DefaultEnvironment>()
                .expect("Cannot get contract id");

        set_account_balance(callee, 50100);
        set_caller(callee, accounts.alice, 100);
        subscrypt_provider_register_routine(
            &mut subscrypt,
            accounts.alice,
            vec![60 * 60 * 24, 60 * 60 * 24 * 30],
            vec![10000, 50000],
            vec![50, 100],
            "alice".to_string(),
            vec![vec!["key".to_string()], vec!["key".to_string()]],
        );

        set_caller(callee, accounts.bob, 50000);

        subscrypt.subscribe(
            accounts.alice,
            1,
            [0; 32],
            "bob".to_string(),
            vec!["nothing important".to_string()],
        );
        assert_eq!(
            subscrypt
                .users
                .get(&accounts.bob)
                .unwrap()
                .list_of_providers
                .get(0)
                .unwrap(),
            &accounts.alice
        );
    }

    /// check get_user_plan_characteristics
    #[ink::test]
    fn get_user_plan_characteristics_works() {
        let mut subscrypt = Subscrypt::new();
        let accounts = ink_env::test::default_accounts::<ink_env::DefaultEnvironment>()
            .expect("Cannot get accounts");
        let callee =
            ink_env::test::get_current_contract_account_id::<ink_env::DefaultEnvironment>()
                .expect("Cannot get contract id");

        set_account_balance(callee, 50100);
        set_caller(callee, accounts.alice, 100);
        subscrypt_provider_register_routine(
            &mut subscrypt,
            accounts.alice,
            vec![60 * 60 * 24, 60 * 60 * 24 * 30],
            vec![10000, 50000],
            vec![50, 100],
            "alice".to_string(),
            vec![vec!["key".to_string()], vec!["key".to_string()]],
        );

        set_caller(callee, accounts.bob, 50000);

        subscrypt.subscribe(
            accounts.alice,
            1,
            [0; 32],
            "bob".to_string(),
            vec!["nothing important".to_string()],
        );
        assert_eq!(
            subscrypt
                .users
                .get(&accounts.bob)
                .unwrap()
                .list_of_providers
                .get(0)
                .unwrap(),
            &accounts.alice
        );
        assert_eq!(
        subscrypt.get_user_plan_characteristics(accounts.bob, accounts.alice, 1).get(0).unwrap() ,&"nothing important".to_string()
        );
    }

    /// Simple scenario that `alice` register as a provider and `bob` tries to subscribe to her second plan
    /// `alice` has two plans. One is daily and other is monthly.
    /// `alice` also pays 100 because of the policy of the registering in contract.
    /// `bob` pays 49500 for her second plan price which is less than 50000 so this will fail
    #[ink::test]
    #[should_panic(expected = "You have to pay exact plan price")]
    fn subscribe_fails_insufficient_paying() {
        let mut subscrypt = Subscrypt::new();
        let accounts = ink_env::test::default_accounts::<ink_env::DefaultEnvironment>()
            .expect("Cannot get accounts");
        let callee =
            ink_env::test::get_current_contract_account_id::<ink_env::DefaultEnvironment>()
                .expect("Cannot get contract id");

        set_account_balance(callee, 50100);
        set_caller(callee, accounts.alice, 100);
        subscrypt_provider_register_routine(
            &mut subscrypt,
            accounts.alice,
            vec![60 * 60 * 24, 60 * 60 * 24 * 30],
            vec![10000, 50000],
            vec![50, 100],
            "alice".to_string(),
            vec![vec!["key".to_string()], vec!["key".to_string()]],
        );
        set_caller(callee, accounts.bob, 49500);

        subscrypt.subscribe(
            accounts.alice,
            1,
            [0; 32],
            "bob".to_string(),
            vec!["nothing important".to_string()],
        );
    }

    /// Simple scenario that `alice` register as a provider and `bob` tries to subscribe to her second plan
    /// `alice` has two plans. One is daily and other is monthly.
    /// `alice` also pays 100 because of the policy of the registering in contract.
    /// `bob` pays 49500 for her second plan price which is less than 50000 so this will fail
    #[ink::test]
    #[should_panic(expected = "username is invalid")]
    fn subscribe_fails_duplicate_username() {
        let mut subscrypt = Subscrypt::new();
        let accounts = ink_env::test::default_accounts::<ink_env::DefaultEnvironment>()
            .expect("Cannot get accounts");
        let callee =
            ink_env::test::get_current_contract_account_id::<ink_env::DefaultEnvironment>()
                .expect("Cannot get contract id");

        set_account_balance(callee, 50100);
        set_caller(callee, accounts.alice, 100);
        subscrypt_provider_register_routine(
            &mut subscrypt,
            accounts.alice,
            vec![60 * 60 * 24, 60 * 60 * 24 * 30],
            vec![10000, 50000],
            vec![50, 100],
            "alice".to_string(),
            vec![vec!["key".to_string()], vec!["key".to_string()]],
        );
        set_caller(callee, accounts.bob, 50000);

        subscrypt.subscribe(
            accounts.alice,
            1,
            [0; 32],
            "alice".to_string(),
            vec!["nothing important".to_string()],
        );
    }

    /// Simple scenario that `alice` register as a provider and `bob` will subscribe to her second plan
    /// and then `alice` tries to withdraw locked money
    /// `alice` has two plans. One is daily and other is monthly.
    /// `alice` also pays 100 because of the policy of the registering in contract.
    /// `bob` pays 50000 for her second plan price
    #[ink::test]
    fn withdraw_works() {
        let mut subscrypt = Subscrypt::new();

        let accounts = ink_env::test::default_accounts::<ink_env::DefaultEnvironment>()
            .expect("Cannot get accounts");

        let callee =
            ink_env::test::get_current_contract_account_id::<ink_env::DefaultEnvironment>()
                .expect("Cannot get contract id");

        set_account_balance(callee, 50100);
        set_caller(callee, accounts.alice, 100);
        subscrypt_provider_register_routine(
            &mut subscrypt,
            accounts.alice,
            vec![60 * 60 * 24, 0],
            vec![10000, 50000],
            vec![50, 100],
            "alice".to_string(),
            vec![vec!["key".to_string()], vec!["key".to_string()]],
        );

        set_caller(callee, accounts.bob, 50000);
        subscrypt.subscribe(
            accounts.alice,
            1,
            [0; 32],
            "bob".to_string(),
            vec!["nothing important".to_string()],
        );
        assert_eq!(
            subscrypt
                .users
                .get(&accounts.bob)
                .unwrap()
                .list_of_providers
                .get(0)
                .unwrap(),
            &accounts.alice
        );
        set_caller(callee, accounts.alice, 0);
        let expected = subscrypt.get_withdrawable_amount();
        assert_eq!(
            subscrypt.withdraw(),
            expected
        );
    }
    /// Simple scenario that `alice` register as a provider and `bob` will subscribe to her second plan
    /// and then `eve` tries to withdraw locked money but she can't.
    /// `alice` has two plans. One is daily and other is monthly.
    /// `alice` also pays 100 because of the policy of the registering in contract.
    /// `bob` pays 50000 for her second plan price
    #[ink::test]
    #[should_panic(expected = "You are not a registered provider")]
    fn withdraw_fails_provider_must_be_registered() {
        let mut subscrypt = Subscrypt::new();

        let accounts = ink_env::test::default_accounts::<ink_env::DefaultEnvironment>()
            .expect("Cannot get accounts");

        let callee =
            ink_env::test::get_current_contract_account_id::<ink_env::DefaultEnvironment>()
                .expect("Cannot get contract id");

        set_account_balance(callee, 50100);
        set_caller(callee, accounts.alice, 100);
        subscrypt_provider_register_routine(
            &mut subscrypt,
            accounts.alice,
            vec![60 * 60 * 24, 60 * 60 * 24 * 30],
            vec![10000, 50000],
            vec![50, 100],
            "alice".to_string(),
            vec![vec!["key".to_string()], vec!["key".to_string()]],
        );

        set_caller(callee, accounts.bob, 50000);
        subscrypt.subscribe(
            accounts.alice,
            1,
            [0; 32],
            "bob".to_string(),
            vec!["nothing important".to_string()],
        );
        assert_eq!(
            subscrypt
                .users
                .get(&accounts.bob)
                .unwrap()
                .list_of_providers
                .get(0)
                .unwrap(),
            &accounts.alice
        );

        set_caller(callee, accounts.eve, 0);
        subscrypt.withdraw();
    }

    /// Simple scenario that `alice` register as a provider and `bob` will subscribe to her second plan
    /// and then `bob` tries to renew;
    /// `alice` has two plans. One is daily and other is monthly.
    /// `alice` also pays 100 because of the policy of the registering in contract.
    /// `bob` pays 50000 for her second plan price
    #[ink::test]
    fn renew_works() {
        let mut subscrypt = Subscrypt::new();

        let accounts = ink_env::test::default_accounts::<ink_env::DefaultEnvironment>()
            .expect("Cannot get accounts");
        let callee =
            ink_env::test::get_current_contract_account_id::<ink_env::DefaultEnvironment>()
                .expect("Cannot get contract id");
        set_account_balance(accounts.alice, 100);
        set_account_balance(accounts.bob, 100000);
        set_account_balance(callee, 100100);
        set_caller(callee, accounts.alice, 100);
        subscrypt_provider_register_routine(
            &mut subscrypt,
            accounts.alice,
            vec![60 * 60 * 24, 60 * 60 * 24 * 30],
            vec![10000, 50000],
            vec![50, 100],
            "alice".to_string(),
            vec![vec!["key".to_string()], vec!["key".to_string()]],
        );

        set_caller(callee, accounts.bob, 50000);

        subscrypt.subscribe(
            accounts.alice,
            1,
            [0; 32],
            "bob".to_string(),
            vec!["value".to_string()],
        );
        assert_eq!(
            subscrypt
                .records
                .get(&(accounts.bob, accounts.alice))
                .unwrap()
                .subscription_records
                .get(0)
                .unwrap()
                .refunded,
            false
        );
        set_caller(callee, accounts.bob, 50000);
        subscrypt.renew(accounts.alice, 1, vec!["value".to_string()]);
        assert_eq!(
            subscrypt
                .records
                .get(&(accounts.bob, accounts.alice))
                .unwrap()
                .subscription_records
                .get(1)
                .unwrap()
                .provider,
            accounts.alice
        );
        assert_eq!(
            ink_env::test::get_account_balance::<ink_env::DefaultEnvironment>(accounts.alice)
                .expect("Cannot set account balance"),
            95100
        );
        assert_eq!(
            ink_env::test::get_account_balance::<ink_env::DefaultEnvironment>(accounts.bob)
                .expect("Cannot set account balance"),
            100000
        );
        assert_eq!(
            ink_env::test::get_account_balance::<ink_env::DefaultEnvironment>(callee)
                .expect("Cannot set account balance"),
            5100
        );
    }

    /// Simple scenario that `alice` register as a provider and `bob` will subscribe to her second plan
    /// and then `bob` tries to refund locked money so he will get 10% of his money back which will be
    /// 5000.
    /// `alice` has two plans. One is daily and other is monthly.
    /// `alice` also pays 100 because of the policy of the registering in contract.
    /// `bob` pays 50000 for her second plan price
    #[ink::test]
    fn refund_works() {
        let mut subscrypt = Subscrypt::new();

        let accounts = ink_env::test::default_accounts::<ink_env::DefaultEnvironment>()
            .expect("Cannot get accounts");
        let callee =
            ink_env::test::get_current_contract_account_id::<ink_env::DefaultEnvironment>()
                .expect("Cannot get contract id");
        set_account_balance(accounts.alice, 100);
        set_account_balance(accounts.bob, 50000);
        set_account_balance(callee, 50100);
        set_caller(callee, accounts.alice, 100);
        subscrypt_provider_register_routine(
            &mut subscrypt,
            accounts.alice,
            vec![60 * 60 * 24, 60 * 60 * 24 * 30],
            vec![10000, 50000],
            vec![50, 100],
            "alice".to_string(),
            vec![vec!["key".to_string()], vec!["key".to_string()]],
        );

        set_caller(callee, accounts.bob, 50000);

        subscrypt.subscribe(
            accounts.alice,
            1,
            [0; 32],
            "bob".to_string(),
            vec!["nothing important".to_string()],
        );
        assert_eq!(
            subscrypt
                .records
                .get(&(accounts.bob, accounts.alice))
                .unwrap()
                .subscription_records
                .get(0)
                .unwrap()
                .refunded,
            false
        );

        subscrypt.refund(accounts.alice, 1);
        assert_eq!(
            subscrypt
                .records
                .get(&(accounts.bob, accounts.alice))
                .unwrap()
                .subscription_records
                .get(0)
                .unwrap()
                .refunded,
            true
        );
        assert_eq!(
            ink_env::test::get_account_balance::<ink_env::DefaultEnvironment>(accounts.alice)
                .expect("Cannot set account balance"),
            45100
        );
        assert_eq!(
            ink_env::test::get_account_balance::<ink_env::DefaultEnvironment>(accounts.bob)
                .expect("Cannot set account balance"),
            55000
        );
        assert_eq!(
            ink_env::test::get_account_balance::<ink_env::DefaultEnvironment>(callee)
                .expect("Cannot set account balance"),
            100
        );
    }
    /// Simple scenario that `alice` register as a provider and `bob` will subscribe to her second plan
    /// and then `bob` tries to refund locked money so he will get 10% of his money back which will be
    /// 5000. Then `bob` will try to refund two times but it will fail.
    /// `alice` has two plans. One is daily and other is monthly.
    /// `alice` also pays 100 because of the policy of the registering in contract.
    /// `bob` pays 50000 for her second plan price
    #[ink::test]
    #[should_panic(expected = "You are not in this plan or already refunded")]
    fn refund_fails_double_refund() {
        let mut subscrypt = Subscrypt::new();
        let accounts = ink_env::test::default_accounts::<ink_env::DefaultEnvironment>()
            .expect("Cannot get accounts");
        let callee =
            ink_env::test::get_current_contract_account_id::<ink_env::DefaultEnvironment>()
                .expect("Cannot get contract id");

        set_account_balance(callee, 50100);
        set_caller(callee, accounts.alice, 100);
        subscrypt_provider_register_routine(
            &mut subscrypt,
            accounts.alice,
            vec![60 * 60 * 24, 60 * 60 * 24 * 30],
            vec![10000, 50000],
            vec![50, 100],
            "alice".to_string(),
            vec![vec!["key".to_string()], vec!["key".to_string()]],
        );
        set_caller(callee, accounts.bob, 50000);

        subscrypt.subscribe(
            accounts.alice,
            1,
            [0; 32],
            "bob".to_string(),
            vec!["nothing important".to_string()],
        );
        assert_eq!(
            subscrypt
                .records
                .get(&(accounts.bob, accounts.alice))
                .unwrap()
                .subscription_records
                .get(0)
                .unwrap()
                .refunded,
            false
        );

        subscrypt.refund(accounts.alice, 1);
        subscrypt.refund(accounts.alice, 1);
    }
    /// Simple scenario that `alice` register as a provider and `bob` will subscribe to her second plan
    /// and then call `check_subscription` function and will get true
    /// `alice` has two plans. One is daily and other is monthly.
    /// `alice` also pays 100 because of the policy of the registering in contract.
    /// `bob` pays 50000 for her second plan price
    #[ink::test]
    fn check_subscription_works() {
        let mut subscrypt = Subscrypt::new();

        let accounts = ink_env::test::default_accounts::<ink_env::DefaultEnvironment>()
            .expect("Cannot get accounts");
        let callee =
            ink_env::test::get_current_contract_account_id::<ink_env::DefaultEnvironment>()
                .expect("Cannot get contract id");

        set_account_balance(callee, 50100);
        set_caller(callee, accounts.alice, 100);
        subscrypt_provider_register_routine(
            &mut subscrypt,
            accounts.alice,
            vec![60 * 60 * 24, 60 * 60 * 24 * 30],
            vec![10000, 50000],
            vec![50, 100],
            "alice".to_string(),
            vec![vec!["key".to_string()], vec!["key".to_string()]],
        );

        set_caller(callee, accounts.bob, 50000);

        subscrypt.subscribe(
            accounts.alice,
            1,
            [0; 32],
            "bob".to_string(),
            vec!["nothing important".to_string()],
        );
        assert_eq!(
            subscrypt
                .records
                .get(&(accounts.bob, accounts.alice))
                .unwrap()
                .subscription_records
                .get(0)
                .unwrap()
                .refunded,
            false
        );
        assert_eq!(
            subscrypt.check_subscription(accounts.bob, accounts.alice, 1),
            true
        );
        assert_eq!(
            subscrypt.check_subscription(accounts.bob, accounts.alice, 0),
            false
        );

        assert_eq!(
            subscrypt.check_subscription_with_username("bob".to_string(), accounts.alice, 1),
            true
        );
        assert_eq!(
            subscrypt.check_subscription_with_username("bob".to_string(), accounts.alice, 0),
            false
        );
    }

    /// Simple scenario that `alice` register as a provider and `bob` will subscribe to her second plan
    /// and put it's token and pass phrase this: token, pass_phrase.
    /// `bob` now tries to retrieve his data with that combination and he will successfully
    /// get the data.
    /// `alice` has two plans. One is daily and other is monthly.
    /// `alice` also pays 100 because of the policy of the registering in contract.
    /// `bob` pays 50000 for her second plan price
    #[ink::test]
    fn retrieve_data_with_username_works() {
        let mut subscrypt = Subscrypt::new();
        let accounts = ink_env::test::default_accounts::<ink_env::DefaultEnvironment>()
            .expect("Cannot get accounts");
        let callee =
            ink_env::test::get_current_contract_account_id::<ink_env::DefaultEnvironment>()
                .expect("Cannot get contract id");
        set_account_balance(callee, 50100);
        set_caller(callee, accounts.alice, 100);
        subscrypt_provider_register_routine(
            &mut subscrypt,
            accounts.alice,
            vec![60 * 60 * 24, 60 * 60 * 24 * 30],
            vec![10000, 50000],
            vec![50, 100],
            "alice".to_string(),
            vec![vec!["key".to_string()], vec!["key".to_string()]],
        );

        set_caller(callee, accounts.bob, 50000);
        let p: String = "pass_phrase".to_string();
        let encodable = [p];
        let mut output = <Sha2x256 as HashOutput>::Type::default(); // 256-bit buffer
        ink_env::hash_encoded::<Sha2x256, _>(&encodable, &mut output);

        subscrypt.subscribe(
            accounts.alice,
            1,
            output,
            "bob".to_string(),
            vec!["nothing important".to_string()],
        );
        assert_eq!(
            subscrypt
                .records
                .get(&(accounts.bob, accounts.alice))
                .unwrap()
                .subscription_records
                .get(0)
                .unwrap()
                .refunded,
            false
        );
        let s = subscrypt.retrieve_data_with_username(
            "bob".to_string(),
            accounts.alice,
            "pass_phrase".parse().unwrap(),
        );
        assert_eq!(s[0].provider, accounts.alice);
        assert_eq!(s[0].plan_index, 1);
        assert_eq!(s[0].plan.duration, 60 * 60 * 24 * 30);
    }
    /// Check comments of `retrieve_data_with_password_works` function
    #[ink::test]
    fn retrieve_data_with_wallet_works() {
        let mut subscrypt = Subscrypt::new();
        let accounts = ink_env::test::default_accounts::<ink_env::DefaultEnvironment>()
            .expect("Cannot get accounts");
        let callee =
            ink_env::test::get_current_contract_account_id::<ink_env::DefaultEnvironment>()
                .expect("Cannot get contract id");
        set_account_balance(callee, 50100);
        set_caller(callee, accounts.alice, 100);
        subscrypt_provider_register_routine(
            &mut subscrypt,
            accounts.alice,
            vec![60 * 60 * 24, 60 * 60 * 24 * 30],
            vec![10000, 50000],
            vec![50, 100],
            "alice".to_string(),
            vec![vec!["key".to_string()], vec!["key".to_string()]],
        );

        set_caller(callee, accounts.bob, 50000);

        subscrypt.subscribe(
            accounts.alice,
            1,
            [0; 32],
            "bob".to_string(),
            vec!["nothing important".to_string()],
        );
        assert_eq!(
            subscrypt
                .records
                .get(&(accounts.bob, accounts.alice))
                .unwrap()
                .subscription_records
                .get(0)
                .unwrap()
                .refunded,
            false
        );
        let s = subscrypt.retrieve_data_with_wallet(accounts.alice);
        assert_eq!(s[0].provider, accounts.alice);
        assert_eq!(s[0].plan_index, 1);
        assert_eq!(s[0].plan.duration, 60 * 60 * 24 * 30);
    }
    /// Check comments of `retrieve_data_with_password_works` function
    #[ink::test]
    fn retrieve_whole_data_with_wallet_works() {
        let mut subscrypt = Subscrypt::new();
        let accounts = ink_env::test::default_accounts::<ink_env::DefaultEnvironment>()
            .expect("Cannot get accounts");
        let callee =
            ink_env::test::get_current_contract_account_id::<ink_env::DefaultEnvironment>()
                .expect("Cannot get contract id");
        set_account_balance(callee, 50100);
        set_caller(callee, accounts.alice, 100);
        subscrypt_provider_register_routine(
            &mut subscrypt,
            accounts.alice,
            vec![60 * 60 * 24, 60 * 60 * 24 * 30],
            vec![10000, 50000],
            vec![50, 100],
            "alice".to_string(),
            vec![vec!["key".to_string()], vec!["key".to_string()]],
        );

        set_caller(callee, accounts.bob, 50000);

        subscrypt.subscribe(
            accounts.alice,
            1,
            [0; 32],
            "bob".to_string(),
            vec!["nothing important".to_string()],
        );
        assert_eq!(
            subscrypt
                .records
                .get(&(accounts.bob, accounts.alice))
                .unwrap()
                .subscription_records
                .get(0)
                .unwrap()
                .refunded,
            false
        );
        let s = subscrypt.retrieve_whole_data_with_wallet();
        assert_eq!(s[0].provider, accounts.alice);
        assert_eq!(s[0].plan_index, 1);
        assert_eq!(s[0].plan.duration, 60 * 60 * 24 * 30);
    }
    /// Check comments of `retrieve_data_with_password_works` function
    #[ink::test]
    fn retrieve_whole_data_with_username_works() {
        let mut subscrypt = Subscrypt::new();
        let accounts = ink_env::test::default_accounts::<ink_env::DefaultEnvironment>()
            .expect("Cannot get accounts");
        let callee =
            ink_env::test::get_current_contract_account_id::<ink_env::DefaultEnvironment>()
                .expect("Cannot get contract id");
        set_account_balance(callee, 50100);
        set_caller(callee, accounts.alice, 100);
        subscrypt_provider_register_routine(
            &mut subscrypt,
            accounts.alice,
            vec![60 * 60 * 24, 60 * 60 * 24 * 30],
            vec![10000, 50000],
            vec![50, 100],
            "alice".to_string(),
            vec![vec!["key".to_string()], vec!["key".to_string()]],
        );

        set_caller(callee, accounts.bob, 50000);

        let p: String = "pass_phrase".to_string();
        let encodable = [p];
        let mut output = <Sha2x256 as HashOutput>::Type::default(); // 256-bit buffer
        ink_env::hash_encoded::<Sha2x256, _>(&encodable, &mut output);

        subscrypt.subscribe(
            accounts.alice,
            1,
            output,
            "bob".to_string(),
            vec!["nothing important".to_string()],
        );
        assert_eq!(
            subscrypt
                .records
                .get(&(accounts.bob, accounts.alice))
                .unwrap()
                .subscription_records
                .get(0)
                .unwrap()
                .refunded,
            false
        );
        let s = subscrypt
            .retrieve_whole_data_with_username("bob".to_string(), "pass_phrase".parse().unwrap());
        assert_eq!(s[0].provider, accounts.alice);
        assert_eq!(s[0].plan_index, 1);
        assert_eq!(s[0].plan.duration, 60 * 60 * 24 * 30);
    }

    /// Simple scenario that `alice` register as a provider and `bob` will subscribe to her second plan
    /// and put it's token and pass phrase this: token, pass_phrase.
    /// `bob` now tries to check his authentication data and he will try two time with two pair of token
    /// and pass phrase and he will fail in first try and authenticate in second try
    /// get the data.
    /// `alice` has two plans. One is daily and other is monthly.
    /// `alice` also pays 100 because of the policy of the registering in contract.
    /// `bob` pays 50000 for her second plan price
    #[ink::test]
    fn check_auth_works() {
        let mut subscrypt = Subscrypt::new();
        let accounts = ink_env::test::default_accounts::<ink_env::DefaultEnvironment>()
            .expect("Cannot get accounts");
        let callee =
            ink_env::test::get_current_contract_account_id::<ink_env::DefaultEnvironment>()
                .expect("Cannot get contract id");
        set_account_balance(callee, 50100);
        set_caller(callee, accounts.alice, 100);
        subscrypt_provider_register_routine(
            &mut subscrypt,
            accounts.alice,
            vec![60 * 60 * 24, 60 * 60 * 24 * 30],
            vec![10000, 50000],
            vec![50, 100],
            "alice".to_string(),
            vec![vec!["key".to_string()], vec!["key".to_string()]],
        );

        set_caller(callee, accounts.bob, 50000);

        let p: String = "pass_phrase".to_string();
        let encodable = [p];
        let mut output = <Sha2x256 as HashOutput>::Type::default(); // 256-bit buffer
        ink_env::hash_encoded::<Sha2x256, _>(&encodable, &mut output);

        subscrypt.subscribe(
            accounts.alice,
            1,
            output,
            "bob".to_string(),
            vec!["nothing important".to_string()],
        );
        assert_eq!(
            subscrypt
                .records
                .get(&(accounts.bob, accounts.alice))
                .unwrap()
                .subscription_records
                .get(0)
                .unwrap()
                .refunded,
            false
        );
        assert_eq!(
            subscrypt.check_auth(accounts.bob, accounts.alice, "pass_phras".parse().unwrap()),
            false
        );

        // No record for user charlie & provider alice
        assert_eq!(
            subscrypt.check_auth(
                accounts.charlie,
                accounts.alice,
                "pass_phrase".parse().unwrap()
            ),
            false
        );

        assert_eq!(
            subscrypt.check_auth_with_username(
                "bob".to_string(),
                accounts.alice,
                "pass_phras".to_string()
            ),
            false
        );

        assert_eq!(
            subscrypt.check_auth_with_username(
                "bob".to_string(),
                accounts.alice,
                "pass_phrase".to_string(),
            ),
            true
        );

        assert_eq!(
            subscrypt.check_auth(accounts.bob, accounts.alice, "pass_phrase".to_string()),
            true
        );

        assert_eq!(
            subscrypt
                .provider_check_auth_with_username("alice".to_string(), "pass_phras".to_string()),
            false
        );
        assert_eq!(
            subscrypt
                .provider_check_auth_with_username("alice".to_string(), "pass_phrase".to_string()),
            true
        );

        assert_eq!(
            subscrypt.provider_check_auth(accounts.alice, "pass_phras".to_string()),
            false
        );
        assert_eq!(
            subscrypt.provider_check_auth(accounts.alice, "pass_phrase".to_string()),
            true
        );

        assert_eq!(
            subscrypt.user_check_auth_with_username("bob".to_string(), "pass_phras".to_string()),
            false
        );
        assert_eq!(
            subscrypt.user_check_auth_with_username("bob".to_string(), "pass_phrase".to_string()),
            true
        );

        assert_eq!(
            subscrypt.user_check_auth(accounts.bob, "pass_phras".to_string()),
            false
        );
        assert_eq!(
            subscrypt.user_check_auth(accounts.bob, "pass_phrase".to_string()),
            true
        );

        assert_eq!(subscrypt.is_username_available("bobb".to_string()), true);
        assert_eq!(subscrypt.is_username_available("bob".to_string()), false);
        set_caller(callee, accounts.bob, 0);
        assert_eq!(subscrypt.get_username(), "bob".to_string());
        set_caller(callee, accounts.alice, 0);
        assert_eq!(subscrypt.get_username(), "alice".to_string());
        assert_eq!(
            subscrypt.get_plan_data(accounts.alice, 0),
            PlanConsts {
                duration: 60 * 60 * 24,
                price: 10000,
                max_refund_permille_policy: 50,
                disabled: false
            }
        );
    }

    #[ink::test]
    fn add_entry_works() {
        let mut subscrypt = Subscrypt::new();

        let accounts = ink_env::test::default_accounts::<ink_env::DefaultEnvironment>()
            .expect("Cannot get accounts");

        let callee =
            ink_env::test::get_current_contract_account_id::<ink_env::DefaultEnvironment>()
                .expect("Cannot get contract id");
        set_account_balance(callee, 90100);
        set_caller(callee, accounts.alice, 100);
        subscrypt_provider_register_routine(
            &mut subscrypt,
            accounts.alice,
            vec![
                60 * 60 * 24,
                60 * 60 * 24 * 30,
                60 * 60 * 24 * 300,
                60 * 60 * 24 * 31,
            ],
            vec![10000, 50000, 10000, 10000],
            vec![50, 100, 200, 100],
            "alice".to_string(),
            vec![
                vec!["key".to_string()],
                vec!["key".to_string()],
                vec!["key".to_string()],
                vec!["key".to_string()],
            ],
        );
        set_caller(callee, accounts.bob, 50000);

        subscrypt.subscribe(
            accounts.alice,
            1,
            [0; 32],
            "bob".to_string(),
            vec!["nothing important".to_string()],
        );
        set_caller(callee, accounts.bob, 10000);

        subscrypt.subscribe(
            accounts.alice,
            0,
            [0; 32],
            "bob".to_string(),
            vec!["nothing important".to_string()],
        );

        subscrypt.subscribe(
            accounts.alice,
            2,
            [0; 32],
            "bob".to_string(),
            vec!["nothing important".to_string()],
        );
        set_caller(callee, accounts.eve, 10000);

        subscrypt.subscribe(
            accounts.alice,
            0,
            [0; 32],
            "eve".to_string(),
            vec!["nothing important".to_string()],
        );
        subscrypt.subscribe(
            accounts.alice,
            3,
            [0; 32],
            "eve".to_string(),
            vec!["nothing important".to_string()],
        );
        assert_eq!(
            subscrypt
                .users
                .get(&accounts.bob)
                .unwrap()
                .list_of_providers
                .get(0)
                .unwrap(),
            &accounts.alice
        );
        set_caller(callee, accounts.alice, 0);

        subscrypt.process(accounts.alice, 1000);
    }
}
