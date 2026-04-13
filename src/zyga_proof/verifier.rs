use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct VerificationResult {
    pub valid: bool,
    pub message: String,
}

#[derive(Debug, Clone)]
pub struct VerificationError(pub String);

impl std::fmt::Display for VerificationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Verification error: {}", self.0)
    }
}

impl std::error::Error for VerificationError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BalanceTransition {
    pub delta: u64,
    pub old_commitment: String,
    pub new_commitment: String,
}

pub fn calculate_liquidation_price_long(entry_price: u64, initial_margin: u64, notional: u64) -> u64 {
    let opening_fee = (notional as f64 * 0.01).ceil() as u64;
    let effective_margin = initial_margin.saturating_sub(opening_fee);
    let margin_fraction = effective_margin as f64 / notional as f64;
    (entry_price as f64 * (1.0 - margin_fraction)) as u64
}

pub fn calculate_liquidation_price_short(entry_price: u64, initial_margin: u64, notional: u64) -> u64 {
    let opening_fee = (notional as f64 * 0.01).ceil() as u64;
    let effective_margin = initial_margin.saturating_sub(opening_fee);
    let margin_fraction = effective_margin as f64 / notional as f64;
    (entry_price as f64 * (1.0 + margin_fraction)) as u64
}

pub fn verify_liquidation_proof(
    proof_mark_price: u64,
    server_liquidation_price: u64,
    is_long: bool,
) -> Result<VerificationResult, VerificationError> {
    let comparison_valid = if is_long {
        proof_mark_price >= server_liquidation_price
    } else {
        proof_mark_price <= server_liquidation_price
    };

    if !comparison_valid {
        return Ok(VerificationResult {
            valid: false,
            message: format!(
                "Verification failed: mark_price {} {} server_liquidation_price {}",
                proof_mark_price,
                if is_long { "<" } else { ">" },
                server_liquidation_price
            ),
        });
    }

    Ok(VerificationResult {
        valid: true,
        message: format!(
            "Verification passed: mark_price {} {} server_liquidation_price {}",
            proof_mark_price,
            if is_long { ">=" } else { "<=" },
            server_liquidation_price
        ),
    })
}

pub fn extract_mark_price_from_public_inputs(public_inputs: &Map<String, Value>) -> Result<u64, VerificationError> {
    let value = public_inputs
        .get("mark_price")
        .ok_or_else(|| VerificationError("public_inputs.mark_price is missing".to_string()))?;

    match value {
        Value::Number(number) => number
            .as_u64()
            .ok_or_else(|| VerificationError("public_inputs.mark_price is not a u64".to_string())),
        Value::String(text) => text
            .parse::<u64>()
            .map_err(|_| VerificationError("public_inputs.mark_price is not parseable".to_string())),
        _ => Err(VerificationError(
            "public_inputs.mark_price must be a number or string".to_string(),
        )),
    }
}

fn extract_string_value(value: &Value, field: &str) -> Result<String, VerificationError> {
    match value {
        Value::String(text) => Ok(text.clone()),
        Value::Number(number) => Ok(number.to_string()),
        _ => Err(VerificationError(format!("{field} must be a number or string"))),
    }
}

pub fn extract_balance_transition(
    proof: &Value,
    expected_delta: u64,
) -> Result<BalanceTransition, String> {
    let public_inputs = proof
        .get("public_inputs")
        .and_then(Value::as_object)
        .ok_or_else(|| "proof.public_inputs is missing".to_string())?;

    let delta = public_inputs
        .get("delta")
        .ok_or_else(|| "proof.public_inputs.delta is missing".to_string())
        .and_then(|value| match value {
            Value::Number(number) => number
                .as_u64()
                .ok_or_else(|| "proof.public_inputs.delta is not a u64".to_string()),
            Value::String(text) => text
                .parse::<u64>()
                .map_err(|_| "proof.public_inputs.delta is not parseable".to_string()),
            _ => Err("proof.public_inputs.delta must be a number or string".to_string()),
        })?;

    if delta != expected_delta {
        return Err(format!(
            "proof delta {} does not match expected amount {}",
            delta, expected_delta
        ));
    }

    let old_commitment = public_inputs
        .get("old_commitment")
        .ok_or_else(|| "proof.public_inputs.old_commitment is missing".to_string())
        .and_then(|value| extract_string_value(value, "proof.public_inputs.old_commitment").map_err(|err| err.to_string()))?;
    let new_commitment = public_inputs
        .get("new_commitment")
        .ok_or_else(|| "proof.public_inputs.new_commitment is missing".to_string())
        .and_then(|value| extract_string_value(value, "proof.public_inputs.new_commitment").map_err(|err| err.to_string()))?;

    Ok(BalanceTransition {
        delta,
        old_commitment,
        new_commitment,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn long_liquidation_price_decreases_from_entry() {
        let price = calculate_liquidation_price_long(1_000_000, 100_000, 1_000_000);
        assert!(price < 1_000_000);
    }

    #[test]
    fn short_liquidation_price_increases_from_entry() {
        let price = calculate_liquidation_price_short(1_000_000, 100_000, 1_000_000);
        assert!(price > 1_000_000);
    }

    #[test]
    fn verify_long_solvent() {
        let result = verify_liquidation_proof(1_000_000, 900_000, true).unwrap();
        assert!(result.valid);
    }

    #[test]
    fn verify_long_insolvent() {
        let result = verify_liquidation_proof(800_000, 900_000, true).unwrap();
        assert!(!result.valid);
    }

    #[test]
    fn verify_short_solvent() {
        let result = verify_liquidation_proof(900_000, 1_000_000, false).unwrap();
        assert!(result.valid);
    }

    #[test]
    fn verify_short_insolvent() {
        let result = verify_liquidation_proof(1_100_000, 1_000_000, false).unwrap();
        assert!(!result.valid);
    }

    #[test]
    fn extract_mark_price_from_numeric_public_inputs() {
        let public_inputs = json!({ "1": 1, "mark_price": 12345 })
            .as_object()
            .unwrap()
            .clone();
        assert_eq!(extract_mark_price_from_public_inputs(&public_inputs).unwrap(), 12345);
    }

    #[test]
    fn extract_mark_price_from_string_public_inputs() {
        let public_inputs = json!({ "1": "1", "mark_price": "12345" })
            .as_object()
            .unwrap()
            .clone();
        assert_eq!(extract_mark_price_from_public_inputs(&public_inputs).unwrap(), 12345);
    }

    #[test]
    fn extract_balance_transition_from_string_inputs() {
        let proof = json!({
            "public_inputs": {
                "delta": "1000",
                "old_commitment": "123",
                "new_commitment": "456"
            }
        });
        let transition = extract_balance_transition(&proof, 1000).unwrap();
        assert_eq!(transition.old_commitment, "123");
        assert_eq!(transition.new_commitment, "456");
    }
}
