#[macro_export]
macro_rules! react_positively {
    ($msg:expr, $ctx_http:expr) => {
        $msg.react($ctx_http, ReactionType::Unicode("✅".to_string()))
            .await
            .unwrap();
    };
}

#[macro_export]
macro_rules! react_negatively {
    ($msg:expr, $ctx_http:expr) => {
        $msg.react($ctx_http, ReactionType::Unicode("❌".to_string()))
            .await
            .unwrap();
    };
}
