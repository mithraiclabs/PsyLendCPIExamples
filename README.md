<div align="center">
  <img height="40" src="PsyDoLogo.png" />

  <h1>PsyLend Protocol CPI Examples (for Integrators)</h1>

  <h4>
    <a href="https://www.psyoptions.io">Website</a>
  </h4>
</div>

Avoid scammers: PsyLend does not have a public crate. The CPI crate (this code) is available <a
href="https://crates.io/crates/psylend-cpi">here</a>

## Developers

See the complete guide to common terms, states, structs, and instructions in our <a
href="Architecture.md">Architecture Docs</a>. Still have questions? Try our <a href="https://discord.gg/MgDdJKgZJc">Discord</a>.

## Rust Integrators

Check out common CPI examples in this repo, and see example front-end usage in the corresponding test suite.

## Front End Developers and Liquidators

Want to integrate with PsyLend? Looking for Typescript utilities to interact with the program? Check
out our npm package: <a
href="https://www.npmjs.com/package/@mithraic-labs/psylend-utils">psylend-utils</a>

## White Hats and Bug Bounty Hunters

See our <a href="https://docs.psyoptions.io/psy-dao-bug-bounty">bug bounty policy</a>. Note that the PsyLend
protocol is in scope, however the CPI library is not, as this is only a set of examples. Feel free
to use the CPI library to look for vulnerabilities in PsyLend.

## Running Tests
Tests run on devnet using pre-made PsyLend markets and reserves. Use `anchor test -- --features
devnet`. Your wallet must have some SOL and USDC for all tests to pass, you can get these at <a
href="https://trade.psyoptions.io/#/faucets">our faucet</a>