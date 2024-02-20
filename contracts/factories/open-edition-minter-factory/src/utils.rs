use cosmwasm_std::{Coin, Uint128};

use crate::error::ContractError;

pub fn check_payment(sent_funds: &[Coin], expected_funds: &[Coin]) -> Result<(), ContractError> {
    // Remove 0 amounts
    let expected_funds = expected_funds
        .iter()
        .filter(|coin| coin.amount > Uint128::zero())
        .cloned() // Clone the elements
        .collect::<Vec<_>>();

    // Check length
    if sent_funds.len() > expected_funds.len() {
        return Err(ContractError::IncorrectFunds {
            expected: expected_funds.clone().to_vec(),
            actual: sent_funds.to_vec(),
        });
    }

    let mut mut_sent_funds = sent_funds.to_vec(); // Create a mutable copy

    for expected in expected_funds.clone() {
        if let Some(sent_index) = mut_sent_funds
            .iter()
            .position(|sent| expected.denom == sent.denom)
        {
            let sent = &mut mut_sent_funds[sent_index];
            if expected.amount > sent.amount {
                return Err(ContractError::IncorrectFunds {
                    expected: expected_funds.to_vec(),
                    actual: sent_funds.to_vec(),
                });
            } else {
                sent.amount = sent.amount.checked_sub(expected.amount).unwrap();
            }
        } else {
            return Err(ContractError::IncorrectFunds {
                expected: expected_funds.to_vec(),
                actual: sent_funds.to_vec(),
            });
        }
    }

    Ok(())
}

// Test check_payment
#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::coin;

    #[test]
    fn test_check_payment() {
        let sent_funds = vec![coin(100, "uluna"), coin(100, "uusd")];
        let expected_funds = vec![coin(100, "uluna"), coin(100, "uusd")];
        let res = check_payment(&sent_funds, &expected_funds);
        assert!(res.is_ok());

        let sent_funds = vec![coin(100, "uluna"), coin(100, "uusd")];
        let expected_funds = vec![coin(100, "uluna")];
        let res = check_payment(&sent_funds, &expected_funds);
        assert!(res.is_err());

        let sent_funds = vec![coin(100, "uluna")];
        let expected_funds = vec![coin(100, "uluna"), coin(100, "uusd")];
        let res = check_payment(&sent_funds, &expected_funds);
        assert!(res.is_err());

        let sent_funds = vec![coin(100, "uluna"), coin(100, "uusd")];
        let expected_funds = vec![coin(100, "uluna"), coin(200, "uusd")];
        let res = check_payment(&sent_funds, &expected_funds);
        assert!(res.is_err());

        let sent_funds = vec![coin(300, "uluna")];
        let expected_funds = vec![coin(100, "uluna"), coin(200, "uluna")];
        let res = check_payment(&sent_funds, &expected_funds);
        assert!(res.is_ok());

        let sent_funds = vec![coin(300 - 1, "uluna")];
        let expected_funds = vec![coin(100, "uluna"), coin(200, "uluna")];
        let res = check_payment(&sent_funds, &expected_funds);
        assert!(res.is_err());

        let sent_funds = vec![coin(300, "uluna"), coin(100, "uusd")];
        let expected_funds = vec![coin(300, "uluna"), coin(200, "uatom")];
        let res = check_payment(&sent_funds, &expected_funds);
        assert!(res.is_err());

        let sent_funds = vec![coin(1100, "uluna")];
        let expected_funds = vec![
            coin(100, "uluna"),
            coin(200, "uluna"),
            coin(300, "uluna"),
            coin(500, "uluna"),
        ];
        let res = check_payment(&sent_funds, &expected_funds);
        assert!(res.is_ok());

        let sent_funds = vec![coin(1100 + 1, "uluna")];
        let expected_funds = vec![
            coin(100, "uluna"),
            coin(200, "uluna"),
            coin(300, "uluna"),
            coin(500, "uluna"),
        ];
        let res = check_payment(&sent_funds, &expected_funds);
        assert!(res.is_ok());

        let sent_funds = vec![coin(1100 - 1, "uluna")];
        let expected_funds = vec![
            coin(100, "uluna"),
            coin(200, "uluna"),
            coin(300, "uluna"),
            coin(500, "uluna"),
        ];
        let res = check_payment(&sent_funds, &expected_funds);
        assert!(res.is_err());

        let sent_funds = vec![coin(1100, "uluna")];
        let expected_funds = vec![coin(0, "something"), coin(1100, "uluna")];
        let res = check_payment(&sent_funds, &expected_funds);
        assert!(res.is_ok());
    }
}
