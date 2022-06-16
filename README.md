## If system does not see cargo or solana

```
export PATH="$HOME/.cargo/bin:$PATH"
export PATH="$HOME/.local/share/solana/install/active_release/bin:$PATH"
```

## Build

```
anchor build
```

## Test

```
# Make sure you’re on the localnet.
solana config set --url localhost

# And check your Anchor.toml file.

anchor test
```

# Deploying to devnet

## Changing the cluster

```
solana config set --url devnet

# Outputs:
# Config File: /Users/viktor/.config/solana/cli/config.yml
# RPC URL: https://api.devnet.solana.com
# WebSocket URL: wss://api.devnet.solana.com/ (computed)
# Keypair Path: /Users/viktor/.config/solana/id.json
# Commitment: confirmed
```

```
anchor build
anchor deploy


Outputs:
To deploy this program:
  $ solana program deploy /Users/viktor/Rust/solana-twitter/target/deploy/solana_twitter.so
The program address will default to this keypair (override with --program-id):
  /Users/viktor/Rust/solana-twitter/target/deploy/solana_twitter-keypair.json
(base) cod005hctln:solana-twitter viktor$ anchor deploy
Deploying workspace: https://api.devnet.solana.com
Upgrade authority: /Users/viktor/.config/solana/id.json
Deploying program "solana-twitter"...
Program path: /Users/viktor/Rust/solana-twitter/target/deploy/solana_twitter.so...
Program Id: 3rUnayciVyLhdoudmFxS55M213ykYjLP6PZvUtTstHKJ
```

run `anchor build` before deploying to make sure I’m deploying the latest version of my code.
