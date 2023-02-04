#[allow(dead_code)]
#[allow(unused_variables)]

// Program keys
#[cfg(feature = "devnet")]
pub const PSYLEND_PROGRAM_KEY: &str = "8bpiM4yhcLYMSeCBTVFWisneXPQQWPYSA5ZpMm4DKAgT";
#[cfg(not(feature = "devnet"))]
pub const PSYLEND_PROGRAM_KEY: &str = "PLENDj46Y4hhqitNV2WqLqGLrWKAaH2xJHm2UyHgJLY";

// Primary borrow/lending market
#[cfg(feature = "devnet")]
pub const MAIN_MARKET_KEY: &str = "5QkMERuZEUP4XPP598z2dNPSZeupR8cNi3kyzvs6mbSb";
#[cfg(not(feature = "devnet"))]
pub const MAIN_MARKET_KEY: &str = "6b2oWJP6NdsLFsY8YJqLKXShGwEtztdmrHeArdb3SCLa";

// Major reserve addresses

// Devnet reserves and PsyFi vault accounts
pub const DEVNET_USDC_RESERVE: &str = "jQhYCJ8S8z7ce1uYWuQeViHyq8kvGTeJFGBHnDh2QXT";
pub const DEVNET_SOL_RESERVE: &str = "DLaK2XDMgF5hGTQYKNtNgeMKA6JiXmeCauAR96TZmyFA";
pub const DEVNET_BTC_RESERVE: &str = "DK7PqQhKqHuW7euXWxA6YXm7APdvE8x6ckPm3mTfDqVc";
pub const DEVNET_BTC_CALL_RESERVE: &str = "FiR57aTPjXDGkr18WdcDGtPYQHdqbPdXocy3tSuwC2ee";
pub const DEVNET_BTC_CALL_PSYFI_VAULT: &str = "D5z4cXPrLQrSvf36tGWbcYpfFhUHbKeB4E7KnbPSqVU8";
pub const DEVNET_BTC_PUT_RESERVE: &str = "CsNg4zfpcJ1LaBvhApqKgLLtrsWZ6prbT1SQuafrMEUN";
pub const DEVNET_BTC_PUT_PSYFI_VAULT: &str = "4UtoTfSXEtjJgMJkajj87FFtFBu3XL463mPA266giZMm";

 // Mainnet reserves and Psyfi vault accounts
pub const MAINNET_USDC_RESERVE: &str = "4y7uK2mH5zYmkeUSyg5xtfV2UfZhP7VjNErYSobJmF1e";
pub const MAINNET_SOL_RESERVE: &str = "BD4xq53K6SWiJVKqA3HY6drEJkFXgofaiyrkNziGXakU";
pub const MAINNET_SOL_PUT_RESERVE: &str = "C1HMiWMiG5HFdJ34RNhP3D1ANLjc4Yns38UicSPDxBFV";
pub const MAINNET_SOL_PUT_PSYFI_VAULT: &str = "7L2TZGBpfB4uBfPqvicqvD3VJSXwLujhzkjUju8yPt5u";
pub const MAINNET_WETH_CALL_RESERVE: &str = "4dT4z89EYJNc3cPgpNPsrnih9Vi7Yg5Hom33ndiES8HM";
pub const MAINNET_WETH_CALL_PSYFI_VAULT: &str = "5vaPss2LvGQCrX7Zh5sRAnPD1BRJbp9MdLvQMwet4Bbr";
pub const MAINNET_SOL_CALL_RESERVE: &str = "FgyVA9xta2rSNVngThQ4q2NMkfuJbdqt7TZWTEtA6sd6";
pub const MAINNET_SOL_CALL_PSYFI_VAULT: &str = "8SNcNyD4FJTFxYRocxaiq47yC41EKNi9bSZ23XngM1M8";
pub const MAINNET_BTC_PUT_RESERVE: &str = "4HSKvUxac2XkyiP5FNQty92uEUgzZoQSLjM9w1dQJysC";
pub const MAINNET_BTC_PUT_PSYFI_VAULT: &str = "792ELQdQ6nZBSBrKjRiJXxH1ZFpNjHktnpMdp2auyXMF";
