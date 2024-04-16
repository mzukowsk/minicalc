use std::collections::HashMap;
use rust_decimal::Decimal;
use rust_decimal::MathematicalOps;

use crate::sheet::FuncDef;

macro_rules! function {
    ($constant_name: ident, $function_name: literal, $required_params: expr, $function_body: expr) => {
        const $constant_name: FuncDef = |params| {
            match $required_params {
                None => {
                    if !params.is_empty() {
                        #[allow(clippy::redundant_closure_call)]
                        $function_body(params)
                    } else {
                        Err(format!("No params for {}", $function_name))
                    }
                },
                Some(params_count) => {
                    if params.len() == params_count {
                        #[allow(clippy::redundant_closure_call)]
                        $function_body(params)
                    } else {
                        Err(format!("{} expected {} parameter{}, got {}",
                            $function_name,
                            params_count,
                            if params_count == 1 { "" } else { "s" },
                            params.len()))
                    }
                },
            }
        };
    };
}

function!(FN_MAX, "max", None::<usize>, |params: Vec<Decimal>| {
    Ok(params.into_iter().max().unwrap())
});

function!(FN_MIN, "min", None::<usize>, |params: Vec<Decimal>| {
    Ok(params.into_iter().min().unwrap())
});

function!(FN_PI, "pi", Some(0), |_| {
    Ok(Decimal::PI)
});

function!(FN_SQRT, "sqrt", Some(1), |params: Vec<Decimal>| {
    let param = params[0];
    param.sqrt().ok_or_else(|| format!("Error applying sqrt to {}", param))
});

function!(FN_POW, "pow", Some(2), |params: Vec<Decimal>| {
    let base = params[0];
    let exp = params[1];
    Ok(base.powd(exp))
});


macro_rules! functions_hashmap {
    ($( $key: literal => $val: expr ),* $(,)? ) => {{
         let mut map = HashMap::new();
         $( map.insert($key.to_ascii_lowercase(), $val); )*
         map
    }}
}

pub fn functions() -> HashMap<String, FuncDef> {
    functions_hashmap!(
        "max" => FN_MAX,
        "min" => FN_MIN,
        "pi" => FN_PI,
        "sqrt" => FN_SQRT,
        "pow" => FN_POW,
    )
}
