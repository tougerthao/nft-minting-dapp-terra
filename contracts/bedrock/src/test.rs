#[cfg(test)]
mod tests {
    use crate::contract::{execute, instantiate, query};
    use crate::error::ContractError;

    use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, RoyaltiesInfoResponse};
    use crate::state::{Extension, CONFIG, ROYALTIES_INFO};
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coin, from_binary, Empty, Response, StdError};
    use cw721::{Cw721Query, NftInfoResponse};
    use cw721_base::state::Cw721Contract;
    use cw721_base::MintMsg;

    const CREATOR: &str = "creator";
    const PUBLIC: &str = "public";
    const OWNER: &str = "owner";

    // Mint tests

    #[test]
    fn init_mint() {
        let mut deps = mock_dependencies(&[]);

        let info = mock_info(CREATOR, &[]);
        let init_msg = InstantiateMsg {
            name: "SpaceShips".to_string(),
            symbol: "SPACE".to_string(),
            max_token_count: 1,
            treasury_account: CREATOR.to_string(),
            is_mint_public: true,
            start_time: None,
            end_time: None,
            price: None,
            royalty_payments: false,
            royalty_percentage: None,
            royalty_payment_address: None,
        };

        instantiate(deps.as_mut(), mock_env(), info.clone(), init_msg).unwrap();

        let token_id = "Enterprise";
        let mint_info = mock_info(OWNER, &[]);
        let mint_msg = MintMsg {
            token_id: token_id.to_string(),
            owner: OWNER.to_string(),
            token_uri: Some("uri".to_string()),
            extension: None,
        };
        let exec_msg = ExecuteMsg::Mint(mint_msg.clone());
        execute(deps.as_mut(), mock_env(), mint_info, exec_msg).unwrap();

        let query_msg: QueryMsg = QueryMsg::NftInfo {
            token_id: (&token_id).to_string(),
        };

        let res: NftInfoResponse<Extension> =
            from_binary(&query(deps.as_ref(), mock_env(), query_msg).unwrap()).unwrap();
        assert_eq!(res.token_uri, mint_msg.token_uri);
    }

    #[test]
    fn mint_limit() {
        let mut deps = mock_dependencies(&[]);
        let contract = Cw721Contract::<Extension, Empty>::default();

        let info = mock_info(CREATOR, &[]);
        let init_msg = InstantiateMsg {
            name: "SpaceShips".to_string(),
            symbol: "SPACE".to_string(),
            max_token_count: 1,
            treasury_account: CREATOR.to_string(),
            is_mint_public: true,
            start_time: None,
            end_time: None,
            price: None,
            royalty_payments: false,
            royalty_percentage: None,
            royalty_payment_address: None,
        };

        instantiate(deps.as_mut(), mock_env(), info.clone(), init_msg).unwrap();

        let token_id = "Enterprise";
        let mint_msg = MintMsg {
            token_id: token_id.to_string(),
            owner: CREATOR.to_string(),
            token_uri: None,
            extension: None,
        };

        let exec_msg = ExecuteMsg::Mint(mint_msg.clone());
        execute(deps.as_mut(), mock_env(), info.clone(), exec_msg).unwrap();

        let token_count = contract.token_count(&deps.storage).unwrap();
        assert_eq!(token_count, 1);

        // Should not allow mints above supply
        let exec_msg = ExecuteMsg::Mint(mint_msg.clone());
        let res = execute(deps.as_mut(), mock_env(), info.clone(), exec_msg);
        assert_eq!(ContractError::MaxTokenSupply {}, res.unwrap_err());
    }

    #[test]
    fn mint_not_public() {
        let mut deps = mock_dependencies(&[]);

        let info = mock_info(CREATOR, &[]);
        let init_msg = InstantiateMsg {
            name: "SpaceShips".to_string(),
            symbol: "SPACE".to_string(),
            max_token_count: 1,
            treasury_account: CREATOR.to_string(),
            is_mint_public: false,
            start_time: None,
            end_time: None,
            price: None,
            royalty_payments: false,
            royalty_percentage: None,
            royalty_payment_address: None,
        };

        instantiate(deps.as_mut(), mock_env(), info.clone(), init_msg).unwrap();

        let token_id = "Enterprise";
        let mint_info = mock_info(OWNER, &[]);
        let mint_msg = MintMsg {
            token_id: token_id.to_string(),
            owner: OWNER.to_string(),
            token_uri: None,
            extension: None,
        };

        let exec_msg = ExecuteMsg::Mint(mint_msg.clone());
        let res = execute(deps.as_mut(), mock_env(), mint_info.clone(), exec_msg);

        assert_eq!(ContractError::Unauthorized {}, res.unwrap_err());
    }

    #[test]
    fn mint_funds() {
        let mut deps = mock_dependencies(&[]);

        let info = mock_info(CREATOR, &[]);
        let init_msg = InstantiateMsg {
            name: "SpaceShips".to_string(),
            symbol: "SPACE".to_string(),
            max_token_count: 2,
            treasury_account: CREATOR.to_string(),
            is_mint_public: true,
            start_time: None,
            end_time: None,
            price: Some(coin(1_000_000, "uluna")),
            royalty_payments: false,
            royalty_percentage: None,
            royalty_payment_address: None,
        };

        instantiate(deps.as_mut(), mock_env(), info.clone(), init_msg).unwrap();

        let token_ids = ["Enterprise", "Falcon"];
        let coins = [coin(1_000_000, "uluna")];
        let success_info = mock_info(OWNER, &coins);
        let failure_info = mock_info(OWNER, &[]);

        // sufficient coins
        let mint_msg1 = MintMsg {
            token_id: token_ids[0].to_string(),
            owner: OWNER.to_string(),
            token_uri: None,
            extension: None,
        };
        let exec_msg = ExecuteMsg::Mint(mint_msg1);
        let res = execute(deps.as_mut(), mock_env(), success_info, exec_msg.clone());
        assert!(res.is_ok());

        // insufficient coins
        let mint_msg2 = MintMsg {
            token_id: token_ids[1].to_string(),
            owner: OWNER.to_string(),
            token_uri: None,
            extension: None,
        };
        let exec_msg = ExecuteMsg::Mint(mint_msg2);
        let res = execute(
            deps.as_mut(),
            mock_env(),
            failure_info.clone(),
            exec_msg.clone(),
        );
        let err = ContractError::Std(StdError::generic_err("incorrect amount of funds sent"));
        assert_eq!(err, res.unwrap_err());
    }

    #[test]
    fn mint_times() {
        let mut deps = mock_dependencies(&[]);

        let info = mock_info(CREATOR, &[]);
        let env = mock_env();
        let init_msg = InstantiateMsg {
            name: "SpaceShips".to_string(),
            symbol: "SPACE".to_string(),
            max_token_count: 1,
            treasury_account: CREATOR.to_string(),
            is_mint_public: true,
            start_time: Some(env.block.time.minus_seconds(1000).seconds()),
            end_time: Some(env.block.time.plus_seconds(1000).seconds()),
            price: None,
            royalty_payments: false,
            royalty_percentage: None,
            royalty_payment_address: None,
        };

        instantiate(deps.as_mut(), env.clone(), info.clone(), init_msg).unwrap();

        let token_id = "Enterprise";
        let mint_msg = MintMsg {
            token_id: token_id.to_string(),
            owner: OWNER.to_string(),
            token_uri: None,
            extension: None,
        };
        let exec_msg = ExecuteMsg::Mint(mint_msg);
        let mint_info = mock_info(OWNER, &[]);

        // Minting too early (fail)
        let mut early_env = mock_env();
        early_env.block.time = env.block.time.minus_seconds(1010); // 10 seconds before start_time
        let res = execute(
            deps.as_mut(),
            early_env,
            mint_info.clone(),
            exec_msg.clone(),
        );
        assert_eq!(ContractError::Unauthorized {}, res.unwrap_err());

        // Minting too late (fail)
        let mut late_env = mock_env();
        late_env.block.time = env.block.time.plus_seconds(1010); // 10 seconds after end_time
        let res = execute(deps.as_mut(), late_env, mint_info.clone(), exec_msg.clone());
        assert_eq!(ContractError::Unauthorized {}, res.unwrap_err());

        // Minting inbetween (success)
        let mut valid_env = mock_env();
        valid_env.block.time = env.block.time;
        let res = execute(deps.as_mut(), valid_env, mint_info, exec_msg);
        assert!(res.is_ok());
    }

    // Transfer tests
    #[test]
    fn transfer_nft() {
        let mut deps = mock_dependencies(&[]);

        let info = mock_info(CREATOR, &[]);
        let init_msg = InstantiateMsg {
            name: "SpaceShips".to_string(),
            symbol: "SPACE".to_string(),
            max_token_count: 2,
            treasury_account: CREATOR.to_string(),
            is_mint_public: true,
            start_time: None,
            end_time: None,
            price: None,
            royalty_payments: false,
            royalty_percentage: None,
            royalty_payment_address: None,
        };

        instantiate(deps.as_mut(), mock_env(), info.clone(), init_msg).unwrap();

        let token_id = "Enterprise";
        let mint_info = mock_info(OWNER, &[]);

        let mint_msg = MintMsg {
            token_id: token_id.to_string(),
            owner: OWNER.to_string(),
            token_uri: None,
            extension: None,
        };
        let exec_msg = ExecuteMsg::Mint(mint_msg);
        execute(deps.as_mut(), mock_env(), mint_info, exec_msg.clone()).unwrap();

        let transfer_msg = ExecuteMsg::TransferNft {
            recipient: PUBLIC.to_string(),
            token_id: token_id.to_string(),
        };

        // random can not transfer
        let random_info = mock_info(PUBLIC, &[]);
        let err =
            execute(deps.as_mut(), mock_env(), random_info, transfer_msg.clone()).unwrap_err();
        assert_eq!(err, ContractError::Unauthorized {});

        // but owner can transfer
        let owner_info = mock_info(OWNER, &[]);
        let res = execute(deps.as_mut(), mock_env(), owner_info, transfer_msg).unwrap();

        // and make sure this is the request sent by the contract
        assert_eq!(
            res,
            Response::new()
                .add_attribute("action", "transfer_nft")
                .add_attribute("sender", OWNER)
                .add_attribute("recipient", PUBLIC)
                .add_attribute("token_id", token_id)
        );
    }

    // Withdraw tests

    #[test]
    fn withdraw() {
        let mut deps = mock_dependencies(&[]);

        let creator_info = mock_info(CREATOR, &[]);
        let init_msg = InstantiateMsg {
            name: "SpaceShips".to_string(),
            symbol: "SPACE".to_string(),
            max_token_count: 1,
            treasury_account: CREATOR.to_string(),
            is_mint_public: true,
            start_time: None,
            end_time: None,
            price: Some(coin(1_000_000, "uluna")),
            royalty_payments: false,
            royalty_percentage: None,
            royalty_payment_address: None,
        };

        instantiate(deps.as_mut(), mock_env(), creator_info.clone(), init_msg).unwrap();

        let token_id = "Enterprise";
        let coins = [coin(1_000_000, "uluna")];
        let success_info = mock_info(CREATOR, &coins);
        let mint_msg = MintMsg {
            token_id: token_id.to_string(),
            owner: OWNER.to_string(),
            token_uri: None,
            extension: None,
        };

        let exec_msg = ExecuteMsg::Mint(mint_msg);
        execute(deps.as_mut(), mock_env(), success_info, exec_msg.clone()).unwrap();

        let fail_info = mock_info(OWNER, &[]);
        let exec_msg = ExecuteMsg::Withdraw {
            amount: coins.to_vec(),
        };

        // Unauthorized withdraw
        let res = execute(deps.as_mut(), mock_env(), fail_info, exec_msg.clone());
        assert_eq!(ContractError::Unauthorized {}, res.unwrap_err());

        // Owner withdraw
        let res = execute(deps.as_mut(), mock_env(), creator_info, exec_msg);
        assert!(res.is_ok());
    }

    // Burn tests

    #[test]
    fn burn() {
        let mut deps = mock_dependencies(&[]);
        let contract = Cw721Contract::<Extension, Empty>::default();

        // Instantiate the contract
        let init_info = mock_info(CREATOR, &[]);
        let init_msg = InstantiateMsg {
            name: "SpaceShips".to_string(),
            symbol: "SPACE".to_string(),
            max_token_count: 1,
            treasury_account: CREATOR.to_string(),
            is_mint_public: true,
            start_time: None,
            end_time: None,
            price: None,
            royalty_payments: false,
            royalty_percentage: None,
            royalty_payment_address: None,
        };
        instantiate(deps.as_mut(), mock_env(), init_info.clone(), init_msg).unwrap();

        // Mint an NFT
        let token_id = "Enterprise";
        let mint_info = mock_info(OWNER, &[]);
        let mint_msg = MintMsg {
            token_id: token_id.to_string(),
            owner: OWNER.to_string(),
            token_uri: None,
            extension: None,
        };

        let exec_msg = ExecuteMsg::Mint(mint_msg.clone());
        execute(deps.as_mut(), mock_env(), mint_info.clone(), exec_msg).unwrap();
        let token_count = contract.token_count(&deps.storage).unwrap();

        assert_eq!(token_count, 1);

        // Burn an NFT
        let burn_info = mock_info(OWNER, &[]);
        let exec_msg = ExecuteMsg::Burn {
            token_id: token_id.to_string(),
        };

        execute(deps.as_mut(), mock_env(), burn_info.clone(), exec_msg).unwrap();
        let token_count = contract.token_count(&deps.storage).unwrap();
        // Token count decrements
        assert_eq!(token_count, 0);
        let res =
            Cw721Contract::<Extension, Empty>::default().nft_info(deps.as_ref(), token_id.into());
        match res {
            Ok(_) => panic!("Should not return token info"),
            Err(_) => {}
        }
    }

    #[test]
    fn royalties_success() {
        let mut deps = mock_dependencies(&[]);
        let contract = Cw721Contract::<Extension, Empty>::default();

        // Instantiate the contract - success
        let init_info = mock_info(CREATOR, &[]);
        let init_msg = InstantiateMsg {
            name: "SpaceShips".to_string(),
            symbol: "SPACE".to_string(),
            max_token_count: 1,
            treasury_account: CREATOR.to_string(),
            is_mint_public: true,
            start_time: None,
            end_time: None,
            price: None,
            royalty_payments: true,
            royalty_percentage: Some(10),
            royalty_payment_address: Some(CREATOR.to_string()),
        };
        let res = instantiate(deps.as_mut(), mock_env(), init_info.clone(), init_msg).unwrap();

        assert_eq!(res.attributes.len(), 0);

        let config = CONFIG.load(&deps.storage).unwrap();
        assert_eq!(config.name, "SpaceShips");

        // Mint an NFT
        let token_id = "Enterprise";
        let mint_info = mock_info(OWNER, &[]);
        let mint_msg = MintMsg {
            token_id: token_id.to_string(),
            owner: OWNER.to_string(),
            token_uri: None,
            extension: None,
        };

        let exec_msg = ExecuteMsg::Mint(mint_msg.clone());
        execute(deps.as_mut(), mock_env(), mint_info.clone(), exec_msg).unwrap();
        let token_count = contract.token_count(&deps.storage).unwrap();

        assert_eq!(token_count, 1);

        let royal = ROYALTIES_INFO.load(&deps.storage).unwrap();
        assert_eq!(royal.royalty_payments, true);
        assert_eq!(royal.royalty_percentage, 10);

        let token_id_real = token_id.to_string();
        let sale_price_real = 100;

        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::RoyaltyInfo {
                sale_price: sale_price_real,
                token_id: token_id_real.to_string(),
            },
        )
        .unwrap();

        let value: RoyaltiesInfoResponse = from_binary(&res).unwrap();

        println!("{:?}", value);

        /* assert_eq!("creator".to_string(), value.address);
        assert_eq!(100000, value.royalty_amount); */
    }

    #[test]
    fn royalties_fail() {
        let mut deps = mock_dependencies(&[]);
        let _contract = Cw721Contract::<Extension, Empty>::default();

        // Instantiate the contract - fails
        let init_info = mock_info(CREATOR, &[]);
        let init_msg = InstantiateMsg {
            name: "SpaceShips".to_string(),
            symbol: "SPACE".to_string(),
            max_token_count: 1,
            treasury_account: CREATOR.to_string(),
            is_mint_public: true,
            start_time: None,
            end_time: None,
            price: None,
            royalty_payments: true,
            royalty_percentage: None,
            royalty_payment_address: None,
        };
        let res = instantiate(deps.as_mut(), mock_env(), init_info.clone(), init_msg);

        //Should not initiate because royalty percentage and royalt payment address is not supplied
        match res {
            Ok(_) => panic!("Should return anything because it's supposed to error"),
            Err(_) => {}
        }

        // Instantiate the contract - fails again
        let init_info = mock_info(CREATOR, &[]);
        let init_msg = InstantiateMsg {
            name: "SpaceShips".to_string(),
            symbol: "SPACE".to_string(),
            max_token_count: 1,
            treasury_account: CREATOR.to_string(),
            is_mint_public: true,
            start_time: None,
            end_time: None,
            price: None,
            royalty_payments: true,
            royalty_percentage: Some(110),
            royalty_payment_address: None,
        };

        let res = instantiate(deps.as_mut(), mock_env(), init_info.clone(), init_msg);

        //Should not initiate because royalty percentage is above 100
        match res {
            Ok(_) => panic!("Should not return anything because it's supposed to error"),
            Err(_) => {}
        }

        // Instantiate the contract - fails again again
        let init_info = mock_info(CREATOR, &[]);
        let init_msg = InstantiateMsg {
            name: "SpaceShips".to_string(),
            symbol: "SPACE".to_string(),
            max_token_count: 1,
            treasury_account: CREATOR.to_string(),
            is_mint_public: true,
            start_time: None,
            end_time: None,
            price: None,
            royalty_payments: true,
            royalty_percentage: Some(10),
            royalty_payment_address: None,
        };

        let res = instantiate(deps.as_mut(), mock_env(), init_info.clone(), init_msg);

        //Should not initiate because royalty payment address is not set
        match res {
            Ok(_) => panic!("Should not return anything because it's supposed to error"),
            Err(_) => {}
        }

        // Instantiate the contract - fails thrice
        let init_info = mock_info(CREATOR, &[]);
        let init_msg = InstantiateMsg {
            name: "SpaceShips".to_string(),
            symbol: "SPACE".to_string(),
            max_token_count: 1,
            treasury_account: CREATOR.to_string(),
            is_mint_public: true,
            start_time: None,
            end_time: None,
            price: None,
            royalty_payments: true,
            royalty_percentage: None,
            royalty_payment_address: Some("CREATOR".to_string()),
        };

        let res = instantiate(deps.as_mut(), mock_env(), init_info.clone(), init_msg);

        //Should not initiate because royalty percentage is not set
        match res {
            Ok(_) => panic!("Should not return anything because it's supposed to error"),
            Err(_) => {}
        }
    }
}
