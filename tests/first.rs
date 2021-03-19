#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::new_without_default)]
#![allow(non_snake_case)]
#![allow(unused_mut)]

#[path = "../src/lib.rs"]
mod subscrypt;

mod utils;

#[cfg(test)]
pub mod tests {
    use ink_lang as ink;
    use crate::subscrypt::subscrypt::SubsCrypt;
    use crate::subscrypt::subscrypt::LinkedList;
    use crate::utils::utils::{setCaller,setAccountBalance,subscrypt_provider_register_scenario1,subscrypt_edit_plan_scenario1,subscrypt_add_plan_scenario1};
    use ink_env::hash::{Sha2x256, HashOutput};

    #[ink::test]
    fn constructor_works() {
        let subsCrypt = SubsCrypt::new();
        assert_eq!(subsCrypt.provider_register_fee, 100);
    }

    #[ink::test]
    fn default_works() {
        let subsCrypt = SubsCrypt::default();
        assert_eq!(subsCrypt.provider_register_fee, 0);
    }
    #[ink::test]
    fn linked_List_works() {
        let linked = LinkedList::new();
        assert_eq!(linked.back, 0);
    }

    #[ink::test]
    fn linked_List_default_works() {
        let linked = LinkedList::default();
        assert_eq!(linked.back, 0);
    }

    #[ink::test]
    fn provider_register_works() {
        let mut subsCrypt = SubsCrypt::new();
        let accounts = ink_env::test::default_accounts::<ink_env::DefaultEnvironment>()
                .expect("Cannot get accounts");
        let callee = ink_env::test::get_current_contract_account_id::<ink_env::DefaultEnvironment>()
            .expect("Cannot get contract id");

        setCaller(callee, accounts.alice,100);
        subscrypt_provider_register_scenario1(&mut subsCrypt, accounts.alice,
                                              vec![60 * 60 * 24, 60 * 60 * 24 * 30],
                                              vec![2, 2],
                                              vec![10000, 50000],
                                              vec![50, 100]);
    }

    #[ink::test]
    #[should_panic]
    fn provider_register_works2() {
        let mut subsCrypt = SubsCrypt::new();
        let accounts = ink_env::test::default_accounts::<ink_env::DefaultEnvironment>()
                .expect("Cannot get accounts");
        let callee = ink_env::test::get_current_contract_account_id::<ink_env::DefaultEnvironment>()
            .expect("Cannot get contract id");
        setCaller(callee, accounts.alice, 90);
        subscrypt_provider_register_scenario1(&mut subsCrypt, accounts.alice,
                                              vec![60 * 60 * 24, 60 * 60 * 24 * 30],
                                              vec![2, 2],
                                              vec![10000, 50000],
                                              vec![50, 100]);
    }

    #[ink::test]
    #[should_panic]
    fn provider_register_works3() {
        let mut subsCrypt = SubsCrypt::new();
        let accounts = ink_env::test::default_accounts::<ink_env::DefaultEnvironment>()
                .expect("Cannot get accounts");
        let callee = ink_env::test::get_current_contract_account_id::<ink_env::DefaultEnvironment>()
            .expect("Cannot get contract id");
        setCaller(callee, accounts.alice, 90);
        subscrypt_provider_register_scenario1(&mut subsCrypt, accounts.alice,
                                              vec![60 * 60 * 24],
                                              vec![2, 2],
                                              vec![10000, 50000],
                                              vec![50, 100]);
    }

    #[ink::test]
    fn edit_plan_works() {
        let mut subsCrypt = SubsCrypt::new();
        let accounts = ink_env::test::default_accounts::<ink_env::DefaultEnvironment>()
                .expect("Cannot get accounts");
        let callee = ink_env::test::get_current_contract_account_id::<ink_env::DefaultEnvironment>()
            .expect("Cannot get contract id");

        setCaller(callee, accounts.alice, 100);
        subscrypt_provider_register_scenario1(&mut subsCrypt, accounts.alice,
                                              vec![60 * 60 * 24, 60 * 60 * 24 * 30],
                                              vec![2, 2],
                                              vec![10000, 50000],
                                              vec![50, 100]);
        subscrypt_edit_plan_scenario1(&mut subsCrypt,accounts.alice,1,
                                      60 * 60 * 24 * 10,
                                      3,
                                      100000,
                                      500,
                                      false);
    }

    #[ink::test]
    #[should_panic]
    fn edit_plan_works2() {
        let mut subsCrypt = SubsCrypt::new();
        let accounts = ink_env::test::default_accounts::<ink_env::DefaultEnvironment>()
                .expect("Cannot get accounts");
        let callee = ink_env::test::get_current_contract_account_id::<ink_env::DefaultEnvironment>()
            .expect("Cannot get contract id");
        setCaller(callee, accounts.alice, 100);
        subscrypt_provider_register_scenario1(&mut subsCrypt, accounts.alice,
                                              vec![60 * 60 * 24, 60 * 60 * 24 * 30],
                                              vec![2, 2],
                                              vec![10000, 50000],
                                              vec![50, 100]);
        subscrypt_edit_plan_scenario1(&mut subsCrypt,accounts.alice,2,
                                      60 * 60 * 24 * 10,
                                      3,
                                      100000,
                                      500,
                                      false);
    }
    #[ink::test]
    fn add_plan_works() {
        let mut subsCrypt = SubsCrypt::new();
        let accounts = ink_env::test::default_accounts::<ink_env::DefaultEnvironment>()
                .expect("Cannot get accounts");
        let callee = ink_env::test::get_current_contract_account_id::<ink_env::DefaultEnvironment>()
            .expect("Cannot get contract id");

        setCaller(callee, accounts.alice, 100);
        subscrypt_provider_register_scenario1(&mut subsCrypt, accounts.alice,
                                              vec![60 * 60 * 24, 60 * 60 * 24 * 30],
                                              vec![2, 2],
                                              vec![10000, 50000],
                                              vec![50, 100]);
        subscrypt_add_plan_scenario1(&mut subsCrypt, accounts.alice,vec![60 * 60 * 24 * 10],
                                     vec![3],
                                     vec![100000],
                                     vec![500])
    }

    #[ink::test]
    #[should_panic]
    fn add_plan_works2() {
        let mut subsCrypt = SubsCrypt::new();
        let accounts = ink_env::test::default_accounts::<ink_env::DefaultEnvironment>()
                .expect("Cannot get accounts");
        let callee = ink_env::test::get_current_contract_account_id::<ink_env::DefaultEnvironment>()
            .expect("Cannot get contract id");
        setCaller(callee, accounts.alice, 100);
        subscrypt_provider_register_scenario1(&mut subsCrypt, accounts.alice,
                                              vec![60 * 60 * 24, 60 * 60 * 24 * 30],
                                              vec![2, 2],
                                              vec![10000, 50000],
                                              vec![50, 100]);
        subscrypt_add_plan_scenario1(&mut subsCrypt, accounts.alice, vec![60 * 60 * 24 * 10],
                                     vec![3,2],
                                     vec![100000],
                                     vec![500]);
    }

    #[ink::test]
    fn change_disable_works() {
        let mut subsCrypt = SubsCrypt::new();
        let accounts = ink_env::test::default_accounts::<ink_env::DefaultEnvironment>()
                .expect("Cannot get accounts");
        let callee = ink_env::test::get_current_contract_account_id::<ink_env::DefaultEnvironment>()
            .expect("Cannot get contract id");

        setAccountBalance(callee,100);
        setCaller(callee, accounts.alice, 100);
        subscrypt_provider_register_scenario1(&mut subsCrypt, accounts.alice,
                                              vec![60 * 60 * 24, 60 * 60 * 24 * 30],
                                              vec![2, 2],
                                              vec![10000, 50000],
                                              vec![50, 100]);

        assert_eq!(subsCrypt.providers.get(&accounts.alice).unwrap().money_address, accounts.alice);
        subsCrypt.change_disable(1);
        assert_eq!(subsCrypt.providers.get(&accounts.alice).unwrap().plans.get(1).unwrap().disabled, true);

        subsCrypt.change_disable(1);
        assert_eq!(subsCrypt.providers.get(&accounts.alice).unwrap().plans.get(1).unwrap().disabled, false);
    }

    #[ink::test]
    fn subscribe_works() {
        let mut subsCrypt = SubsCrypt::new();
        let accounts = ink_env::test::default_accounts::<ink_env::DefaultEnvironment>()
                .expect("Cannot get accounts");
        let callee = ink_env::test::get_current_contract_account_id::<ink_env::DefaultEnvironment>()
            .expect("Cannot get contract id");

        setAccountBalance(callee,50100);
        setCaller(callee, accounts.alice, 100);
        subscrypt_provider_register_scenario1(&mut subsCrypt, accounts.alice,
                                              vec![60 * 60 * 24, 60 * 60 * 24 * 30],
                                              vec![2, 2],
                                              vec![10000, 50000],
                                              vec![50, 100]);

        setCaller(callee, accounts.bob, 50000);

        subsCrypt.subscribe(
            accounts.alice,
            1,
            [0; 32],
            "nothing important".to_string(),
        );
        assert_eq!(subsCrypt.users.get(&accounts.bob).unwrap().list_of_providers.get(0).unwrap(), &accounts.alice);
    }

    #[ink::test]
    #[should_panic]
    fn subscribe_works2() {
        let mut subsCrypt = SubsCrypt::new();
        let accounts = ink_env::test::default_accounts::<ink_env::DefaultEnvironment>()
                .expect("Cannot get accounts");
        let callee = ink_env::test::get_current_contract_account_id::<ink_env::DefaultEnvironment>()
            .expect("Cannot get contract id");

        setAccountBalance(callee,50100);
        setCaller(callee, accounts.alice, 100);
        subscrypt_provider_register_scenario1(&mut subsCrypt, accounts.alice,
                                              vec![60 * 60 * 24, 60 * 60 * 24 * 30],
                                              vec![2, 2],
                                              vec![10000, 50000],
                                              vec![50, 100]);
        setCaller(callee, accounts.bob, 49500);

        subsCrypt.subscribe(
            accounts.alice,
            1,
            [0; 32],
            "nothing important".to_string(),
        );
        assert_eq!(subsCrypt.users.get(&accounts.bob).unwrap().list_of_providers.get(0).unwrap(), &accounts.alice);
    }

    #[ink::test]
    fn withdraw_works() {
        let mut subsCrypt = SubsCrypt::new();

        let accounts =
            ink_env::test::default_accounts::<ink_env::DefaultEnvironment>()
                .expect("Cannot get accounts");

        let callee = ink_env::test::get_current_contract_account_id::<ink_env::DefaultEnvironment>()
            .expect("Cannot get contract id");

        setAccountBalance(callee,50100);
        setCaller(callee, accounts.alice, 100);
        subscrypt_provider_register_scenario1(&mut subsCrypt, accounts.alice,
                                              vec![60 * 60 * 24, 60 * 60 * 24 * 30],
                                              vec![2, 2],
                                              vec![10000, 50000],
                                              vec![50, 100]);

        setCaller(callee, accounts.bob, 50000);
        subsCrypt.subscribe(
            accounts.alice,
            1,
            [0; 32],
            "nothing important".to_string(),
        );
        assert_eq!(subsCrypt.users.get(&accounts.bob).unwrap().list_of_providers.get(0).unwrap(), &accounts.alice);
        setCaller(callee, accounts.alice, 0);
        subsCrypt.withdraw();

    }

    #[ink::test]
    #[should_panic]
    fn withdraw_works2() {
        let mut subsCrypt = SubsCrypt::new();

        let accounts =
            ink_env::test::default_accounts::<ink_env::DefaultEnvironment>()
                .expect("Cannot get accounts");

        let callee = ink_env::test::get_current_contract_account_id::<ink_env::DefaultEnvironment>()
            .expect("Cannot get contract id");

        setAccountBalance(callee,50100);
        setCaller(callee, accounts.alice, 100);
        subscrypt_provider_register_scenario1(&mut subsCrypt, accounts.alice,
                                              vec![60 * 60 * 24, 60 * 60 * 24 * 30],
                                              vec![2, 2],
                                              vec![10000, 50000],
                                              vec![50, 100]);

        setCaller(callee, accounts.bob, 50000);
        subsCrypt.subscribe(
            accounts.alice,
            1,
            [0; 32],
            "nothing important".to_string(),
        );
        assert_eq!(subsCrypt.users.get(&accounts.bob).unwrap().list_of_providers.get(0).unwrap(), &accounts.alice);

        setCaller(callee, accounts.eve, 0);
        subsCrypt.withdraw();

    }

    #[ink::test]
    fn refund_works() {
        let mut subsCrypt = SubsCrypt::new();

        let accounts =
            ink_env::test::default_accounts::<ink_env::DefaultEnvironment>()
                .expect("Cannot get accounts");
        let callee = ink_env::test::get_current_contract_account_id::<ink_env::DefaultEnvironment>()
            .expect("Cannot get contract id");

        setAccountBalance(callee,50100);
        setCaller(callee, accounts.alice, 100);
        subscrypt_provider_register_scenario1(&mut subsCrypt, accounts.alice,
                                              vec![60 * 60 * 24, 60 * 60 * 24 * 30],
                                              vec![2, 2],
                                              vec![10000, 50000],
                                              vec![50, 100]);

        setCaller(callee, accounts.bob, 50000);

        subsCrypt.subscribe(
            accounts.alice,
            1,
            [0; 32],
            "nothing important".to_string(),
        );
        assert_eq!(subsCrypt.records.get(&(accounts.bob, accounts.alice)).unwrap().subscription_records.get(0).unwrap().refunded, false);

        subsCrypt.refund(
            accounts.alice,
            1,
        );
        assert_eq!(subsCrypt.records.get(&(accounts.bob, accounts.alice)).unwrap().subscription_records.get(0).unwrap().refunded, true);
    }

    #[ink::test]
    #[should_panic]
    fn refund_works2() {
        let mut subsCrypt = SubsCrypt::new();
        let accounts = ink_env::test::default_accounts::<ink_env::DefaultEnvironment>()
                .expect("Cannot get accounts");
        let callee = ink_env::test::get_current_contract_account_id::<ink_env::DefaultEnvironment>()
            .expect("Cannot get contract id");

        setAccountBalance(callee,50100);
        setCaller(callee, accounts.alice, 100);
        subscrypt_provider_register_scenario1(&mut subsCrypt, accounts.alice,
                                              vec![60 * 60 * 24, 60 * 60 * 24 * 30],
                                              vec![2, 2],
                                              vec![10000, 50000],
                                              vec![50, 100]);
        setCaller(callee, accounts.bob, 50000);

        subsCrypt.subscribe(
            accounts.alice,
            1,
            [0; 32],
            "nothing important".to_string(),
        );
        assert_eq!(subsCrypt.records.get(&(accounts.bob, accounts.alice)).unwrap().subscription_records.get(0).unwrap().refunded, false);

        subsCrypt.refund(
            accounts.alice,
            1,
        );
        subsCrypt.refund(
            accounts.alice,
            1,
        );
    }

    #[ink::test]
    fn check_subscription_works() {
        let mut subsCrypt = SubsCrypt::new();

        let accounts =
            ink_env::test::default_accounts::<ink_env::DefaultEnvironment>()
                .expect("Cannot get accounts");
        let callee = ink_env::test::get_current_contract_account_id::<ink_env::DefaultEnvironment>()
            .expect("Cannot get contract id");

        setAccountBalance(callee,50100);
        setCaller(callee, accounts.alice, 100);
        subscrypt_provider_register_scenario1(&mut subsCrypt, accounts.alice,
                                              vec![60 * 60 * 24, 60 * 60 * 24 * 30],
                                              vec![2, 2],
                                              vec![10000, 50000],
                                              vec![50, 100]);

        setCaller(callee, accounts.bob, 50000);

        subsCrypt.subscribe(
            accounts.alice,
            1,
            [0; 32],
            "nothing important".to_string(),
        );
        assert_eq!(subsCrypt.records.get(&(accounts.bob, accounts.alice)).unwrap().subscription_records.get(0).unwrap().refunded, false);
        assert_eq!(subsCrypt.check_subscription(accounts.bob,accounts.alice,1),true);
    }


    #[ink::test]
    fn retrieve_data_with_wallet_works() {
        let mut subsCrypt = SubsCrypt::new();
        let accounts = ink_env::test::default_accounts::<ink_env::DefaultEnvironment>()
                .expect("Cannot get accounts");
        let callee = ink_env::test::get_current_contract_account_id::<ink_env::DefaultEnvironment>()
            .expect("Cannot get contract id");
        setAccountBalance(callee,50100);
        setCaller(callee, accounts.alice, 100);
        subscrypt_provider_register_scenario1(&mut subsCrypt, accounts.alice,
                                              vec![60 * 60 * 24, 60 * 60 * 24 * 30],
                                              vec![2, 2],
                                              vec![10000, 50000],
                                              vec![50, 100]);

        setCaller(callee, accounts.bob, 50000);

        subsCrypt.subscribe(
            accounts.alice,
            1,
            [0; 32],
            "nothing important".to_string(),
        );
        assert_eq!(subsCrypt.records.get(&(accounts.bob, accounts.alice)).unwrap().subscription_records.get(0).unwrap().refunded, false);
        let s = subsCrypt.retrieve_data_with_wallet(accounts.alice);
        assert_eq!(s[0].provider,accounts.alice);
        assert_eq!(s[0].plan_index,1);
        assert_eq!(s[0].plan.duration,60 * 60 * 24 * 30);
    }

    #[ink::test]
    fn retrieve_whole_data_with_wallet_works() {
        let mut subsCrypt = SubsCrypt::new();
        let accounts = ink_env::test::default_accounts::<ink_env::DefaultEnvironment>()
                .expect("Cannot get accounts");
        let callee = ink_env::test::get_current_contract_account_id::<ink_env::DefaultEnvironment>()
            .expect("Cannot get contract id");
        setAccountBalance(callee,50100);
        setCaller(callee, accounts.alice, 100);
        subscrypt_provider_register_scenario1(&mut subsCrypt, accounts.alice,
                                              vec![60 * 60 * 24, 60 * 60 * 24 * 30],
                                              vec![2, 2],
                                              vec![10000, 50000],
                                              vec![50, 100]);

        setCaller(callee, accounts.bob, 50000);

        subsCrypt.subscribe(
            accounts.alice,
            1,
            [0; 32],
            "nothing important".to_string(),
        );
        assert_eq!(subsCrypt.records.get(&(accounts.bob, accounts.alice)).unwrap().subscription_records.get(0).unwrap().refunded, false);
        let s = subsCrypt.retrieve_whole_data_with_wallet();
        assert_eq!(s[0].provider,accounts.alice);
        assert_eq!(s[0].plan_index,1);
        assert_eq!(s[0].plan.duration, 60 * 60 * 24 * 30);
    }

    #[ink::test]
    fn retrieve_data_with_password_works() {
        let mut subsCrypt = SubsCrypt::new();
        let accounts = ink_env::test::default_accounts::<ink_env::DefaultEnvironment>()
                .expect("Cannot get accounts");
        let callee = ink_env::test::get_current_contract_account_id::<ink_env::DefaultEnvironment>()
            .expect("Cannot get contract id");
        setAccountBalance(callee,50100);
        setCaller(callee, accounts.alice, 100);
        subscrypt_provider_register_scenario1(&mut subsCrypt, accounts.alice,
                                              vec![60 * 60 * 24, 60 * 60 * 24 * 30],
                                              vec![2, 2],
                                              vec![10000, 50000],
                                              vec![50, 100]);

        setCaller(callee, accounts.bob, 50000);
        let t:String= "token".to_string();
        let p:String="pass_phrase".to_string();
        let encodable = [
            t,
            p
        ];
        let mut output = <Sha2x256 as HashOutput>::Type::default(); // 256-bit buffer
        ink_env::hash_encoded::<Sha2x256, _>(&encodable, &mut output);

        subsCrypt.subscribe(
            accounts.alice,
            1,
            output,
            "nothing important".to_string(),
        );
        assert_eq!(subsCrypt.records.get(&(accounts.bob, accounts.alice)).unwrap().subscription_records.get(0).unwrap().refunded, false);
        let s = subsCrypt.retrieve_data_with_password(accounts.bob, accounts.alice, "token".parse().unwrap(), "pass_phrase".parse().unwrap());
        assert_eq!(s[0].provider,accounts.alice);
        assert_eq!(s[0].plan_index,1);
        assert_eq!(s[0].plan.duration, 60 * 60 * 24 * 30);

    }

    #[ink::test]
    fn retrieve_whole_data_with_password_works() {
        let mut subsCrypt = SubsCrypt::new();
        let accounts = ink_env::test::default_accounts::<ink_env::DefaultEnvironment>()
                .expect("Cannot get accounts");
        let callee = ink_env::test::get_current_contract_account_id::<ink_env::DefaultEnvironment>()
            .expect("Cannot get contract id");
        setAccountBalance(callee,50100);
        setCaller(callee, accounts.alice, 100);
        subscrypt_provider_register_scenario1(&mut subsCrypt, accounts.alice,
                                              vec![60 * 60 * 24, 60 * 60 * 24 * 30],
                                              vec![2, 2],
                                              vec![10000, 50000],
                                              vec![50, 100]);

        setCaller(callee, accounts.bob, 50000);

        let t:String= "token".to_string();
        let p:String="pass_phrase".to_string();
        let encodable = [
            t,
            p
        ];
        let mut output = <Sha2x256 as HashOutput>::Type::default(); // 256-bit buffer
        ink_env::hash_encoded::<Sha2x256, _>(&encodable, &mut output);

        subsCrypt.subscribe(
            accounts.alice,
            1,
            output,
            "nothing important".to_string(),
        );
        assert_eq!(subsCrypt.records.get(&(accounts.bob, accounts.alice)).unwrap().subscription_records.get(0).unwrap().refunded, false);
        let s = subsCrypt.retrieve_whole_data_with_password(accounts.bob, "token".parse().unwrap(), "pass_phrase".parse().unwrap());
        assert_eq!(s[0].provider,accounts.alice);
        assert_eq!(s[0].plan_index,1);
        assert_eq!(s[0].plan.duration, 60 * 60 * 24 * 30);

    }

    #[ink::test]
    fn check_auth_works() {
        let mut subsCrypt = SubsCrypt::new();
        let accounts = ink_env::test::default_accounts::<ink_env::DefaultEnvironment>()
                .expect("Cannot get accounts");
        let callee = ink_env::test::get_current_contract_account_id::<ink_env::DefaultEnvironment>()
            .expect("Cannot get contract id");
        setAccountBalance(callee,50100);
        setCaller(callee, accounts.alice, 100);
        subscrypt_provider_register_scenario1(&mut subsCrypt, accounts.alice,
                                              vec![60 * 60 * 24, 60 * 60 * 24 * 30],
                                              vec![2, 2],
                                              vec![10000, 50000],
                                              vec![50, 100]);

        setCaller(callee, accounts.bob, 50000);

        let t:String= "token".to_string();
        let p:String="pass_phrase".to_string();
        let encodable = [
            t,
            p
        ];
        let mut output = <Sha2x256 as HashOutput>::Type::default(); // 256-bit buffer
        ink_env::hash_encoded::<Sha2x256, _>(&encodable, &mut output);

        subsCrypt.subscribe(
            accounts.alice,
            1,
            output,
            "nothing important".to_string(),
        );
        assert_eq!(subsCrypt.records.get(&(accounts.bob, accounts.alice)).unwrap().subscription_records.get(0).unwrap().refunded, false);
        let s = subsCrypt.check_auth(accounts.bob,accounts.alice, "token".parse().unwrap(), "pass_phrase".parse().unwrap());
        assert_eq!(s,true);
    }

    #[ink::test]
    fn add_entry_works() {
        let mut subsCrypt = SubsCrypt::new();

        let accounts =
            ink_env::test::default_accounts::<ink_env::DefaultEnvironment>()
                .expect("Cannot get accounts");

        let callee = ink_env::test::get_current_contract_account_id::<ink_env::DefaultEnvironment>()
            .expect("Cannot get contract id");
        setAccountBalance(callee,90100);
        setCaller(callee, accounts.alice, 100);
        subscrypt_provider_register_scenario1(&mut subsCrypt, accounts.alice,
                                              vec![60 * 60 * 24, 60 * 60 * 24 * 30,60 * 60 * 24 * 300,60 * 60 * 24 * 31],
                                              vec![2, 2,2,2],
                                              vec![10000, 50000,10000,10000],
                                              vec![50, 100,200,100]);
        setCaller(callee, accounts.bob, 50000);

        subsCrypt.subscribe(
            accounts.alice,
            1,
            [0; 32],
            "nothing important".to_string(),
        );
        setCaller(callee, accounts.bob, 10000);

        subsCrypt.subscribe(
            accounts.alice,
            0,
            [0; 32],
            "nothing important".to_string(),
        );

        subsCrypt.subscribe(
            accounts.alice,
            2,
            [0; 32],
            "nothing important".to_string(),
        );
        setCaller(callee, accounts.eve, 10000);

        subsCrypt.subscribe(
            accounts.alice,
            0,
            [0; 32],
            "nothing important".to_string(),
        );
        subsCrypt.subscribe(
            accounts.alice,
            3,
            [0; 32],
            "nothing important".to_string(),
        );
        assert_eq!(subsCrypt.users.get(&accounts.bob).unwrap().list_of_providers.get(0).unwrap(), &accounts.alice);
        setCaller(callee, accounts.alice, 0);

        subsCrypt.process(accounts.alice,1000);
    }
}