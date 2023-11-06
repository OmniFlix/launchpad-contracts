#[cfg(test)]
mod tests {

    use crate::error::ContractError;
    use crate::msg::{CollectionDetails, ExecuteMsg, InstantiateMsg, QueryMsg};

    use crate::contract::{execute, instantiate, query};
    use crate::state::{Config, Round, Token, UserDetails};

    use cosmwasm_std::testing::{mock_dependencies, mock_info};
    use cosmwasm_std::{coin, from_binary, to_binary, Decimal, StdError};
    use cosmwasm_std::{testing::mock_env, Addr, Timestamp, TransactionInfo, Uint128};
    use cw_utils::PaymentError;
    use omniflix_std::types::omniflix::onft::v1beta1::{Metadata, MsgCreateDenom, MsgMintOnft};

    pub fn return_instantiate_msg() -> InstantiateMsg {
        let collection_details = CollectionDetails {
            name: "name".to_string(),
            description: "description".to_string(),
            preview_uri: "preview_uri".to_string(),
            schema: "schema".to_string(),
            symbol: "symbol".to_string(),
            id: "id".to_string(),
            extensible: true,
            nsfw: false,
            num_tokens: 1000,
            base_uri: "base_uri".to_string(),
        };

        let instantiate_msg = InstantiateMsg {
            per_address_limit: 10,
            creator: Some("creator".to_string()),
            collection_details: collection_details,
            rounds: None,
            mint_denom: "uflix".to_string(),
            start_time: Timestamp::from_nanos(782784568767866),
            mint_price: Uint128::from(1000000u128),
            royalty_ratio: "0.1".to_string(), //0.1
            payment_collector: Some("payment_collector".to_string()),
        };
        instantiate_msg
    }

    #[test]
    fn test_proper_init() {
        let mut env = mock_env();
        env.block.height = 100_000_000;
        env.block.time = Timestamp::from_nanos(100_000_000);
        env.transaction = Some(TransactionInfo { index: 100_000_000 });
        let mut deps = mock_dependencies();

        let instantiate_msg = return_instantiate_msg();

        // Send no funds
        let info = mock_info("creator", &[]);
        let res =
            instantiate(deps.as_mut(), env.clone(), info, instantiate_msg.clone()).unwrap_err();

        assert_eq!(res, ContractError::PaymentError(PaymentError::NoFunds {}));

        // Send incorrect denom
        let instantiate_msg = return_instantiate_msg();

        let info = mock_info("creator", &[coin(1000000, "incorrect_denom")]);
        let res =
            instantiate(deps.as_mut(), env.clone(), info, instantiate_msg.clone()).unwrap_err();
        assert_eq!(
            res,
            ContractError::PaymentError(PaymentError::MissingDenom("uflix".to_string()))
        );
        // Send correct denom incorrect amount
        let instantiate_msg = return_instantiate_msg();

        let info = mock_info("creator", &[coin(1000000, "uflix")]);
        let res =
            instantiate(deps.as_mut(), env.clone(), info, instantiate_msg.clone()).unwrap_err();
        assert_eq!(
            res,
            ContractError::InvalidCreationFee {
                expected: Uint128::from(1000000u128),
                sent: Uint128::from(1000000u128)
            }
        );

        // Send 0 num tokens
        let mut instantiate_msg = return_instantiate_msg();

        instantiate_msg.collection_details.num_tokens = 0;
        let info = mock_info("creator", &[coin(100000000, "uflix")]);
        let res =
            instantiate(deps.as_mut(), env.clone(), info, instantiate_msg.clone()).unwrap_err();
        assert_eq!(res, ContractError::InvalidNumTokens {});

        // Send royalty ratio more than 100%
        let mut instantiate_msg = return_instantiate_msg();

        let ratio_1: u64 = 1_000_000_000_000_000_000;
        instantiate_msg.royalty_ratio = "1.1".to_string();
        let info = mock_info("creator", &[coin(100000000, "uflix")]);
        let res =
            instantiate(deps.as_mut(), env.clone(), info, instantiate_msg.clone()).unwrap_err();
        assert_eq!(res, ContractError::InvalidRoyaltyRatio {});

        // Send mint price 0
        let mut instantiate_msg = return_instantiate_msg();

        instantiate_msg.mint_price = Uint128::zero();
        let info = mock_info("creator", &[coin(100000000, "uflix")]);
        let res =
            instantiate(deps.as_mut(), env.clone(), info, instantiate_msg.clone()).unwrap_err();
        assert_eq!(res, ContractError::InvalidMintPrice {});

        // Incorrect start time
        let mut instantiate_msg = return_instantiate_msg();

        instantiate_msg.start_time = Timestamp::from_nanos(1_000_000 - 1);
        let mut env = mock_env();
        env.block.time = Timestamp::from_nanos(1_000_000);
        let info = mock_info("creator", &[coin(100000000, "uflix")]);
        let res =
            instantiate(deps.as_mut(), env.clone(), info, instantiate_msg.clone()).unwrap_err();
        assert_eq!(res, ContractError::InvalidStartTime {});

        // HAPPY PATH
        let instantiate_msg = return_instantiate_msg();

        let info = mock_info("creator", &[coin(100000000, "uflix")]);
        let res = instantiate(deps.as_mut(), env.clone(), info, instantiate_msg.clone()).unwrap();
        let response = res.messages[0].clone().msg;
        assert_eq!(
            response,
            MsgCreateDenom {
                id: "id".to_string(),
                symbol: "symbol".to_string(),
                name: "name".to_string(),
                schema: "schema".to_string(),
                description: "description".to_string(),
                preview_uri: "preview_uri".to_string(),
                sender: env.clone().contract.address.to_string(),
                creation_fee: Some(omniflix_std::types::cosmos::base::v1beta1::Coin {
                    amount: "100000000".to_string(),
                    denom: "uflix".to_string(),
                }),
            }
            .into()
        );

        // Query config
        let config_data = query(deps.as_ref(), env.clone(), QueryMsg::Config {}).unwrap();
        let config: Config = from_binary(&config_data).unwrap();
        assert_eq!(config.per_address_limit, 10);
        assert_eq!(config.mint_denom, "uflix".to_string());
        assert_eq!(config.start_time, Timestamp::from_nanos(782784568767866));
        assert_eq!(config.mint_price, Uint128::from(1000000u128));
        assert_eq!(config.royalty_ratio, Decimal::from_ratio(1u128, 10u128));
        assert_eq!(config.creator, Addr::unchecked("creator"));
        assert_eq!(
            config.payment_collector,
            Addr::unchecked("payment_collector")
        );

        // query collection
        let collection_data = query(deps.as_ref(), env.clone(), QueryMsg::Collection {}).unwrap();
        let collection: CollectionDetails = from_binary(&collection_data).unwrap();
        assert_eq!(collection.name, "name".to_string());
        assert_eq!(collection.description, "description".to_string());
        assert_eq!(collection.preview_uri, "preview_uri".to_string());
        assert_eq!(collection.schema, "schema".to_string());
        assert_eq!(collection.symbol, "symbol".to_string());
        assert_eq!(collection.id, "id".to_string());
        assert_eq!(collection.extensible, true);
        assert_eq!(collection.nsfw, false);
        assert_eq!(collection.num_tokens, 1000);

        // query mintable tokens
        let mintable_tokens_data =
            query(deps.as_ref(), env.clone(), QueryMsg::MintableTokens {}).unwrap();
        let mintable_tokens: Vec<Token> = from_binary(&mintable_tokens_data).unwrap();
        assert_eq!(mintable_tokens.len(), 1000);
        // This is not a proper check but I am making sure list is randomized and is not starting from 1
        assert_ne!(mintable_tokens[0].token_id, 1.to_string());

        // Check total tokens remaining
        let total_tokens_remaining_data =
            query(deps.as_ref(), env.clone(), QueryMsg::TotalTokens {}).unwrap();
        let total_tokens_remaining: u32 = from_binary(&total_tokens_remaining_data).unwrap();
        assert_eq!(total_tokens_remaining, 1000);
    }

    #[test]
    pub fn test_mint() {
        let mut env = mock_env();
        env.block.height = 657625347635765;
        env.block.time = Timestamp::from_nanos(782784568767866);
        env.transaction = Some(TransactionInfo { index: 12147492 });
        let mut deps = mock_dependencies();

        let mut instantiate_msg = return_instantiate_msg();
        instantiate_msg.per_address_limit = 1;

        // instantiate
        let info = mock_info("creator", &[coin(100000000, "uflix")]);
        let _res = instantiate(deps.as_mut(), env.clone(), info, instantiate_msg.clone()).unwrap();

        // mint without funds
        let info = mock_info("creator", &[]);
        let res = execute(deps.as_mut(), env.clone(), info, ExecuteMsg::Mint {}).unwrap_err();
        assert_eq!(res, ContractError::PaymentError(PaymentError::NoFunds {}));

        // mint with incorrect denom
        let info = mock_info("creator", &[coin(1000000, "incorrect_denom")]);
        let res = execute(deps.as_mut(), env.clone(), info, ExecuteMsg::Mint {}).unwrap_err();
        assert_eq!(
            res,
            ContractError::PaymentError(PaymentError::MissingDenom("uflix".to_string()))
        );

        // mint with incorrect amount
        let info = mock_info("creator", &[coin(100000, "uflix")]);
        let res = execute(deps.as_mut(), env.clone(), info, ExecuteMsg::Mint {}).unwrap_err();
        assert_eq!(
            res,
            ContractError::IncorrectPaymentAmount {
                expected: Uint128::from(1000000u128),
                sent: Uint128::from(100000u128)
            }
        );
        // Try minting before start time
        let mut env = mock_env();
        env.block.time = Timestamp::from_nanos(1_000_000);
        let info = mock_info("creator", &[coin(1000000, "uflix")]);
        let res = execute(deps.as_mut(), env.clone(), info, ExecuteMsg::Mint {}).unwrap_err();
        assert_eq!(
            res,
            ContractError::MintingNotStarted {
                start_time: Timestamp::from_nanos(782784568767866).nanos(),
                current_time: Timestamp::from_nanos(1_000_000).nanos()
            }
        );

        // Mint
        let env = mock_env();
        let info = mock_info("creator", &[coin(1000000, "uflix")]);
        let res = execute(deps.as_mut(), env.clone(), info, ExecuteMsg::Mint {}).unwrap();
        let response = res.messages[0].clone().msg;
        assert_eq!(
            response,
            MsgMintOnft {
                data: "".to_string(),
                // With this env parameters token id for mint is 334
                denom_id: "id".to_string(),
                id: "id425".to_string(),
                recipient: "creator".to_string(),
                royalty_share: Decimal::from_ratio(1u128, 10u128).atomics().to_string(),
                sender: env.clone().contract.address.to_string(),
                extensible: true,
                nsfw: false,
                transferable: true,
                metadata: Some(Metadata {
                    description: "description".to_string(),
                    media_uri: "base_uri/425".to_string(),
                    name: "name # 425".to_string(),
                    preview_uri: "preview_uri".to_string(),
                })
            }
            .into()
        );

        // Check if this address minted
        let minted_tokens_data = query(
            deps.as_ref(),
            env.clone(),
            QueryMsg::MintedTokens {
                address: "creator".to_string(),
            },
        )
        .unwrap();
        let user_details: UserDetails = from_binary(&minted_tokens_data).unwrap();
        assert_eq!(
            user_details.minted_tokens[0],
            Token {
                token_id: "425".to_string()
            }
        );
        // Check total tokens remaining
        let total_tokens_remaining_data =
            query(deps.as_ref(), env.clone(), QueryMsg::TotalTokens {}).unwrap();
        let total_tokens_remaining: u32 = from_binary(&total_tokens_remaining_data).unwrap();
        assert_eq!(total_tokens_remaining, 999);

        // Check mintable tokens
        let mintable_tokens_data =
            query(deps.as_ref(), env.clone(), QueryMsg::MintableTokens {}).unwrap();
        let mintable_tokens: Vec<Token> = from_binary(&mintable_tokens_data).unwrap();
        assert_eq!(mintable_tokens.len(), 999);

        // Try minting second time with same address
        let info = mock_info("creator", &[coin(1000000, "uflix")]);
        let res = execute(deps.as_mut(), env.clone(), info, ExecuteMsg::Mint {}).unwrap_err();
        assert_eq!(res, ContractError::AddressReachedMintLimit {});

        // Create a loop from 1 to 999 and mint every remaining token to receivers
        for i in 1..=999 {
            // Mint
            let env = mock_env();
            let info = mock_info(&format!("creator # {}", i), &[coin(1000000, "uflix")]);
            let _res = execute(deps.as_mut(), env.clone(), info, ExecuteMsg::Mint {}).unwrap();
        }
        // query total mintable tokens
        let mintable_tokens_data =
            query(deps.as_ref(), env.clone(), QueryMsg::MintableTokens {}).unwrap();
        let mintable_tokens: Vec<Token> = from_binary(&mintable_tokens_data).unwrap();
        assert_eq!(mintable_tokens.len(), 0);

        // There should be no tokens left to mint
        let info = mock_info("creator", &[coin(1000000, "uflix")]);
        let res = execute(deps.as_mut(), env.clone(), info, ExecuteMsg::Mint {}).unwrap_err();
        assert_eq!(res, ContractError::NoTokensLeftToMint {});

        // Check minted tokens for address we will unwrap every query so if not failed in loop we minted correctly
        // Every token should be diffirent
        let mut minted_list: Vec<Token> = Vec::new();

        for i in 1..=999 {
            let user_details_data = query(
                deps.as_ref(),
                env.clone(),
                QueryMsg::MintedTokens {
                    address: format!("creator # {}", i),
                },
            )
            .unwrap();
            let user_details: UserDetails = from_binary(&user_details_data).unwrap();
            minted_list.push(user_details.minted_tokens[0].clone());
        }
        minted_list.sort_by(|a, b| a.token_id.cmp(&b.token_id));
        for i in 0..=997 {
            assert_ne!(minted_list[i], minted_list[i + 1]);
        }
    }

    #[test]
    pub fn test_mint_admin() {
        let mut env = mock_env();
        env.block.height = 100_000_000;
        env.block.time = Timestamp::from_nanos(100_000_000);
        env.transaction = Some(TransactionInfo { index: 100_000_000 });
        let mut deps = mock_dependencies();

        let instantiate_msg = return_instantiate_msg();

        // instantiate
        let info = mock_info("creator", &[coin(100000000, "uflix")]);
        let _res = instantiate(deps.as_mut(), env.clone(), info, instantiate_msg.clone()).unwrap();

        // Try minting with money but non payable for admin
        let info = mock_info("creator", &[coin(1000000, "uflix")]);
        let res = execute(
            deps.as_mut(),
            env.clone(),
            info,
            ExecuteMsg::MintAdmin {
                recipient: "gift_recipient".to_string(),
                denom_id: Some("334".to_string()),
            },
        )
        .unwrap_err();
        assert_eq!(
            res,
            ContractError::PaymentError(PaymentError::NonPayable {})
        );
        // Try minting
        let info = mock_info("creator", &[]);
        let _res = execute(
            deps.as_mut(),
            env.clone(),
            info,
            ExecuteMsg::MintAdmin {
                recipient: "gift_recipient".to_string(),
                denom_id: Some("334".to_string()),
            },
        )
        .unwrap();

        // Try minting with same denom
        let info = mock_info("creator", &[]);
        let res = execute(
            deps.as_mut(),
            env.clone(),
            info,
            ExecuteMsg::MintAdmin {
                recipient: "gift_recipient".to_string(),
                denom_id: Some("334".to_string()),
            },
        )
        .unwrap_err();
        assert_eq!(res, ContractError::TokenIdNotMintable {});

        // Try minting with without denom id
        let info = mock_info("creator", &[]);
        let _res = execute(
            deps.as_mut(),
            env.clone(),
            info,
            ExecuteMsg::MintAdmin {
                recipient: "gift_recipient".to_string(),
                denom_id: None,
            },
        )
        .unwrap();
        // Check minted tokens for address
        // Check second token minted with random denom id is not same as first one
        let user_details_data = query(
            deps.as_ref(),
            env.clone(),
            QueryMsg::MintedTokens {
                address: "gift_recipient".to_string(),
            },
        )
        .unwrap();

        let user_details: UserDetails = from_binary(&user_details_data).unwrap();
        let minted_tokens = user_details.minted_tokens;
        assert_ne!(
            minted_tokens[1],
            Token {
                token_id: "334".to_string()
            }
        );

        // Test random mint again
        let info = mock_info("creator", &[]);
        let _res = execute(
            deps.as_mut(),
            env.clone(),
            info,
            ExecuteMsg::MintAdmin {
                recipient: "gift_recipient".to_string(),
                denom_id: None,
            },
        )
        .unwrap();
        // Here we are not changing any entropy but that token is minted so this one must be something else
        let user_details_data = query(
            deps.as_ref(),
            env.clone(),
            QueryMsg::MintedTokens {
                address: "gift_recipient".to_string(),
            },
        )
        .unwrap();

        let user_details: UserDetails = from_binary(&user_details_data).unwrap();
        assert_ne!(
            user_details.minted_tokens[2],
            Token {
                token_id: "334".to_string()
            }
        );
    }

    #[test]
    pub fn test_burn_remaining_tokens() {
        let mut env = mock_env();
        env.block.height = 100_000_000;
        env.block.time = Timestamp::from_nanos(100_000_000);
        env.transaction = Some(TransactionInfo { index: 100_000_000 });
        let mut deps = mock_dependencies();

        let instantiate_msg = return_instantiate_msg();

        // instantiate
        let info = mock_info("creator", &[coin(100000000, "uflix")]);
        let _res = instantiate(deps.as_mut(), env.clone(), info, instantiate_msg.clone()).unwrap();

        // Try burning with non creator
        let info = mock_info("non_creator", &[]);
        let res = execute(
            deps.as_mut(),
            env.clone(),
            info,
            ExecuteMsg::BurnRemainingTokens {},
        )
        .unwrap_err();
        assert_eq!(res, ContractError::Unauthorized {});

        // Try burning with creator
        let info = mock_info("creator", &[]);
        let _res = execute(
            deps.as_mut(),
            env.clone(),
            info,
            ExecuteMsg::BurnRemainingTokens {},
        )
        .unwrap();

        // Check mintable tokens
        let mintable_tokens_data =
            query(deps.as_ref(), env.clone(), QueryMsg::MintableTokens {}).unwrap();
        let mintable_tokens: Vec<Token> = from_binary(&mintable_tokens_data).unwrap();
        assert_eq!(mintable_tokens.len(), 0);
    }

    #[test]
    pub fn test_update_royalty_ratio() {
        let mut env = mock_env();
        env.block.height = 100_000_000;
        env.block.time = Timestamp::from_nanos(100_000_000);
        env.transaction = Some(TransactionInfo { index: 100_000_000 });
        let mut deps = mock_dependencies();

        let instantiate_msg = return_instantiate_msg();

        // instantiate
        let info = mock_info("creator", &[coin(100000000, "uflix")]);
        let _res = instantiate(deps.as_mut(), env.clone(), info, instantiate_msg.clone()).unwrap();

        // Try updating royalty ratio with non creator
        let info = mock_info("non_creator", &[]);
        let res = execute(
            deps.as_mut(),
            env.clone(),
            info,
            ExecuteMsg::UpdateRoyaltyRatio {
                ratio: "0.2".to_string(),
            },
        )
        .unwrap_err();
        assert_eq!(res, ContractError::Unauthorized {});

        // Try updating royalty ratio with creator
        let info = mock_info("creator", &[]);
        let _res = execute(
            deps.as_mut(),
            env.clone(),
            info,
            ExecuteMsg::UpdateRoyaltyRatio {
                ratio: "0.2".to_string(),
            },
        )
        .unwrap();

        // Check royalty ratio
        let config_data = query(deps.as_ref(), env.clone(), QueryMsg::Config {}).unwrap();
        let config: Config = from_binary(&config_data).unwrap();
        assert_eq!(config.royalty_ratio, Decimal::from_ratio(2u128, 10u128));
    }

    #[test]
    pub fn test_update_mint_price() {
        let mut env = mock_env();
        env.block.height = 100_000_000;
        env.block.time = Timestamp::from_nanos(100_000_000);
        env.transaction = Some(TransactionInfo { index: 100_000_000 });
        let mut deps = mock_dependencies();

        let instantiate_msg = return_instantiate_msg();

        // instantiate
        let info = mock_info("creator", &[coin(100000000, "uflix")]);
        let _res = instantiate(deps.as_mut(), env.clone(), info, instantiate_msg.clone()).unwrap();

        // Try updating mint price with non creator
        let info = mock_info("non_creator", &[]);
        let res = execute(
            deps.as_mut(),
            env.clone(),
            info,
            ExecuteMsg::UpdateMintPrice {
                mint_price: Uint128::from(1000000u128),
            },
        )
        .unwrap_err();
        assert_eq!(res, ContractError::Unauthorized {});

        // Try updating mint price with creator
        let info = mock_info("creator", &[]);
        let _res = execute(
            deps.as_mut(),
            env.clone(),
            info,
            ExecuteMsg::UpdateMintPrice {
                mint_price: Uint128::from(1000000u128),
            },
        )
        .unwrap();

        // Check mint price
        let config_data = query(deps.as_ref(), env.clone(), QueryMsg::Config {}).unwrap();
        let config: Config = from_binary(&config_data).unwrap();
        assert_eq!(config.mint_price, Uint128::from(1000000u128));
    }

    #[test]
    pub fn test_randomize_list() {
        let mut env = mock_env();
        env.block.height = 657625347635765;
        env.block.time = Timestamp::from_nanos(782784568767866);
        env.transaction = Some(TransactionInfo { index: 12147492 });
        let mut deps = mock_dependencies();

        let instantiate_msg = return_instantiate_msg();

        // instantiate
        let info = mock_info("creator", &[coin(100000000, "uflix")]);
        let _res = instantiate(deps.as_mut(), env.clone(), info, instantiate_msg.clone()).unwrap();

        // Try randomizing list with non creator
        let info = mock_info("non_creator", &[]);
        let res = execute(
            deps.as_mut(),
            env.clone(),
            info,
            ExecuteMsg::RandomizeList {},
        )
        .unwrap_err();
        assert_eq!(res, ContractError::Unauthorized {});

        // Check mintable tokens
        let mintable_tokens_data =
            query(deps.as_ref(), env.clone(), QueryMsg::MintableTokens {}).unwrap();
        let mintable_tokens: Vec<Token> = from_binary(&mintable_tokens_data).unwrap();
        let fifth_token = mintable_tokens[4].clone();

        // Try randomizing list with creator
        let info = mock_info("creator", &[]);
        let _res = execute(
            deps.as_mut(),
            env.clone(),
            info,
            ExecuteMsg::RandomizeList {},
        )
        .unwrap();

        // Check mintable tokens
        let mintable_tokens_data =
            query(deps.as_ref(), env.clone(), QueryMsg::MintableTokens {}).unwrap();
        let mintable_tokens: Vec<Token> = from_binary(&mintable_tokens_data).unwrap();
        assert_ne!(mintable_tokens[4], fifth_token);
    }

    #[test]
    fn test_rounds() {
        let mut env = mock_env();
        env.block.height = 100_000;
        env.block.time = Timestamp::from_nanos(100_000);
        env.transaction = Some(TransactionInfo { index: 100_000 });
        let mut deps = mock_dependencies();

        let mut instantiate_msg = return_instantiate_msg();

        // Add three rounds

        instantiate_msg.rounds = Some(vec![
            Round::WhitelistCollection {
                collection_id: "collection_id".to_string(),
                start_time: Timestamp::from_nanos(200_000),
                end_time: Timestamp::from_nanos(300_000),
                mint_price: Uint128::from(100_000u128),
                round_limit: 10,
            },
            Round::WhitelistCollection {
                collection_id: "collection_id".to_string(),
                start_time: Timestamp::from_nanos(300_000),
                end_time: Timestamp::from_nanos(400_000),
                mint_price: Uint128::from(200_000u128),
                round_limit: 10,
            },
            Round::WhitelistCollection {
                collection_id: "collection_id".to_string(),
                start_time: Timestamp::from_nanos(400_000),
                end_time: Timestamp::from_nanos(500_000),
                mint_price: Uint128::from(300_000u128),
                round_limit: 10,
            },
        ]);
        instantiate_msg.start_time = Timestamp::from_nanos(500_000);

        // instantiate
        // This is a happy path
        let info = mock_info("creator", &[coin(100000000, "uflix")]);
        let _res = instantiate(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            instantiate_msg.clone(),
        )
        .unwrap();

        // Check rounds
        let rounds_data = query(deps.as_ref(), env.clone(), QueryMsg::Rounds {}).unwrap();
        let rounds: Vec<(u32, Round)> = from_binary(&rounds_data).unwrap();
        assert_eq!(rounds.len(), 3);

        // Now make them overlaped by changing start time
        instantiate_msg.start_time = Timestamp::from_nanos(100_000);
        let res = instantiate(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            instantiate_msg.clone(),
        )
        .unwrap_err();
        // left: RoundsOverlaped { round: WhitelistAddress { address: Addr("public"), start_time: Some(Timestamp(Uint64(100000))), end_time: Some(Timestamp(Uint64(8640000000100000))), mint_price: Uint128(0), round_limit: 0 } }
        assert_eq!(
            res,
            ContractError::RoundsOverlaped {
                round: Round::WhitelistAddress {
                    // This is a hacky solution should be fixed in future
                    // For public sale I generate if its a round too
                    // Dont save it that way but returning error
                    address: Addr::unchecked("public"),
                    start_time: Some(Timestamp::from_nanos(100_000)),
                    end_time: Some(Timestamp::from_nanos(8640000000100000)),
                    mint_price: Uint128::zero(),
                    round_limit: 0,
                }
            }
        );

        // Now make them overlaped by changing round 1 end time
        instantiate_msg.start_time = Timestamp::from_nanos(500_000);
        instantiate_msg.rounds = Some(vec![
            Round::WhitelistCollection {
                collection_id: "collection_id".to_string(),
                start_time: Timestamp::from_nanos(200_000),
                end_time: Timestamp::from_nanos(300_000 + 1),
                mint_price: Uint128::from(100_000u128),
                round_limit: 10,
            },
            Round::WhitelistCollection {
                collection_id: "collection_id".to_string(),
                start_time: Timestamp::from_nanos(300_000),
                end_time: Timestamp::from_nanos(400_000),
                mint_price: Uint128::from(200_000u128),
                round_limit: 10,
            },
            Round::WhitelistCollection {
                collection_id: "collection_id".to_string(),
                start_time: Timestamp::from_nanos(400_000),
                end_time: Timestamp::from_nanos(500_000),
                mint_price: Uint128::from(300_000u128),
                round_limit: 10,
            },
        ]);
        let res = instantiate(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            instantiate_msg.clone(),
        )
        .unwrap_err();

        assert_eq!(
            res,
            ContractError::RoundsOverlaped {
                round: Round::WhitelistCollection {
                    collection_id: "collection_id".to_string(),
                    start_time: Timestamp::from_nanos(200_000),
                    end_time: Timestamp::from_nanos(300_000 + 1),
                    mint_price: Uint128::from(100_000u128),
                    round_limit: 10,
                }
            }
        );

        // Restart fresh
        let mut env = mock_env();
        env.block.height = 100_000;
        env.block.time = Timestamp::from_nanos(100_000);
        env.transaction = Some(TransactionInfo { index: 100_000 });
        let mut deps = mock_dependencies();

        let mut instantiate_msg = return_instantiate_msg();
        instantiate_msg.start_time = Timestamp::from_nanos(1_000_000);

        // Add only one round
        instantiate_msg.rounds = Some(vec![Round::WhitelistCollection {
            collection_id: "collection_id".to_string(),
            start_time: Timestamp::from_nanos(200_000),
            end_time: Timestamp::from_nanos(300_000),
            mint_price: Uint128::from(100_000u128),
            round_limit: 10,
        }]);

        // instantiate
        let info = mock_info("creator", &[coin(100000000, "uflix")]);
        let _res = instantiate(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            instantiate_msg.clone(),
        )
        .unwrap();

        // Now try adding same round again
        let res = execute(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            ExecuteMsg::AddRound {
                round: Round::WhitelistCollection {
                    collection_id: "collection_id".to_string(),
                    start_time: Timestamp::from_nanos(200_000),
                    end_time: Timestamp::from_nanos(300_000),
                    mint_price: Uint128::from(100_000u128),
                    round_limit: 10,
                },
            },
        )
        .unwrap_err();
        assert_eq!(res, ContractError::RoundAlreadyExists {});

        // Try adding a round already started
        let res = execute(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            ExecuteMsg::AddRound {
                round: Round::WhitelistCollection {
                    collection_id: "collection_id".to_string(),
                    start_time: Timestamp::from_nanos(50_000),
                    end_time: Timestamp::from_nanos(60_000),
                    mint_price: Uint128::from(100_000u128),
                    round_limit: 10,
                },
            },
        )
        .unwrap_err();
        assert_eq!(res, ContractError::RoundAlreadyStarted {});

        // Try adding overlapping round
        let res = execute(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            ExecuteMsg::AddRound {
                round: Round::WhitelistCollection {
                    collection_id: "collection_id".to_string(),
                    start_time: Timestamp::from_nanos(100_000),
                    end_time: Timestamp::from_nanos(250_000),
                    mint_price: Uint128::from(100_000u128),
                    round_limit: 10,
                },
            },
        )
        .unwrap_err();
        assert_eq!(
            res,
            ContractError::RoundsOverlaped {
                round: Round::WhitelistCollection {
                    collection_id: "collection_id".to_string(),
                    start_time: Timestamp::from_nanos(100_000),
                    end_time: Timestamp::from_nanos(250_000),
                    mint_price: Uint128::from(100_000u128),
                    round_limit: 10,
                }
            }
        );

        // Try updating collection round with wrong index
        let res = execute(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            ExecuteMsg::UpdateCollectionRound {
                round_index: 0,
                round: Round::WhitelistCollection {
                    collection_id: "collection_id".to_string(),
                    start_time: Timestamp::from_nanos(100_000),
                    end_time: Timestamp::from_nanos(250_000),
                    mint_price: Uint128::from(100_000u128),
                    round_limit: 10,
                },
            },
        )
        .unwrap_err();
        assert_eq!(
            res,
            ContractError::Std(StdError::NotFound {
                kind: "omniflix_minter::state::Round".to_string()
            })
        );

        // Try updating wrong round type
        let res = execute(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            ExecuteMsg::UpdateCollectionRound {
                round_index: 1,
                round: Round::WhitelistAddress {
                    address: Addr::unchecked("public"),
                    start_time: Some(Timestamp::from_nanos(100_000)),
                    end_time: Some(Timestamp::from_nanos(250_000)),
                    mint_price: Uint128::from(100_000u128),
                    round_limit: 10,
                },
            },
        )
        .unwrap_err();
        assert_eq!(
            res,
            ContractError::InvalidRoundType {
                expected: "collection".to_string(),
                actual: "address".to_string()
            }
        );

        // Update round
        let _res = execute(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            ExecuteMsg::UpdateCollectionRound {
                round_index: 1,
                round: Round::WhitelistCollection {
                    collection_id: "collection_id".to_string(),
                    start_time: Timestamp::from_nanos(100_000),
                    end_time: Timestamp::from_nanos(250_000),
                    mint_price: Uint128::from(100_000u128),
                    round_limit: 10,
                },
            },
        )
        .unwrap();

        // Check rounds
        let rounds_data = query(deps.as_ref(), env.clone(), QueryMsg::Rounds {}).unwrap();
        let rounds: Vec<(u32, Round)> = from_binary(&rounds_data).unwrap();
        assert_eq!(rounds.len(), 1);
        assert_eq!(
            rounds[0].1,
            Round::WhitelistCollection {
                collection_id: "collection_id".to_string(),
                start_time: Timestamp::from_nanos(100_000),
                end_time: Timestamp::from_nanos(250_000),
                mint_price: Uint128::from(100_000u128),
                round_limit: 10,
            }
        );
    }
}
