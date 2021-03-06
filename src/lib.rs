#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::new_without_default)]
#![allow(non_snake_case)]
#![allow(unused_mut)]
use ink_lang as ink;

#[ink::contract]
mod subscrypt {
    use ink_storage::collections::HashMap;
    use ink_env::{Error as Er, AccountId as Account, Error};
    use ink_env::hash::{Sha2x256};
    use ink_prelude::vec::Vec;
    use ink_storage::{
        traits::{
            PackedLayout,
            SpreadLayout,
        },
    };
    use core::convert::TryInto;
    use ink_prelude::string::String;

    /// this struct represents a subscription record
    /// # fields:
    /// * provider
    /// * plan
    /// * plan_index
    /// * subscription_time : this stores start time of subscription (used in linkedList)
    /// * meta_data_encrypted
    #[derive(scale::Encode, scale::Decode, SpreadLayout, PackedLayout, Debug, scale_info::TypeInfo)]
    pub struct SubscriptionRecord {
        provider: Account,
        plan: PlanConsts,
        plan_index: u128,
        subscription_time: u64,
        meta_data_encrypted: String,        //encrypted Data with public key of provider
        refunded: bool,
    }

    /// this struct stores user plan records
    /// # fields:
    /// * subscription_records : history of subscription
    /// * pass_hash : hash of (token + pass_phrase) for authenticating user without wallet
    #[derive(scale::Encode, scale::Decode, SpreadLayout, PackedLayout, Debug, scale_info::TypeInfo)]
    struct PlanRecord {
        subscription_records: Vec<SubscriptionRecord>,
        pass_hash: [u8; 32],
    }


    #[derive(scale::Encode, scale::Decode, PackedLayout, SpreadLayout, Debug, scale_info::TypeInfo,Clone, Copy)]
    struct PlanConsts {
        duration: u64,
        active_session_limit: u128,
        price: u128,
        max_refund_percent_policy: u128,
        disabled: bool,
    }

    /// this struct represents a provider
    /// # fields:
    /// * plans
    /// * money_address : provider earned money will be sent to this address
    /// * payment_manager : struct for handling refund requests
    #[derive(scale::Encode, scale::Decode, PackedLayout, SpreadLayout, Debug, scale_info::TypeInfo)]
    struct Provider {
        plans: Vec<PlanConsts>,
        money_address: Account,
        payment_manager: LinkedList,
    }

    /// this struct represents a user
    /// # fields:
    /// * list_of_providers
    /// * joined_date
    /// * subs_crypt_pass_hash : pass hash for retrieve data
    #[derive(scale::Encode, scale::Decode, SpreadLayout, PackedLayout, Debug, scale_info::TypeInfo)]
    struct User {
        list_of_providers: Vec<Account>,
        joined_date: u64,
        subs_crypt_pass_hash: [u8; 32],
        a: Vec<Account>,
    }

    /// struct for handling payments of refund
    /// * head
    /// * back
    /// * length
    #[derive(scale::Encode, scale::Decode, PackedLayout, SpreadLayout, Debug, scale_info::TypeInfo)]
    struct LinkedList {
        head: u64,
        back: u64,
        length: u128,
    }

    /// struct that represents a payment admission
    #[derive(scale::Encode, scale::Decode, PackedLayout, SpreadLayout, Debug, scale_info::TypeInfo)]
    struct PaymentAdmission {
        number: u128,
        next_day: u64,
    }

    /// main struct of contract
    /// # fields:
    /// * index_counter : counter for index_to_address hashmap
    /// * start_time : start time of the contract
    /// * providers : the hashmap that stores providers data
    /// * users : the hashmap that stores users data
    /// * paymentAdmissions : the hashmap that stores payment admissions data
    /// * records : the hashmap that stores user's subscription records data
    /// * plan_index_to_record_index : the hashmap that stores user's last plan index for each plan index
    #[ink(storage)]
    pub struct SubsCrypt {
        index_counter: u128,
        start_time: u64,
        provider_register_fee: u128,
        providers: HashMap<Account, Provider>, // (provider AccountId) -> provider data
        users: HashMap<Account, User>, // (user AccountId) -> user data
        paymentAdmissions: HashMap<(Account, u64), PaymentAdmission>, // (provider AccountId , day_id) -> payment admission
        records: HashMap<(Account, Account), PlanRecord>, // (user AccountId, provider AccountId) -> PlanRecord struct
        plan_index_to_record_index: HashMap<(Account, Account, u128), u128>, // (user AccountId, provider AccountId, plan_index) -> index
    }

    impl SubsCrypt {
        /// constructor:
        /// initializes the main struct data
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {
                index_counter: 0,
                start_time: Self::env().block_timestamp(),
                provider_register_fee: 100,
                providers: HashMap::new(),
                users: ink_storage::collections::HashMap::new(),
                paymentAdmissions: ink_storage::collections::HashMap::new(),
                records: ink_storage::collections::HashMap::new(),
                plan_index_to_record_index: ink_storage::collections::HashMap::new(),
            }
        }

        /// constructor:
        /// initializes the main struct data
        #[ink(constructor)]
        pub fn default() -> Self {
            Self {
                index_counter: 0,
                start_time: 0,
                provider_register_fee: 0,
                providers: Default::default(),
                users: Default::default(),
                paymentAdmissions: Default::default(),
                records: Default::default(),
                plan_index_to_record_index: Default::default(),
            }
        }

        /// provider_register : add a provider to contract storage
        /// # arguments:
        /// * durations
        /// * active_session_limits
        /// * prices
        /// * max_refund_percent_policies
        /// * address : money detination address for this provider
        #[ink(message, payable)]
        pub fn provider_register(&mut self, durations: Vec<u64>, active_session_limits: Vec<u128>, prices: Vec<u128>, max_refund_percent_policies: Vec<u128>, address: Account) {

            let caller = self.env().caller();
            assert!(self.env().transferred_balance() >= self.provider_register_fee, "You have to pay a minimum amount to register in the contract!");
            assert!(!self.providers.contains_key(&caller), "You can not register again in the contract!");

            let mut provider = Provider {
                plans: Vec::new(),
                money_address: address,
                payment_manager: LinkedList::new(),
            };
            self.providers.insert(caller, provider);

            self.add_plan(durations, active_session_limits, prices, max_refund_percent_policies);
        }


        /// add_plan : add plans to provider storage
        /// # arguments:
        /// * durations
        /// * active_session_limits
        /// * prices
        /// * max_refund_percent_policies
        #[ink(message)]
        pub fn add_plan(&mut self, durations: Vec<u64>, active_session_limits: Vec<u128>, prices: Vec<u128>, max_refund_percent_policies: Vec<u128>) {
            assert_eq!(durations.len() , active_session_limits.len());
            assert_eq!(prices.len() , active_session_limits.len());
            assert_eq!(max_refund_percent_policies.len() , active_session_limits.len());

            let caller = self.env().caller();
            assert!(self.providers.contains_key(&caller), "You should first register in the contract!");
            let provider = self.providers.get_mut(&caller).unwrap();
            for i in 0..durations.len() {
                let cons = PlanConsts {
                    duration: durations[i],
                    active_session_limit: active_session_limits[i],
                    price: prices[i],
                    max_refund_percent_policy: max_refund_percent_policies[i],
                    disabled: false,
                };
                provider.plans.push(cons);
            }
        }

        /// edit_plan : edit a plan
        /// # arguments:
        /// * plan_index
        /// * duration
        /// * active_session_limit
        /// * prices
        /// * max_refund_percent_policies
        #[ink(message)]
        pub fn edit_plan(&mut self, plan_index: u128, duration: u64, active_session_limit: u128, price: u128, max_refund_percent_policy: u128, disabled: bool) {
            let number: usize = plan_index.try_into().unwrap();
            let caller = self.env().caller();
            assert!(self.providers.get(&caller).unwrap().plans.len() > plan_index.try_into().unwrap(), "please select a valid plan");
            let provider = self.providers.get_mut(&caller).unwrap();
            let mut plan: &mut PlanConsts = provider.plans.get_mut(number).unwrap();


            plan.duration = duration;
            plan.active_session_limit = active_session_limit;
            plan.price = price;
            plan.max_refund_percent_policy = max_refund_percent_policy;
            plan.disabled = disabled;
        }

        /// change_disable : disable and enable a plan
        /// # arguments:
        /// * plan_index
        #[ink(message)]
        pub fn change_disable(&mut self, plan_index: u128) {
            let caller = self.env().caller();
            let number: usize = plan_index.try_into().unwrap();
            assert!(self.providers.get(&caller).unwrap().plans.len() > plan_index.try_into().unwrap(), "please select a valid plan");
            let x = self.providers.get(&caller).unwrap().plans[number].disabled;
            self.providers.get_mut(&caller).unwrap().plans[number].disabled = !x;
        }

        /// subscribe : users call this function to subscribe a plan
        /// # arguments:
        /// * provider_address
        /// * plan_index
        /// * pass : hash of (token + pass_phrase)
        /// * metadata : extra metadata of the plan
        #[ink(message, payable)]
        pub fn subscribe(&mut self, provider_address: Account, plan_index: u128, pass: [u8; 32], metadata: String) {
            let caller: Account = self.env().caller();
            let time: u64 = self.env().block_timestamp();
            let value: u128 = self.env().transferred_balance();
            let number: usize = plan_index.try_into().unwrap();
            let consts: PlanConsts = self.providers.get(&provider_address).unwrap().plans[number];

            assert_eq!(consts.price, value, "You have to pay exact plan price");
            assert!(!consts.disabled, "Plan is currently disabled by provider");

            assert!(!self.check_subscription(caller, provider_address, plan_index), "You are already subscribed to this plan!");
            assert!(self.providers.contains_key(&provider_address), "Provider not existed in the contract!");
            assert!(self.providers.get(&provider_address).unwrap().plans.len() > plan_index.try_into().unwrap(), "Wrong plan index!");
            if !self.users.contains_key(&caller) {
                self.users.insert(caller, User {
                    list_of_providers: Vec::new(),
                    joined_date: self.env().block_timestamp(),
                    subs_crypt_pass_hash: pass,
                    a: Vec::new(),
                });
            }

            let user: &mut User = self.users.get_mut(&caller).unwrap();
            if !self.records.contains_key(&(caller, provider_address)) {
                user.list_of_providers.push(provider_address);
                self.records.insert((caller, provider_address), PlanRecord {
                    subscription_records: Vec::new(),
                    pass_hash: pass,
                });
            }
            let mut plan_record: &mut PlanRecord = self.records.get_mut(&(caller, provider_address)).unwrap();
            self.plan_index_to_record_index.insert((caller, provider_address, plan_index), plan_record.subscription_records.len().try_into().unwrap());
            let record: SubscriptionRecord = SubscriptionRecord {
                provider: provider_address,
                plan: consts,
                plan_index,
                subscription_time: time,
                meta_data_encrypted: metadata,
                refunded: false,
            };
            plan_record.subscription_records.push(record);

            let addr: &Account = &self.providers.get(&provider_address).unwrap().money_address;
            // send money to money_address (1000 - plan.max_refund_percent_policy) / 1000;
            assert_eq!(self.transfer(*addr, consts.price * (1000 - consts.max_refund_percent_policy) / 1000),Ok(()));
            let start_time = self.start_time;
            let block_time = self.env().block_timestamp();
            let transferred_balance= self.env().transferred_balance();
            self.add_entry(provider_address, (block_time + consts.duration - start_time) / 86400, (transferred_balance * consts.max_refund_percent_policy) / 1000)
        }

        /// set_subscrypt_pass : users can change their pass_hash
        /// # arguments:
        /// * pass : hash of (token + pass_phrase)
        #[ink(message)]
        pub fn set_subscrypt_pass(&mut self, pass: [u8; 32]) {
            assert!(self.users.contains_key(&self.env().caller()));
            self.users.get_mut(&self.env().caller()).unwrap().subs_crypt_pass_hash = pass;
        }

        /// withdraw : providers call this function to claim their unlocked money
        #[ink(message)]
        pub fn withdraw(&mut self) -> u128 {
            assert!(self.providers.contains_key(&self.env().caller()), "You are not a registered provider");
            let caller: Account = self.env().caller();
            let paid: u128 = self.process(caller, self.env().block_timestamp() / 86400);
            if paid > 0 {
                assert_eq!(self.transfer(caller, paid),Ok(()));
            }
            paid
        }

        /// refund : users can refund their money back
        /// # arguments:
        /// * provider_address
        /// * plan_index
        #[ink(message)]
        pub fn refund(&mut self, provider_address: Account, plan_index: u128) {
            let caller: Account = self.env().caller();
            let time: u64 = self.env().block_timestamp();
            assert!(self.check_subscription(caller,provider_address, plan_index),"You are not in this plan or already refunded");
            assert!(self.plan_index_to_record_index.contains_key(&(caller, provider_address, plan_index)));
            let last_index: &u128 = self.plan_index_to_record_index.get(&(caller, provider_address, plan_index)).unwrap();
            let number: usize = (*last_index).try_into().unwrap();
            let record: &SubscriptionRecord = self.records.get(&(caller, provider_address)).unwrap().subscription_records.get(number).unwrap();

            // it shows how much of your subscription plan is passed in range of 0 to 1000 and more (if you refund after the plan is finished)
            let mut spent_time_percent: u128 = ((time - record.subscription_time) * 1000 / (record.plan.duration)).try_into().unwrap();
            // to avoid refund after plan is finished
            assert!(spent_time_percent <= 1000);

            // amount of time that is remained till the end of your plan = 1000 - spent_time_percent

            let mut remained_time_percent = 1000 - spent_time_percent;
            if remained_time_percent > record.plan.max_refund_percent_policy {
                // in this case the customer wants to refund very early so he want to get
                // more than the amount of refund policy, so we can only give back just
                // max_refund_percent_policy of his/her subscription. Whole locked money will go directly to
                // account of the customer
                remained_time_percent = record.plan.max_refund_percent_policy;
            } else {
                // in this case the customer wants to refund, but he/she used most of his subscription time
                // and now he/she will get portion of locked money, and the provider will get the rest of money
                let provider_portion_locked_money: u128 = (record.plan.max_refund_percent_policy - remained_time_percent) * record.plan.price / 1000;
                assert_eq!(self.transfer(self.providers.get(&provider_address).unwrap().money_address, provider_portion_locked_money),Ok(()));
            }

            let customer_portion_locked_money: u128 = remained_time_percent * record.plan.price / 1000;
            assert_eq!(self.transfer(caller, customer_portion_locked_money),Ok(()));


            let passed_time=record.plan.duration + record.subscription_time - self.start_time;
            let amount = record.plan.price * record.plan.max_refund_percent_policy;
            self.remove_entry(provider_address, passed_time / 86400,  amount/ 1000);
            self.records.get_mut(&(caller, provider_address)).unwrap().subscription_records.get_mut(number).unwrap().refunded = true;
        }

        /// check_auth : this function authenticates the user with token and pass_phrase
        /// # arguments:
        /// * user
        /// * provider
        /// * plan_index
        /// * token and pass_phrase : these are for authenticating
        #[ink(message)]
        pub fn check_auth(&self, user: Account, provider: Account, token: String, pass_phrase: String) -> bool {
            if !self.records.contains_key(&(user, provider)) {
                return false;
            }
            let encodable = [
                token,
                pass_phrase
            ];
            let encoded = self.env().hash_encoded::<Sha2x256, _>(&encodable);
            if encoded == self.records.get(&(user, provider)).unwrap().pass_hash {
                return true;
            }
            false
        }

        /// retrieve_whole_data_with_password : retrieve all user data when wallet is not available.
        /// # arguments:
        /// * user
        /// * token and phrase : subscrypt passphrase
        /// # return value : vector of subscription records
        #[ink(message)]
        pub fn retrieve_whole_data_with_password(&self, user: Account, token: String, phrase: String) -> Vec<SubscriptionRecord> {
            let encodable = [
                token,
                phrase
            ];
            let encoded = self.env().hash_encoded::<Sha2x256, _>(&encodable);
            assert_eq!(encoded, self.users.get(&user).unwrap().subs_crypt_pass_hash, "Wrong auth");
            self.retrieve_whole_data(user)
        }

        /// retrieve_whole_data_with_wallet : retrieve all user data.
        /// # return value : vector of subscription records
        #[ink(message)]
        pub fn retrieve_whole_data_with_wallet(&self) -> Vec<SubscriptionRecord> {
            let caller: Account = self.env().caller();
            self.retrieve_whole_data(caller)
        }

        fn retrieve_whole_data(&self, caller: Account) -> Vec<SubscriptionRecord> {
            assert!(self.users.contains_key(&caller));
            let mut data: Vec<SubscriptionRecord> = Vec::new();
            let user: &User = self.users.get(&caller).unwrap();
            for i in 0..user.list_of_providers.len() {
                let plan_records: &PlanRecord = self.records.get(&(caller, *&user.list_of_providers[i])).unwrap();
                for i in 0..plan_records.subscription_records.len() {
                    let k = SubscriptionRecord {
                        provider: plan_records.subscription_records[i].provider,
                        plan: plan_records.subscription_records[i].plan,
                        plan_index: plan_records.subscription_records[i].plan_index,
                        subscription_time: plan_records.subscription_records[i].subscription_time,
                        meta_data_encrypted: plan_records.subscription_records[i].meta_data_encrypted.clone(),
                        refunded: plan_records.subscription_records[i].refunded,
                    };
                    data.push(k);
                }
            }
            data
        }

        /// retrieve_data_with_password : retrieve user data when wallet is not available.
        /// # arguments:
        /// * user
        /// * provider_address
        /// * token and phrase
        /// # return value: vector of subscription records from a specific provider
        #[ink(message)]
        pub fn retrieve_data_with_password(&self, user: Account, provider_address: Account, token: String, phrase: String) -> Vec<SubscriptionRecord> {
            let encodable = [
                token,
                phrase
            ];
            let encoded = self.env().hash_encoded::<Sha2x256, _>(&encodable);
            assert_eq!(encoded, self.records.get(&(user, provider_address)).unwrap().pass_hash, "Wrong auth");
            self.retrieve_data(user, provider_address)
        }

        /// retrieve_data_with_wallet : retrieve user data whit wallet.
        /// # arguments:
        /// * provider_address
        /// # return value: vector of subscription records from a specific provider
        #[ink(message)]
        pub fn retrieve_data_with_wallet(&self, provider_address: Account) -> Vec<SubscriptionRecord> {
            let caller: Account = self.env().caller();
            self.retrieve_data( caller,provider_address)
        }

        fn retrieve_data(&self, caller: Account, provider_address: Account) -> Vec<SubscriptionRecord> {
            assert!(self.users.contains_key(&caller));
            assert!(self.records.contains_key(&(caller, provider_address)));
            let mut data: Vec<SubscriptionRecord> = Vec::new();

            let plan_records: &PlanRecord = self.records.get(&(caller, provider_address)).unwrap();
            for i in 0..plan_records.subscription_records.len() {
                let k = SubscriptionRecord {
                    provider: plan_records.subscription_records[i].provider,
                    plan: plan_records.subscription_records[i].plan,
                    plan_index: plan_records.subscription_records[i].plan_index,
                    subscription_time: plan_records.subscription_records[i].subscription_time,
                    //meta_data_encrypted: plan_records.subscription_records[i].meta_data_encrypted,
                    meta_data_encrypted: plan_records.subscription_records[i].meta_data_encrypted.clone(),
                    refunded: plan_records.subscription_records[i].refunded,
                };
                data.push(k);
            }
            data
        }


        /// check_subscription : provides can use this function to check if user has authority to plan or not
        /// # arguments:
        /// * user : user address
        /// * provider_address
        /// * plan_index
        /// # return value: boolean that indicates the user has authority or not
        #[ink(message)]
        pub fn check_subscription(&self, user: Account, provider_address: Account, plan_index: u128) -> bool {
            if !self.users.contains_key(&user) {
                return false;
            }
            if !self.records.contains_key(&(user, provider_address)) {
                return false;
            }
            if !self.plan_index_to_record_index.contains_key(&(user, provider_address, plan_index)) {
                return false;
            }
            let last_index: u128 = *self.plan_index_to_record_index.get(&(user, provider_address, plan_index)).unwrap();
            let number: usize = last_index.try_into().unwrap();
            let record: &SubscriptionRecord = &self.records.get(&(user, provider_address)).unwrap().subscription_records[number];
            if record.plan_index != plan_index || record.refunded || record.plan.duration + record.subscription_time < self.env().block_timestamp()  {
                return false;
            }
            true
        }

        fn transfer(&self, addr: Account, amount: u128) -> Result<(), Error> {
            self.env()
                .transfer(addr, amount)
                .map_err(|err| {
                    match err {
                        ink_env::Error::BelowSubsistenceThreshold => {
                            Er::BelowSubsistenceThreshold
                        }
                        _ => Er::TransferFailed,
                    }
                })

        }

        /// add_entry : add a payment entry to provider payment management linked list
        /// # arguments:
        /// * provider_address
        /// * day_id : the calculation formula is : (finish date - contract start date) / 86400
        /// * amount : money amount
        fn add_entry(&mut self, provider_address: Account, day_id: u64, amount: u128) {
            let linked_list: &mut LinkedList = &mut self.providers.get_mut(&provider_address).unwrap().payment_manager;
            if linked_list.length == 0 {
                let object = PaymentAdmission { number: amount, next_day: day_id };
                linked_list.head = day_id;
                self.paymentAdmissions.insert((provider_address, day_id), object);
                linked_list.back = day_id;
                linked_list.length += 1;
            } else if day_id < linked_list.head {
                let object = PaymentAdmission { number: amount, next_day: linked_list.head };
                linked_list.head = day_id;
                self.paymentAdmissions.insert((provider_address, day_id), object);
                linked_list.length += 1;
            } else if day_id > linked_list.back {
                self.paymentAdmissions.get_mut(&(provider_address, linked_list.back)).unwrap().next_day = day_id;
                let object = PaymentAdmission { number: amount, next_day: day_id };
                linked_list.back = day_id;
                self.paymentAdmissions.insert((provider_address, day_id), object);
                linked_list.length += 1;
            } else {
                let mut cur_id: u64 = linked_list.head;
                loop {
                    if day_id == cur_id {
                        self.paymentAdmissions.get_mut(&(provider_address, day_id)).unwrap().number += amount;
                        break;
                    } else if day_id < self.paymentAdmissions.get(&(provider_address, cur_id)).unwrap().next_day {
                        let object = PaymentAdmission { number: amount, next_day: self.paymentAdmissions.get(&(provider_address, cur_id)).unwrap().next_day };
                        self.paymentAdmissions.get_mut(&(provider_address, cur_id)).unwrap().next_day = day_id;
                        self.paymentAdmissions.insert((provider_address, day_id), object);
                        linked_list.length += 1;
                        break;
                    }
                    cur_id = self.paymentAdmissions.get(&(provider_address, cur_id)).unwrap().next_day;
                    if cur_id == linked_list.back {
                        break;
                    }
                }
            }
        }

        /// remove_entry : when a user refunds this function removes its related entry
        /// # arguments:
        /// * provider_address
        /// * day_id : the calculation formula is : (finish date - contract start date) / 86400
        /// * amount
        fn remove_entry(&mut self, provider_address: Account, day_id: u64, amount: u128) {
            self.paymentAdmissions.get_mut(&(provider_address, day_id)).unwrap().number -= amount;
        }

        /// process : when providers withdraw this function calculates the amount of money
        /// # arguments:
        /// * provider_address
        /// * day_id : the calculation formula is : (finish date - contract start date) / 86400
        fn process(&mut self, provider_address: Account, day_id: u64) -> u128 {
            let linked_list: &mut LinkedList = &mut self.providers.get_mut(&provider_address).unwrap().payment_manager;
            let mut sum: u128 = 0;
            let mut cur_id: u64 = linked_list.head;
            while day_id >= cur_id {
                sum += self.paymentAdmissions.get(&(provider_address, cur_id)).unwrap().number;
                cur_id = self.paymentAdmissions.get(&(provider_address, cur_id)).unwrap().next_day;
                linked_list.length -= 1;
                if cur_id == linked_list.back {
                    break;
                }
            }
            linked_list.head = cur_id;
            sum
        }
    }



    impl LinkedList {
        pub fn new() -> Self {
            LinkedList::default()
        }
        pub fn default() -> Self {
            Self {
                back: 0,
                head: 0,
                length: 0,
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use ink_env::{call, test};
        use ink_lang as ink;
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

            let accounts =
                ink_env::test::default_accounts::<ink_env::DefaultEnvironment>()
                    .expect("Cannot get accounts");

            let callee = ink_env::test::get_current_contract_account_id::<ink_env::DefaultEnvironment>()
                .expect("Cannot get contract id");
            test::push_execution_context::<Environment>(
                accounts.alice,
                callee,
                100,
                100,
                test::CallData::new(call::Selector::new([0x00; 4])), // dummy
            );

            subsCrypt.provider_register(
                vec![60 * 60 * 24, 60 * 60 * 24 * 30],
                vec![2, 2],
                vec![10000, 50000],
                vec![50, 100],
                accounts.alice);
            assert_eq!(subsCrypt.providers.get(&accounts.alice).unwrap().plans.get(0).unwrap().duration, 60 * 60 * 24);
            assert_eq!(subsCrypt.providers.get(&accounts.alice).unwrap().plans.get(1).unwrap().active_session_limit, 2);
            assert_eq!(subsCrypt.providers.get(&accounts.alice).unwrap().plans.get(1).unwrap().duration, 60 * 60 * 24 * 30);
            assert_eq!(subsCrypt.providers.get(&accounts.alice).unwrap().plans.get(1).unwrap().price, 50000);
            assert_eq!(subsCrypt.providers.get(&accounts.alice).unwrap().money_address, accounts.alice);
        }

        #[ink::test]
        #[should_panic]
        fn provider_register_works2() {
            let mut subsCrypt = SubsCrypt::new();

            let accounts =
                ink_env::test::default_accounts::<ink_env::DefaultEnvironment>()
                    .expect("Cannot get accounts");

            let callee = ink_env::test::get_current_contract_account_id::<ink_env::DefaultEnvironment>()
                .expect("Cannot get contract id");
            test::push_execution_context::<Environment>(
                accounts.alice,
                callee,
                100,
                90,
                test::CallData::new(call::Selector::new([0x00; 4])), // dummy
            );

            subsCrypt.provider_register(
                vec![60 * 60 * 24, 60 * 60 * 24 * 30],
                vec![2, 2],
                vec![10000, 50000],
                vec![50, 100],
                accounts.alice);

        }
        #[ink::test]
        #[should_panic]
        fn provider_register_works3() {
            let mut subsCrypt = SubsCrypt::new();

            let accounts =
                ink_env::test::default_accounts::<ink_env::DefaultEnvironment>()
                    .expect("Cannot get accounts");

            let callee = ink_env::test::get_current_contract_account_id::<ink_env::DefaultEnvironment>()
                .expect("Cannot get contract id");
            test::push_execution_context::<Environment>(
                accounts.alice,
                callee,
                100,
                100,
                test::CallData::new(call::Selector::new([0x00; 4])), // dummy
            );

            subsCrypt.provider_register(
                vec![60 * 60 * 24],
                vec![2, 2],
                vec![10000, 50000],
                vec![50, 100],
                accounts.alice);

        }

        #[ink::test]
        fn edit_plan_works() {
            let mut subsCrypt = SubsCrypt::new();

            let accounts =
                ink_env::test::default_accounts::<ink_env::DefaultEnvironment>()
                    .expect("Cannot get accounts");

            let callee = ink_env::test::get_current_contract_account_id::<ink_env::DefaultEnvironment>()
                .expect("Cannot get contract id");
            test::push_execution_context::<Environment>(
                accounts.alice,
                callee,
                100,
                100,
                test::CallData::new(call::Selector::new([0x00; 4])), // dummy
            );

            subsCrypt.provider_register(
                vec![60 * 60 * 24, 60 * 60 * 24 * 30],
                vec![2, 2],
                vec![10000, 50000],
                vec![50, 100],
                accounts.alice);
            assert_eq!(subsCrypt.providers.get(&accounts.alice).unwrap().plans.get(0).unwrap().duration, 60 * 60 * 24);
            assert_eq!(subsCrypt.providers.get(&accounts.alice).unwrap().plans.get(1).unwrap().active_session_limit, 2);
            assert_eq!(subsCrypt.providers.get(&accounts.alice).unwrap().plans.get(1).unwrap().duration, 60 * 60 * 24 * 30);
            assert_eq!(subsCrypt.providers.get(&accounts.alice).unwrap().plans.get(1).unwrap().price, 50000);
            assert_eq!(subsCrypt.providers.get(&accounts.alice).unwrap().money_address, accounts.alice);

            subsCrypt.edit_plan(
                1,
                60 * 60 * 24 * 10,
                3,
                100000,
                500,
                false,
            );
            assert_eq!(subsCrypt.providers.get(&accounts.alice).unwrap().plans.get(1).unwrap().active_session_limit, 3);
            assert_eq!(subsCrypt.providers.get(&accounts.alice).unwrap().plans.get(1).unwrap().duration, 60 * 60 * 24 * 10);
            assert_eq!(subsCrypt.providers.get(&accounts.alice).unwrap().plans.get(1).unwrap().price, 100000);
            assert_eq!(subsCrypt.providers.get(&accounts.alice).unwrap().plans.get(1).unwrap().max_refund_percent_policy, 500);

            assert_eq!(subsCrypt.providers.get(&accounts.alice).unwrap().money_address, accounts.alice);
        }

        #[ink::test]
        #[should_panic]
        fn edit_plan_works2() {
            let mut subsCrypt = SubsCrypt::new();

            let accounts =
                ink_env::test::default_accounts::<ink_env::DefaultEnvironment>()
                    .expect("Cannot get accounts");

            let callee = ink_env::test::get_current_contract_account_id::<ink_env::DefaultEnvironment>()
                .expect("Cannot get contract id");
            test::push_execution_context::<Environment>(
                accounts.alice,
                callee,
                100,
                100,
                test::CallData::new(call::Selector::new([0x00; 4])), // dummy
            );

            subsCrypt.provider_register(
                vec![60 * 60 * 24, 60 * 60 * 24 * 30],
                vec![2, 2],
                vec![10000, 50000],
                vec![50, 100],
                accounts.alice);
            assert_eq!(subsCrypt.providers.get(&accounts.alice).unwrap().plans.get(0).unwrap().duration, 60 * 60 * 24);
            assert_eq!(subsCrypt.providers.get(&accounts.alice).unwrap().plans.get(1).unwrap().active_session_limit, 2);
            assert_eq!(subsCrypt.providers.get(&accounts.alice).unwrap().plans.get(1).unwrap().duration, 60 * 60 * 24 * 30);
            assert_eq!(subsCrypt.providers.get(&accounts.alice).unwrap().plans.get(1).unwrap().price, 50000);
            assert_eq!(subsCrypt.providers.get(&accounts.alice).unwrap().money_address, accounts.alice);

            subsCrypt.edit_plan(
                2,
                60 * 60 * 24 * 10,
                3,
                100000,
                500,
                false,
            );
            assert_eq!(subsCrypt.providers.get(&accounts.alice).unwrap().plans.get(1).unwrap().active_session_limit, 3);
            assert_eq!(subsCrypt.providers.get(&accounts.alice).unwrap().plans.get(1).unwrap().duration, 60 * 60 * 24 * 10);
            assert_eq!(subsCrypt.providers.get(&accounts.alice).unwrap().plans.get(1).unwrap().price, 100000);
            assert_eq!(subsCrypt.providers.get(&accounts.alice).unwrap().plans.get(1).unwrap().max_refund_percent_policy, 500);

            assert_eq!(subsCrypt.providers.get(&accounts.alice).unwrap().money_address, accounts.alice);
        }

        #[ink::test]
        fn add_plan_works() {
            let mut subsCrypt = SubsCrypt::new();

            let accounts =
                ink_env::test::default_accounts::<ink_env::DefaultEnvironment>()
                    .expect("Cannot get accounts");

            let callee = ink_env::test::get_current_contract_account_id::<ink_env::DefaultEnvironment>()
                .expect("Cannot get contract id");
            test::push_execution_context::<Environment>(
                accounts.alice,
                callee,
                100,
                100,
                test::CallData::new(call::Selector::new([0x00; 4])), // dummy
            );

            subsCrypt.provider_register(
                vec![60 * 60 * 24, 60 * 60 * 24 * 30],
                vec![2, 2],
                vec![10000, 50000],
                vec![50, 100],
                accounts.alice);
            assert_eq!(subsCrypt.providers.get(&accounts.alice).unwrap().plans.get(0).unwrap().duration, 60 * 60 * 24);
            assert_eq!(subsCrypt.providers.get(&accounts.alice).unwrap().plans.get(1).unwrap().active_session_limit, 2);
            assert_eq!(subsCrypt.providers.get(&accounts.alice).unwrap().plans.get(1).unwrap().duration, 60 * 60 * 24 * 30);
            assert_eq!(subsCrypt.providers.get(&accounts.alice).unwrap().plans.get(1).unwrap().price, 50000);
            assert_eq!(subsCrypt.providers.get(&accounts.alice).unwrap().money_address, accounts.alice);

            subsCrypt.add_plan(
                vec![60 * 60 * 24 * 10],
                vec![3],
                vec![100000],
                vec![500]
            );
            assert_eq!(subsCrypt.providers.get(&accounts.alice).unwrap().plans.get(2).unwrap().active_session_limit, 3);
            assert_eq!(subsCrypt.providers.get(&accounts.alice).unwrap().plans.get(2).unwrap().duration, 60 * 60 * 24 * 10);
            assert_eq!(subsCrypt.providers.get(&accounts.alice).unwrap().plans.get(2).unwrap().price, 100000);
            assert_eq!(subsCrypt.providers.get(&accounts.alice).unwrap().plans.get(2).unwrap().max_refund_percent_policy, 500);

            assert_eq!(subsCrypt.providers.get(&accounts.alice).unwrap().money_address, accounts.alice);
        }

        #[ink::test]
        #[should_panic]
        fn add_plan_works2() {
            let mut subsCrypt = SubsCrypt::new();

            let accounts =
                ink_env::test::default_accounts::<ink_env::DefaultEnvironment>()
                    .expect("Cannot get accounts");

            let callee = ink_env::test::get_current_contract_account_id::<ink_env::DefaultEnvironment>()
                .expect("Cannot get contract id");
            test::push_execution_context::<Environment>(
                accounts.alice,
                callee,
                100,
                100,
                test::CallData::new(call::Selector::new([0x00; 4])), // dummy
            );

            subsCrypt.provider_register(
                vec![60 * 60 * 24, 60 * 60 * 24 * 30],
                vec![2, 2],
                vec![10000, 50000],
                vec![50, 100],
                accounts.alice);
            assert_eq!(subsCrypt.providers.get(&accounts.alice).unwrap().plans.get(0).unwrap().duration, 60 * 60 * 24);
            assert_eq!(subsCrypt.providers.get(&accounts.alice).unwrap().plans.get(1).unwrap().active_session_limit, 2);
            assert_eq!(subsCrypt.providers.get(&accounts.alice).unwrap().plans.get(1).unwrap().duration, 60 * 60 * 24 * 30);
            assert_eq!(subsCrypt.providers.get(&accounts.alice).unwrap().plans.get(1).unwrap().price, 50000);
            assert_eq!(subsCrypt.providers.get(&accounts.alice).unwrap().money_address, accounts.alice);

            subsCrypt.add_plan(
                vec![60 * 60 * 24 * 10],
                vec![3,2],
                vec![100000],
                vec![500]
            );
            assert_eq!(subsCrypt.providers.get(&accounts.alice).unwrap().plans.get(2).unwrap().active_session_limit, 3);
            assert_eq!(subsCrypt.providers.get(&accounts.alice).unwrap().plans.get(2).unwrap().duration, 60 * 60 * 24 * 10);
            assert_eq!(subsCrypt.providers.get(&accounts.alice).unwrap().plans.get(2).unwrap().price, 100000);
            assert_eq!(subsCrypt.providers.get(&accounts.alice).unwrap().plans.get(2).unwrap().max_refund_percent_policy, 500);

            assert_eq!(subsCrypt.providers.get(&accounts.alice).unwrap().money_address, accounts.alice);
        }

        #[ink::test]
        fn change_disable_works() {
            let mut subsCrypt = SubsCrypt::new();

            let accounts =
                ink_env::test::default_accounts::<ink_env::DefaultEnvironment>()
                    .expect("Cannot get accounts");

            let callee = ink_env::test::get_current_contract_account_id::<ink_env::DefaultEnvironment>()
                .expect("Cannot get contract id");
            ink_env::test::set_account_balance::<ink_env::DefaultEnvironment>(
                callee, 100,
            )
                .expect("Cannot set account balance");
            test::push_execution_context::<Environment>(
                accounts.alice,
                callee,
                100,
                100,
                test::CallData::new(call::Selector::new([0x00; 4])), // dummy
            );

            subsCrypt.provider_register(
                vec![60 * 60 * 24, 60 * 60 * 24 * 30],
                vec![2, 2],
                vec![10000, 50000],
                vec![50, 100],
                accounts.alice);
            assert_eq!(subsCrypt.providers.get(&accounts.alice).unwrap().plans.get(0).unwrap().duration, 60 * 60 * 24);
            assert_eq!(subsCrypt.providers.get(&accounts.alice).unwrap().plans.get(1).unwrap().active_session_limit, 2);
            assert_eq!(subsCrypt.providers.get(&accounts.alice).unwrap().plans.get(1).unwrap().duration, 60 * 60 * 24 * 30);
            assert_eq!(subsCrypt.providers.get(&accounts.alice).unwrap().plans.get(1).unwrap().price, 50000);
            assert_eq!(subsCrypt.providers.get(&accounts.alice).unwrap().plans.get(1).unwrap().disabled, false);

            assert_eq!(subsCrypt.providers.get(&accounts.alice).unwrap().money_address, accounts.alice);
            subsCrypt.change_disable(1);
            assert_eq!(subsCrypt.providers.get(&accounts.alice).unwrap().plans.get(1).unwrap().disabled, true);

            subsCrypt.change_disable(1);
            assert_eq!(subsCrypt.providers.get(&accounts.alice).unwrap().plans.get(1).unwrap().disabled, false);
        }

        #[ink::test]
        fn subscribe_works() {
            let mut subsCrypt = SubsCrypt::new();

            let accounts =
                ink_env::test::default_accounts::<ink_env::DefaultEnvironment>()
                    .expect("Cannot get accounts");

            let callee = ink_env::test::get_current_contract_account_id::<ink_env::DefaultEnvironment>()
                .expect("Cannot get contract id");
            ink_env::test::set_account_balance::<ink_env::DefaultEnvironment>(
                callee, 50100,
            )
                .expect("Cannot set account balance");
            test::push_execution_context::<Environment>(
                accounts.alice,
                callee,
                100,
                100,
                test::CallData::new(call::Selector::new([0x00; 4])), // dummy
            );

            subsCrypt.provider_register(
                vec![60 * 60 * 24, 60 * 60 * 24 * 30],
                vec![2, 2],
                vec![10000, 50000],
                vec![50, 100],
                accounts.alice);
            assert_eq!(subsCrypt.providers.get(&accounts.alice).unwrap().plans.get(0).unwrap().duration, 60 * 60 * 24);
            assert_eq!(subsCrypt.providers.get(&accounts.alice).unwrap().plans.get(1).unwrap().active_session_limit, 2);
            assert_eq!(subsCrypt.providers.get(&accounts.alice).unwrap().plans.get(1).unwrap().duration, 60 * 60 * 24 * 30);
            assert_eq!(subsCrypt.providers.get(&accounts.alice).unwrap().plans.get(1).unwrap().price, 50000);
            assert_eq!(subsCrypt.providers.get(&accounts.alice).unwrap().plans.get(1).unwrap().disabled, false);

            assert_eq!(subsCrypt.providers.get(&accounts.alice).unwrap().money_address, accounts.alice);


            let callee = ink_env::test::get_current_contract_account_id::<ink_env::DefaultEnvironment>()
                .expect("Cannot get contract id");
            test::push_execution_context::<Environment>(
                accounts.bob,
                callee,
                50000,
                50000,
                test::CallData::new(call::Selector::new([0x00; 4])), // dummy
            );
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

            let accounts =
                ink_env::test::default_accounts::<ink_env::DefaultEnvironment>()
                    .expect("Cannot get accounts");

            let callee = ink_env::test::get_current_contract_account_id::<ink_env::DefaultEnvironment>()
                .expect("Cannot get contract id");
            ink_env::test::set_account_balance::<ink_env::DefaultEnvironment>(
                callee, 50100,
            )
                .expect("Cannot set account balance");
            test::push_execution_context::<Environment>(
                accounts.alice,
                callee,
                100,
                100,
                test::CallData::new(call::Selector::new([0x00; 4])), // dummy
            );

            subsCrypt.provider_register(
                vec![60 * 60 * 24, 60 * 60 * 24 * 30],
                vec![2, 2],
                vec![10000, 50000],
                vec![50, 100],
                accounts.alice);
            assert_eq!(subsCrypt.providers.get(&accounts.alice).unwrap().plans.get(0).unwrap().duration, 60 * 60 * 24);
            assert_eq!(subsCrypt.providers.get(&accounts.alice).unwrap().plans.get(1).unwrap().active_session_limit, 2);
            assert_eq!(subsCrypt.providers.get(&accounts.alice).unwrap().plans.get(1).unwrap().duration, 60 * 60 * 24 * 30);
            assert_eq!(subsCrypt.providers.get(&accounts.alice).unwrap().plans.get(1).unwrap().price, 50000);
            assert_eq!(subsCrypt.providers.get(&accounts.alice).unwrap().plans.get(1).unwrap().disabled, false);

            assert_eq!(subsCrypt.providers.get(&accounts.alice).unwrap().money_address, accounts.alice);


            let callee = ink_env::test::get_current_contract_account_id::<ink_env::DefaultEnvironment>()
                .expect("Cannot get contract id");
            test::push_execution_context::<Environment>(
                accounts.bob,
                callee,
                50000,
                49500,
                test::CallData::new(call::Selector::new([0x00; 4])), // dummy
            );
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
            ink_env::test::set_account_balance::<ink_env::DefaultEnvironment>(
                callee, 50100,
            )
                .expect("Cannot set account balance");
            test::push_execution_context::<Environment>(
                accounts.alice,
                callee,
                100,
                100,
                test::CallData::new(call::Selector::new([0x00; 4])), // dummy
            );

            subsCrypt.provider_register(
                vec![60 * 60 * 24, 60 * 60 * 24 * 30],
                vec![2, 2],
                vec![10000, 50000],
                vec![50, 100],
                accounts.alice);
            assert_eq!(subsCrypt.providers.get(&accounts.alice).unwrap().plans.get(0).unwrap().duration, 60 * 60 * 24);
            assert_eq!(subsCrypt.providers.get(&accounts.alice).unwrap().plans.get(1).unwrap().active_session_limit, 2);
            assert_eq!(subsCrypt.providers.get(&accounts.alice).unwrap().plans.get(1).unwrap().duration, 60 * 60 * 24 * 30);
            assert_eq!(subsCrypt.providers.get(&accounts.alice).unwrap().plans.get(1).unwrap().price, 50000);
            assert_eq!(subsCrypt.providers.get(&accounts.alice).unwrap().plans.get(1).unwrap().disabled, false);

            assert_eq!(subsCrypt.providers.get(&accounts.alice).unwrap().money_address, accounts.alice);


            let callee = ink_env::test::get_current_contract_account_id::<ink_env::DefaultEnvironment>()
                .expect("Cannot get contract id");
            test::push_execution_context::<Environment>(
                accounts.bob,
                callee,
                50000,
                50000,
                test::CallData::new(call::Selector::new([0x00; 4])), // dummy
            );
            subsCrypt.subscribe(
                accounts.alice,
                1,
                [0; 32],
                "nothing important".to_string(),
            );
            assert_eq!(subsCrypt.users.get(&accounts.bob).unwrap().list_of_providers.get(0).unwrap(), &accounts.alice);
            test::push_execution_context::<Environment>(
                accounts.alice,
                callee,
                0,
                0,
                test::CallData::new(call::Selector::new([0x00; 4])), // dummy
            );
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
            ink_env::test::set_account_balance::<ink_env::DefaultEnvironment>(
                callee, 50100,
            )
                .expect("Cannot set account balance");
            test::push_execution_context::<Environment>(
                accounts.alice,
                callee,
                100,
                100,
                test::CallData::new(call::Selector::new([0x00; 4])), // dummy
            );

            subsCrypt.provider_register(
                vec![60 * 60 * 24, 60 * 60 * 24 * 30],
                vec![2, 2],
                vec![10000, 50000],
                vec![50, 100],
                accounts.alice);
            assert_eq!(subsCrypt.providers.get(&accounts.alice).unwrap().plans.get(0).unwrap().duration, 60 * 60 * 24);
            assert_eq!(subsCrypt.providers.get(&accounts.alice).unwrap().plans.get(1).unwrap().active_session_limit, 2);
            assert_eq!(subsCrypt.providers.get(&accounts.alice).unwrap().plans.get(1).unwrap().duration, 60 * 60 * 24 * 30);
            assert_eq!(subsCrypt.providers.get(&accounts.alice).unwrap().plans.get(1).unwrap().price, 50000);
            assert_eq!(subsCrypt.providers.get(&accounts.alice).unwrap().plans.get(1).unwrap().disabled, false);

            assert_eq!(subsCrypt.providers.get(&accounts.alice).unwrap().money_address, accounts.alice);


            let callee = ink_env::test::get_current_contract_account_id::<ink_env::DefaultEnvironment>()
                .expect("Cannot get contract id");
            test::push_execution_context::<Environment>(
                accounts.bob,
                callee,
                50000,
                50000,
                test::CallData::new(call::Selector::new([0x00; 4])), // dummy
            );
            subsCrypt.subscribe(
                accounts.alice,
                1,
                [0; 32],
                "nothing important".to_string(),
            );
            assert_eq!(subsCrypt.users.get(&accounts.bob).unwrap().list_of_providers.get(0).unwrap(), &accounts.alice);
            test::push_execution_context::<Environment>(
                accounts.eve,
                callee,
                0,
                0,
                test::CallData::new(call::Selector::new([0x00; 4])), // dummy
            );
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
            ink_env::test::set_account_balance::<ink_env::DefaultEnvironment>(
                callee, 50100,
            )
                .expect("Cannot set account balance");
            let mut data = ink_env::test::CallData::new(ink_env::call::Selector::new([
                0xCA, 0xFE, 0xBA, 0xBE,
            ]));
            data.push_arg(&accounts.alice);
            test::push_execution_context::<Environment>(
                accounts.alice,
                callee,
                100000,
                100,
                data,
            );

            subsCrypt.provider_register(
                vec![60 * 60 * 24, 60 * 5],
                vec![2, 2],
                vec![10000, 50000],
                vec![50, 100],
                accounts.alice);
            assert_eq!(subsCrypt.providers.get(&accounts.alice).unwrap().plans.get(0).unwrap().duration, 60 * 60 * 24);
            assert_eq!(subsCrypt.providers.get(&accounts.alice).unwrap().plans.get(1).unwrap().active_session_limit, 2);
            assert_eq!(subsCrypt.providers.get(&accounts.alice).unwrap().plans.get(1).unwrap().duration, 60 * 5);
            assert_eq!(subsCrypt.providers.get(&accounts.alice).unwrap().plans.get(1).unwrap().price, 50000);
            assert_eq!(subsCrypt.providers.get(&accounts.alice).unwrap().plans.get(1).unwrap().disabled, false);

            assert_eq!(subsCrypt.providers.get(&accounts.alice).unwrap().money_address, accounts.alice);


            let callee = ink_env::test::get_current_contract_account_id::<ink_env::DefaultEnvironment>()
                .expect("Cannot get contract id");
            test::push_execution_context::<Environment>(
                accounts.bob,
                callee,
                50000,
                50000,
                test::CallData::new(call::Selector::new([0x00; 4])), // dummy
            );

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

            let accounts =
                ink_env::test::default_accounts::<ink_env::DefaultEnvironment>()
                    .expect("Cannot get accounts");
            let callee = ink_env::test::get_current_contract_account_id::<ink_env::DefaultEnvironment>()
                .expect("Cannot get contract id");
            ink_env::test::set_account_balance::<ink_env::DefaultEnvironment>(
                callee, 50100,
            )
                .expect("Cannot set account balance");
            let mut data = ink_env::test::CallData::new(ink_env::call::Selector::new([
                0xCA, 0xFE, 0xBA, 0xBE,
            ]));
            data.push_arg(&accounts.alice);
            test::push_execution_context::<Environment>(
                accounts.alice,
                callee,
                100000,
                100,
                data,
            );

            subsCrypt.provider_register(
                vec![60 * 60 * 24, 60 * 5],
                vec![2, 2],
                vec![10000, 50000],
                vec![50, 100],
                accounts.alice);
            assert_eq!(subsCrypt.providers.get(&accounts.alice).unwrap().plans.get(0).unwrap().duration, 60 * 60 * 24);
            assert_eq!(subsCrypt.providers.get(&accounts.alice).unwrap().plans.get(1).unwrap().active_session_limit, 2);
            assert_eq!(subsCrypt.providers.get(&accounts.alice).unwrap().plans.get(1).unwrap().duration, 60 * 5);
            assert_eq!(subsCrypt.providers.get(&accounts.alice).unwrap().plans.get(1).unwrap().price, 50000);
            assert_eq!(subsCrypt.providers.get(&accounts.alice).unwrap().plans.get(1).unwrap().disabled, false);

            assert_eq!(subsCrypt.providers.get(&accounts.alice).unwrap().money_address, accounts.alice);


            let callee = ink_env::test::get_current_contract_account_id::<ink_env::DefaultEnvironment>()
                .expect("Cannot get contract id");
            test::push_execution_context::<Environment>(
                accounts.bob,
                callee,
                50000,
                50000,
                test::CallData::new(call::Selector::new([0x00; 4])), // dummy
            );

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
            assert_eq!(subsCrypt.records.get(&(accounts.bob, accounts.alice)).unwrap().subscription_records.get(0).unwrap().refunded, true);
        }

        #[ink::test]
        fn check_subscription_works() {
            let mut subsCrypt = SubsCrypt::new();

            let accounts =
                ink_env::test::default_accounts::<ink_env::DefaultEnvironment>()
                    .expect("Cannot get accounts");
            let callee = ink_env::test::get_current_contract_account_id::<ink_env::DefaultEnvironment>()
                .expect("Cannot get contract id");
            ink_env::test::set_account_balance::<ink_env::DefaultEnvironment>(
                callee, 50100,
            )
                .expect("Cannot set account balance");
            let mut data = ink_env::test::CallData::new(ink_env::call::Selector::new([
                0xCA, 0xFE, 0xBA, 0xBE,
            ]));
            data.push_arg(&accounts.alice);
            test::push_execution_context::<Environment>(
                accounts.alice,
                callee,
                100000,
                100,
                data,
            );

            subsCrypt.provider_register(
                vec![60 * 60 * 24, 60 *600000* 5],
                vec![2, 2],
                vec![10000, 50000],
                vec![50, 100],
                accounts.alice);
            assert_eq!(subsCrypt.providers.get(&accounts.alice).unwrap().plans.get(0).unwrap().duration, 60 * 60 * 24);
            assert_eq!(subsCrypt.providers.get(&accounts.alice).unwrap().plans.get(1).unwrap().active_session_limit, 2);
            assert_eq!(subsCrypt.providers.get(&accounts.alice).unwrap().plans.get(1).unwrap().duration,  60 *600000* 5);
            assert_eq!(subsCrypt.providers.get(&accounts.alice).unwrap().plans.get(1).unwrap().price, 50000);
            assert_eq!(subsCrypt.providers.get(&accounts.alice).unwrap().plans.get(1).unwrap().disabled, false);

            assert_eq!(subsCrypt.providers.get(&accounts.alice).unwrap().money_address, accounts.alice);


            let callee = ink_env::test::get_current_contract_account_id::<ink_env::DefaultEnvironment>()
                .expect("Cannot get contract id");
            test::push_execution_context::<Environment>(
                accounts.bob,
                callee,
                50000,
                50000,
                test::CallData::new(call::Selector::new([0x00; 4])), // dummy
            );

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

            let accounts =
                ink_env::test::default_accounts::<ink_env::DefaultEnvironment>()
                    .expect("Cannot get accounts");
            let callee = ink_env::test::get_current_contract_account_id::<ink_env::DefaultEnvironment>()
                .expect("Cannot get contract id");
            ink_env::test::set_account_balance::<ink_env::DefaultEnvironment>(
                callee, 50100,
            )
                .expect("Cannot set account balance");
            let mut data = ink_env::test::CallData::new(ink_env::call::Selector::new([
                0xCA, 0xFE, 0xBA, 0xBE,
            ]));
            data.push_arg(&accounts.alice);
            test::push_execution_context::<Environment>(
                accounts.alice,
                callee,
                100000,
                100,
                data,
            );

            subsCrypt.provider_register(
                vec![60 * 60 * 24, 60 *600000* 5],
                vec![2, 2],
                vec![10000, 50000],
                vec![50, 100],
                accounts.alice);
            assert_eq!(subsCrypt.providers.get(&accounts.alice).unwrap().plans.get(0).unwrap().duration, 60 * 60 * 24);
            assert_eq!(subsCrypt.providers.get(&accounts.alice).unwrap().plans.get(1).unwrap().active_session_limit, 2);
            assert_eq!(subsCrypt.providers.get(&accounts.alice).unwrap().plans.get(1).unwrap().duration,  60 *600000* 5);
            assert_eq!(subsCrypt.providers.get(&accounts.alice).unwrap().plans.get(1).unwrap().price, 50000);
            assert_eq!(subsCrypt.providers.get(&accounts.alice).unwrap().plans.get(1).unwrap().disabled, false);

            assert_eq!(subsCrypt.providers.get(&accounts.alice).unwrap().money_address, accounts.alice);


            let callee = ink_env::test::get_current_contract_account_id::<ink_env::DefaultEnvironment>()
                .expect("Cannot get contract id");
            test::push_execution_context::<Environment>(
                accounts.bob,
                callee,
                50000,
                50000,
                test::CallData::new(call::Selector::new([0x00; 4])), // dummy
            );

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
            assert_eq!(s[0].plan.duration,60 * 600000 * 5);
        }

        #[ink::test]
        fn retrieve_whole_data_with_wallet_works() {
            let mut subsCrypt = SubsCrypt::new();

            let accounts =
                ink_env::test::default_accounts::<ink_env::DefaultEnvironment>()
                    .expect("Cannot get accounts");
            let callee = ink_env::test::get_current_contract_account_id::<ink_env::DefaultEnvironment>()
                .expect("Cannot get contract id");
            ink_env::test::set_account_balance::<ink_env::DefaultEnvironment>(
                callee, 50100,
            )
                .expect("Cannot set account balance");
            let mut data = ink_env::test::CallData::new(ink_env::call::Selector::new([
                0xCA, 0xFE, 0xBA, 0xBE,
            ]));
            data.push_arg(&accounts.alice);
            test::push_execution_context::<Environment>(
                accounts.alice,
                callee,
                100000,
                100,
                data,
            );

            subsCrypt.provider_register(
                vec![60 * 60 * 24, 60 *600000* 5],
                vec![2, 2],
                vec![10000, 50000],
                vec![50, 100],
                accounts.alice);
            assert_eq!(subsCrypt.providers.get(&accounts.alice).unwrap().plans.get(0).unwrap().duration, 60 * 60 * 24);
            assert_eq!(subsCrypt.providers.get(&accounts.alice).unwrap().plans.get(1).unwrap().active_session_limit, 2);
            assert_eq!(subsCrypt.providers.get(&accounts.alice).unwrap().plans.get(1).unwrap().duration,  60 *600000* 5);
            assert_eq!(subsCrypt.providers.get(&accounts.alice).unwrap().plans.get(1).unwrap().price, 50000);
            assert_eq!(subsCrypt.providers.get(&accounts.alice).unwrap().plans.get(1).unwrap().disabled, false);

            assert_eq!(subsCrypt.providers.get(&accounts.alice).unwrap().money_address, accounts.alice);


            let callee = ink_env::test::get_current_contract_account_id::<ink_env::DefaultEnvironment>()
                .expect("Cannot get contract id");
            test::push_execution_context::<Environment>(
                accounts.bob,
                callee,
                50000,
                50000,
                test::CallData::new(call::Selector::new([0x00; 4])), // dummy
            );

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
            assert_eq!(s[0].plan.duration,60 * 600000 * 5);
        }

        #[ink::test]
        fn retrieve_data_with_password_works() {
            let mut subsCrypt = SubsCrypt::new();

            let accounts =
                ink_env::test::default_accounts::<ink_env::DefaultEnvironment>()
                    .expect("Cannot get accounts");
            let callee = ink_env::test::get_current_contract_account_id::<ink_env::DefaultEnvironment>()
                .expect("Cannot get contract id");
            ink_env::test::set_account_balance::<ink_env::DefaultEnvironment>(
                callee, 50100,
            )
                .expect("Cannot set account balance");
            let mut data = ink_env::test::CallData::new(ink_env::call::Selector::new([
                0xCA, 0xFE, 0xBA, 0xBE,
            ]));
            data.push_arg(&accounts.alice);
            test::push_execution_context::<Environment>(
                accounts.alice,
                callee,
                100000,
                100,
                data,
            );

            subsCrypt.provider_register(
                vec![60 * 60 * 24, 60 *600000* 5],
                vec![2, 2],
                vec![10000, 50000],
                vec![50, 100],
                accounts.alice);
            assert_eq!(subsCrypt.providers.get(&accounts.alice).unwrap().plans.get(0).unwrap().duration, 60 * 60 * 24);
            assert_eq!(subsCrypt.providers.get(&accounts.alice).unwrap().plans.get(1).unwrap().active_session_limit, 2);
            assert_eq!(subsCrypt.providers.get(&accounts.alice).unwrap().plans.get(1).unwrap().duration,  60 *600000* 5);
            assert_eq!(subsCrypt.providers.get(&accounts.alice).unwrap().plans.get(1).unwrap().price, 50000);
            assert_eq!(subsCrypt.providers.get(&accounts.alice).unwrap().plans.get(1).unwrap().disabled, false);

            assert_eq!(subsCrypt.providers.get(&accounts.alice).unwrap().money_address, accounts.alice);


            let callee = ink_env::test::get_current_contract_account_id::<ink_env::DefaultEnvironment>()
                .expect("Cannot get contract id");
            test::push_execution_context::<Environment>(
                accounts.bob,
                callee,
                50000,
                50000,
                test::CallData::new(call::Selector::new([0x00; 4])), // dummy
            );
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
            assert_eq!(s[0].plan.duration,60 * 600000 * 5);

        }

        #[ink::test]
        fn retrieve_whole_data_with_password_works() {
            let mut subsCrypt = SubsCrypt::new();

            let accounts =
                ink_env::test::default_accounts::<ink_env::DefaultEnvironment>()
                    .expect("Cannot get accounts");
            let callee = ink_env::test::get_current_contract_account_id::<ink_env::DefaultEnvironment>()
                .expect("Cannot get contract id");
            ink_env::test::set_account_balance::<ink_env::DefaultEnvironment>(
                callee, 50100,
            )
                .expect("Cannot set account balance");
            let mut data = ink_env::test::CallData::new(ink_env::call::Selector::new([
                0xCA, 0xFE, 0xBA, 0xBE,
            ]));
            data.push_arg(&accounts.alice);
            test::push_execution_context::<Environment>(
                accounts.alice,
                callee,
                100000,
                100,
                data,
            );

            subsCrypt.provider_register(
                vec![60 * 60 * 24, 60 *600000* 5],
                vec![2, 2],
                vec![10000, 50000],
                vec![50, 100],
                accounts.alice);
            assert_eq!(subsCrypt.providers.get(&accounts.alice).unwrap().plans.get(0).unwrap().duration, 60 * 60 * 24);
            assert_eq!(subsCrypt.providers.get(&accounts.alice).unwrap().plans.get(1).unwrap().active_session_limit, 2);
            assert_eq!(subsCrypt.providers.get(&accounts.alice).unwrap().plans.get(1).unwrap().duration,  60 *600000* 5);
            assert_eq!(subsCrypt.providers.get(&accounts.alice).unwrap().plans.get(1).unwrap().price, 50000);
            assert_eq!(subsCrypt.providers.get(&accounts.alice).unwrap().plans.get(1).unwrap().disabled, false);

            assert_eq!(subsCrypt.providers.get(&accounts.alice).unwrap().money_address, accounts.alice);

            test::push_execution_context::<Environment>(
                accounts.bob,
                callee,
                50000,
                50000,
                test::CallData::new(call::Selector::new([0x00; 4])), // dummy
            );
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
            assert_eq!(s[0].plan.duration,60 * 600000 * 5);

        }
        #[ink::test]
        fn check_auth_works() {
            let mut subsCrypt = SubsCrypt::new();

            let accounts =
                ink_env::test::default_accounts::<ink_env::DefaultEnvironment>()
                    .expect("Cannot get accounts");
            let callee = ink_env::test::get_current_contract_account_id::<ink_env::DefaultEnvironment>()
                .expect("Cannot get contract id");
            ink_env::test::set_account_balance::<ink_env::DefaultEnvironment>(
                callee, 50100,
            )
                .expect("Cannot set account balance");
            let mut data = ink_env::test::CallData::new(ink_env::call::Selector::new([
                0xCA, 0xFE, 0xBA, 0xBE,
            ]));
            data.push_arg(&accounts.alice);
            test::push_execution_context::<Environment>(
                accounts.alice,
                callee,
                100000,
                100,
                data,
            );

            subsCrypt.provider_register(
                vec![60 * 60 * 24, 60 *600000* 5],
                vec![2, 2],
                vec![10000, 50000],
                vec![50, 100],
                accounts.alice);
            assert_eq!(subsCrypt.providers.get(&accounts.alice).unwrap().plans.get(0).unwrap().duration, 60 * 60 * 24);
            assert_eq!(subsCrypt.providers.get(&accounts.alice).unwrap().plans.get(1).unwrap().active_session_limit, 2);
            assert_eq!(subsCrypt.providers.get(&accounts.alice).unwrap().plans.get(1).unwrap().duration,  60 *600000* 5);
            assert_eq!(subsCrypt.providers.get(&accounts.alice).unwrap().plans.get(1).unwrap().price, 50000);
            assert_eq!(subsCrypt.providers.get(&accounts.alice).unwrap().plans.get(1).unwrap().disabled, false);

            assert_eq!(subsCrypt.providers.get(&accounts.alice).unwrap().money_address, accounts.alice);


            let callee = ink_env::test::get_current_contract_account_id::<ink_env::DefaultEnvironment>()
                .expect("Cannot get contract id");
            test::push_execution_context::<Environment>(
                accounts.bob,
                callee,
                50000,
                50000,
                test::CallData::new(call::Selector::new([0x00; 4])), // dummy
            );
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
            ink_env::test::set_account_balance::<ink_env::DefaultEnvironment>(
                callee, 90100,
            )
                .expect("Cannot set account balance");
            test::push_execution_context::<Environment>(
                accounts.alice,
                callee,
                100,
                100,
                test::CallData::new(call::Selector::new([0x00; 4])), // dummy
            );

            subsCrypt.provider_register(
                vec![60 * 60 * 24, 60 * 60 * 24 * 30,60 * 60 * 24 * 300,60 * 60 * 24 * 31],
                vec![2, 2,2,2],
                vec![10000, 50000,10000,10000],
                vec![50, 100,200,100],
                accounts.alice);
            assert_eq!(subsCrypt.providers.get(&accounts.alice).unwrap().plans.get(0).unwrap().duration, 60 * 60 * 24);
            assert_eq!(subsCrypt.providers.get(&accounts.alice).unwrap().plans.get(1).unwrap().active_session_limit, 2);
            assert_eq!(subsCrypt.providers.get(&accounts.alice).unwrap().plans.get(1).unwrap().duration, 60 * 60 * 24 * 30);
            assert_eq!(subsCrypt.providers.get(&accounts.alice).unwrap().plans.get(1).unwrap().price, 50000);
            assert_eq!(subsCrypt.providers.get(&accounts.alice).unwrap().plans.get(1).unwrap().disabled, false);

            assert_eq!(subsCrypt.providers.get(&accounts.alice).unwrap().money_address, accounts.alice);


            let callee = ink_env::test::get_current_contract_account_id::<ink_env::DefaultEnvironment>()
                .expect("Cannot get contract id");
            test::push_execution_context::<Environment>(
                accounts.bob,
                callee,
                50000,
                50000,
                test::CallData::new(call::Selector::new([0x00; 4])), // dummy
            );
            subsCrypt.subscribe(
                accounts.alice,
                1,
                [0; 32],
                "nothing important".to_string(),
            );
            test::push_execution_context::<Environment>(
                accounts.bob,
                callee,
                10000,
                10000,
                test::CallData::new(call::Selector::new([0x00; 4])), // dummy
            );
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
            test::push_execution_context::<Environment>(
                accounts.eve,
                callee,
                10000,
                10000,
                test::CallData::new(call::Selector::new([0x00; 4])), // dummy
            );
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
            test::push_execution_context::<Environment>(
                accounts.alice,
                callee,
                100000,
                0,
                test::CallData::new(call::Selector::new([0x00; 4])), // dummy
            );
            subsCrypt.process(accounts.alice,1000);
        }

    }
}
