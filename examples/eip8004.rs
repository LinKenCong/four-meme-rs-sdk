mod common;

#[tokio::main]
async fn main() -> four_meme_sdk::Result<()> {
    let read_only_sdk = common::build_read_only_sdk()?;
    let owner = common::example_owner_address()?;
    let balance = read_only_sdk.eip8004_balance(owner).await?;
    println!("agent nft balance: {balance}");

    let Some(private_key) = common::example_private_key() else {
        common::skip_write_example("eip8004 register");
        return Ok(());
    };

    let fork_sdk = common::build_local_fork_sdk()?;
    let registration = fork_sdk
        .register_agent(
            private_key,
            "Example Agent",
            "https://example.com/agent.png",
            "Compile-checked local fork registration example.",
        )
        .await?;

    println!("local fork registration tx: {}", registration.tx_hash);
    println!("agent uri: {}", registration.agent_uri);

    Ok(())
}
