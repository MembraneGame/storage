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


