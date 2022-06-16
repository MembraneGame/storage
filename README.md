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



```

run `anchor build` before deploying to make sure I’m deploying the latest version of my code.
