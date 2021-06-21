use cosmwasm_bignumber::{Decimal256};
use bigint::U256;
use cosmwasm_std::{StdResult, StdError};

pub trait Math {
    const TWO : Decimal256;
    const ONE_HALF : Decimal256;
    const LITTLE_ENOUGH: Decimal256;
    const M_LN2: Decimal256;
    const COEFFS: [Decimal256; 14];
    const EPSILON :  Decimal256;
    const INIT_PN : Decimal256;

    fn error(x: Decimal256, y: Decimal256) -> Decimal256;
    fn two_power_n(n: Decimal256) -> Decimal256;
    fn expm1(y: Decimal256) -> StdResult<Decimal256>;
    fn expm1_minus(y: Decimal256) -> StdResult<Decimal256>;
    fn floor(x: Decimal256) -> Decimal256;
    fn exp(x: Decimal256) -> StdResult<Decimal256>;
    fn ln(x: Decimal256) -> StdResult<Decimal256>;
    fn powf(&self, pow: Decimal256) -> StdResult<Decimal256>;
}

impl Math for Decimal256 {
    const TWO : Decimal256 = Decimal256(U256([2_000_000_000_000_000_000u64, 0, 0, 0]));
    const ONE_HALF : Decimal256 = Decimal256(U256([200_000_000_000_000_000u64, 0, 0, 0]));
    const LITTLE_ENOUGH: Decimal256 = Decimal256(U256([10_000_000_000_000u64, 0, 0, 0]));
    const M_LN2: Decimal256 = Decimal256(U256([693_147_180_559_945_309u64, 0, 0, 0]));    
    const INIT_PN: Decimal256 = Decimal256(U256([000000000011433647u64, 0, 0, 0]));
    const EPSILON :  Decimal256 = Decimal256(U256([1u64, 0, 0, 0]));
    const COEFFS: [Decimal256; 14] = [
        Decimal256(U256([000000000163246178u64, 0, 0, 0])),
        Decimal256(U256([000000002088459690u64, 0, 0, 0])),
        Decimal256(U256([000000025048614864u64, 0, 0, 0])),
        Decimal256(U256([000000275571567596u64, 0, 0, 0])),
        Decimal256(U256([000002755734045527u64, 0, 0, 0])),
        Decimal256(U256([000024801588665468u64, 0, 0, 0])),
        Decimal256(U256([000198412697873478u64, 0, 0, 0])),
        Decimal256(U256([001388888888388000u64, 0, 0, 0])),
        Decimal256(U256([008333333333342000u64, 0, 0, 0])),
        Decimal256(U256([041666666666727000u64, 0, 0, 0])),
        Decimal256(U256([166666666666680000u64, 0, 0, 0])),
        Decimal256(U256([500000000000002000u64, 0, 0, 0])),
        Decimal256(U256([1_000_000_000_000_000_000u64, 0, 0, 0])),
        Decimal256(U256([1_000_000_000_000_000_000u64, 0, 0, 0])),
        ];

    fn error(x: Decimal256, y: Decimal256) -> Decimal256 {
        if x > y {
            x - y
        } else {
            y - x
        }
    }

    fn two_power_n(n: Decimal256) -> Decimal256 {
        let mut pow = Decimal256::one();
        if n >= Decimal256::one() {
            let mut tmp = Decimal256::one();
            while tmp <= n {
                pow = pow * Decimal256::TWO;
                tmp = tmp + Decimal256::one();
            }
        }
        pow
    }

    fn floor(x: Decimal256) -> Decimal256 {
        let whole = (x.0) / Decimal256::DECIMAL_FRACTIONAL;
        Decimal256::from_uint256(whole)
    }

    fn expm1(y: Decimal256) -> StdResult<Decimal256> { 
        if y < Decimal256::LITTLE_ENOUGH {
            Ok(y + y * y * Decimal256::ONE_HALF)
        } else {
            Ok(Decimal256::exp(y)? - Decimal256::one()) //	# predefined exponential function
        }
    }

    fn expm1_minus(y: Decimal256) -> StdResult<Decimal256> {
        if y < Decimal256::LITTLE_ENOUGH {
            Ok(y - y * y * Decimal256::ONE_HALF)
        } else {
            Ok( Decimal256::one() - Decimal256::one() / Decimal256::exp(y)?) //	# predefined exponential function
        }
    }

    fn ln(x: Decimal256) -> StdResult<Decimal256> {        
        let one = Decimal256::one();
        if x < one {
            return Err(StdError::GenericErr {msg: String::from("Ln: Not in range < 1"), backtrace: None });
        }
        if x == one {
            return Ok(Decimal256::zero());
        }

        let mut result = Decimal256::zero();
        let mut xx = x.clone();
        let four_third : Decimal256 = Decimal256::from_ratio(4, 3);
        while xx > four_third {
            result = result + Decimal256::one();
            xx = xx / Decimal256::TWO;
        }
        result = result * Decimal256::M_LN2; // n*ln(2)

        let mut expy = one;

        let mut ydiff1 = Decimal256::TWO * (xx) / (xx + one);
        let mut ydiff2 = Decimal256::TWO * (one) / (xx + one);

        let mut error = one;
        while error > Decimal256::EPSILON {
            if ydiff1 > ydiff2 {
                let ydiff = ydiff1 - ydiff2;
                expy += expy * Decimal256::expm1(ydiff)?;
                result += ydiff;
            } else {
                let ydiff = ydiff2 - ydiff1;
                expy = expy - expy * Decimal256::expm1_minus(ydiff)?;
                result = result - ydiff;
            }

            ydiff1 = Decimal256::TWO * (xx) / (xx + expy);
            ydiff2 = Decimal256::TWO * (expy) / (xx + expy);

            if ydiff1 > ydiff2 {
                error = ydiff1 - ydiff2;
            } else {
                error = ydiff2 - ydiff1;
            }
        }
        Ok(result)
    }

    fn exp(x: Decimal256) -> StdResult<Decimal256> {
        if x < Decimal256::zero() || x > Decimal256::from_ratio(709, 1) {
            return Err(StdError::GenericErr {msg: String::from("Exp: Not in range < 0 or > 709"), backtrace: None });
        }      
        let one = Decimal256::one();
        if x.is_zero() {
            return Ok(one);
        }


        let x0 = x;
        let a = x / Decimal256::M_LN2;
        let b = Decimal256::ONE_HALF;
        let k;
        if a >= b {
            k = Decimal256::floor(a - b);
        } else {
            k = Decimal256::zero();
        }
        let _2_pow_k = Decimal256::two_power_n(k);
        let t = k * Decimal256::M_LN2;
        let r = x0 - t;
        let mut pn = Decimal256::INIT_PN;
        for c in Decimal256::COEFFS.iter() {
            pn = pn * r + *c;
        }
        pn = pn * _2_pow_k;
        Ok(pn)
    }

    fn powf(&self, power: Decimal256) -> StdResult<Decimal256> {
            Decimal256::exp(power * Decimal256::ln(*self)?)
    }
}

pub struct ExchangeRate;

pub trait Calculate {
    const A: Decimal256;
    const B: Decimal256;
    const LN_A: Decimal256;

    fn a_terra_exchange_rate(day: Decimal256) -> StdResult<Decimal256>;
    fn invert_a_terra_exchange_rate(exchange_rate: Decimal256) -> StdResult<Decimal256>;
    fn capapult_exchange_rate(a_terra_exchange_rate: Decimal256) -> StdResult<Decimal256>;
}

impl Calculate for ExchangeRate {
    const A: Decimal256 = Decimal256(U256([1_000_499_635_890_955_755u64, 0, 0, 0]));
    const B: Decimal256 = Decimal256(U256([1_000_261_157_876_067_855u64, 0, 0, 0]));
    const LN_A: Decimal256 = Decimal256(U256([0_000_499_511_114_504_063u64, 0, 0, 0]));

    fn a_terra_exchange_rate(day: Decimal256) -> StdResult<Decimal256> {
        // https://www.omnicalculator.com/statistics/exponential-regression
        // exponential fit for points (0,1), (365,1.2), (730,1.44)
        // does not matter really as it will depend on market and will be fetched on chain
        Ok(ExchangeRate::A.powf(day)?)
    }

    fn invert_a_terra_exchange_rate(exchange_rate: Decimal256) -> StdResult<Decimal256> {
        Ok(Decimal256::ln(exchange_rate)? / ExchangeRate::LN_A)
    }

    fn capapult_exchange_rate(a_terra_exchange_rate: Decimal256) -> StdResult<Decimal256> {
        // https://www.omnicalculator.com/statistics/exponential-regression
        // exponential fit for points (0,1), (365,1.1), (730,1.21)
        // does not matter really as it will depend on market and will be fetched on chain
        let day: Decimal256 = ExchangeRate::invert_a_terra_exchange_rate(a_terra_exchange_rate)?;
        Ok(ExchangeRate::B.powf(day)?)
    }
}