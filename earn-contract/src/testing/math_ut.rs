
use cosmwasm_bignumber::{Decimal256};

use crate::math::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_a_terra_exchange_rate() {
        assert_eq!(
            "1",
            ExchangeRate::a_terra_exchange_rate(Decimal256::zero())
                .unwrap()
                .to_string()
        );
        assert_eq!(
            "1.20000000000",
            ExchangeRate::a_terra_exchange_rate(Decimal256::from_ratio(365, 1))
                .unwrap()
                .to_string()
                .get(0..13)
                .unwrap()
        );
        assert_eq!(
            "1.44000000000",
            ExchangeRate::a_terra_exchange_rate(Decimal256::from_ratio(2 * 365, 1))
                .unwrap()
                .to_string()
                .get(0..13)
                .unwrap()
        );
        assert_eq!(
            "1.57744096561",
            ExchangeRate::a_terra_exchange_rate(Decimal256::from_ratio(5 * 365, 2))
                .unwrap()
                .to_string()
                .get(0..13)
                .unwrap()
        );
    }
    #[test]
    fn test_invert_a_terra_exchange_rate() {
        assert_eq!(
            Decimal256::zero(),
            ExchangeRate::invert_a_terra_exchange_rate(
                ExchangeRate::a_terra_exchange_rate(Decimal256::zero()).unwrap()
            )
            .unwrap()
        );
        assert_eq!(
            "365.0000000000",
            ExchangeRate::invert_a_terra_exchange_rate(
                ExchangeRate::a_terra_exchange_rate(Decimal256::from_ratio(365, 1)).unwrap()
            )
            .unwrap()
            .to_string()
            .get(0..14)
            .unwrap()
        );
        assert_eq!(
            "1.00000000000",
            ExchangeRate::invert_a_terra_exchange_rate(
                ExchangeRate::a_terra_exchange_rate(Decimal256::one()).unwrap()
            )
            .unwrap()
            .to_string()
            .get(0..13)
            .unwrap()
        );
        assert_eq!(
            "1.50000000000",
            ExchangeRate::invert_a_terra_exchange_rate(
                ExchangeRate::a_terra_exchange_rate(Decimal256::from_ratio(3, 2)).unwrap()
            )
            .unwrap()
            .to_string()
            .get(0..13)
            .unwrap()
        );
    }

    #[test]
    fn test_floor() {
        assert_eq!("0", Decimal256::floor(Decimal256::zero()).to_string());
        assert_eq!("1", Decimal256::floor(Decimal256::one()).to_string());
        assert_eq!(
            "0",
            Decimal256::floor(Decimal256::from_ratio(1, 2)).to_string()
        );
        assert_eq!(
            "1",
            Decimal256::floor(Decimal256::from_ratio(3, 2)).to_string()
        );
    }

    #[test]
    fn test_two_power_n() {
        assert_eq!("1", Decimal256::two_power_n(Decimal256::zero()).to_string());

        assert_eq!("2", Decimal256::two_power_n(Decimal256::one()).to_string());
        assert_eq!(
            "4",
            Decimal256::two_power_n(Decimal256::from_ratio(2, 1)).to_string()
        );
        assert_eq!(
            "8",
            Decimal256::two_power_n(Decimal256::from_ratio(3, 1)).to_string()
        );
        assert_eq!(
            "1024",
            Decimal256::two_power_n(Decimal256::from_ratio(10, 1)).to_string()
        );
    }

    #[test]
    fn test_exp_fixedpoint() {
        assert_eq!(
            "1",
            Decimal256::exp(Decimal256::zero()).unwrap().to_string()
        );

        assert_eq!(
            "2.71828182845904",
            Decimal256::exp(Decimal256::one())
                .unwrap()
                .to_string()
                .get(0..16)
                .unwrap()
        );

        assert_eq!(
            "90.0171313005218",
            Decimal256::exp(Decimal256::from_ratio(9, 2))
                .unwrap()
                .to_string()
                .get(0..16)
                .unwrap()
        );

        assert_eq!(
            "7.38905609893065",
            Decimal256::exp(Decimal256::from_ratio(2, 1))
                .unwrap()
                .to_string()
                .get(0..16)
                .unwrap()
        );

        assert_eq!(
            "3.79366789468317",
            Decimal256::exp(Decimal256::from_ratio(4, 3))
                .unwrap()
                .to_string()
                .get(0..16)
                .unwrap()
        );
    }
    #[test]
    fn test_ln_fixedpoint() {
        assert_eq!("0", Decimal256::ln(Decimal256::one()).unwrap().to_string());

        assert_eq!(
            "0.51082562376599",
            Decimal256::ln(Decimal256::from_ratio(5, 3))
                .unwrap()
                .to_string()
                .get(0..16)
                .unwrap()
        );

        assert_eq!(
            "0.69314718055994",
            Decimal256::ln(Decimal256::from_ratio(2, 1))
                .unwrap()
                .to_string()
                .get(0..16)
                .unwrap()
        );

        assert_eq!(
            "1.09861228866811",
            Decimal256::ln(Decimal256::from_ratio(3, 1))
                .unwrap()
                .to_string()
                .get(0..16)
                .unwrap()
        );

        assert_eq!(
            "2.56494935746153",
            Decimal256::ln(Decimal256::from_ratio(13, 1))
                .unwrap()
                .to_string()
                .get(0..16)
                .unwrap()
        );
    }

    #[test]
    fn test_a_terra_exchange_rate_fixedpoint() {
        assert_eq!(
            "1",
            ExchangeRate::a_terra_exchange_rate(Decimal256::zero())
                .unwrap()
                .to_string()
        );
        assert_eq!(
            "1.20000000000",
            ExchangeRate::a_terra_exchange_rate(Decimal256::from_ratio(365, 1))
                .unwrap()
                .to_string()
                .get(0..13)
                .unwrap()
        );
        assert_eq!(
            "1.44000000000",
            ExchangeRate::a_terra_exchange_rate(Decimal256::from_ratio(2 * 365, 1))
                .unwrap()
                .to_string()
                .get(0..13)
                .unwrap()
        );
        assert_eq!(
            "1.57744096561",
            ExchangeRate::a_terra_exchange_rate(Decimal256::from_ratio(5 * 365, 2))
                .unwrap()
                .to_string()
                .get(0..13)
                .unwrap()
        );
    }
}
