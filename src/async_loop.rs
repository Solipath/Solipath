use std::future::Future;

use futures::future::join_all;

pub async fn run_async<'a, INPUT, FUTURE, RETURN, FUNCTION>(inputs: &'a [INPUT], function: FUNCTION) -> Vec<RETURN>
where
    FUTURE: Future<Output = RETURN>,
    FUNCTION: Fn(&'a INPUT) -> FUTURE,
{
    let async_function_list = inputs.iter().map(|input| function(input));
    join_all(async_function_list).await
}