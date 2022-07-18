import * as anchor from '@project-serum/anchor';
import { Program } from '@project-serum/anchor';
import * as spl from '@solana/spl-token';
import {
  getAccount,
  getMint,
  getOrCreateAssociatedTokenAccount,
  TOKEN_PROGRAM_ID
} from '@solana/spl-token';
import { Keypair, LAMPORTS_PER_SOL, PublicKey } from '@solana/web3.js';
import { Membrane } from '../target/types/membrane';
import {
  adjustSupply,
  findAssociatedTokenAddress,
  getAirdrop
} from './utils/web3';
import { expect } from 'chai';
import {
  calculateInitialRewardParams,
  calculatePlayerPayout,
  generateRandomGameResult,
  initializeMint
} from './utils/mocks';
import {
  FEE_LAMPORTS,
  MAX_PLAYER_SIZE,
  MAX_SIZE_REWARD,
  PLASMA_DECIMALS,
  PLASMA_INITIAL_SUPPLY,
  VAULT_PDA_SEED
} from './utils/constants';

describe('Membrane', () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.Membrane as Program<Membrane>;
  const anchorProvider = program.provider as anchor.AnchorProvider;

  const systemProgram = anchor.web3.SystemProgram.programId;
  const tokenProgram = spl.TOKEN_PROGRAM_ID;

  let storage: Keypair;
  let storagePDA: PublicKey; // storage account PDA
  let reward: Keypair;
  let mintAddress: PublicKey;
  let storageTokenAddress: PublicKey;
  let player: PublicKey;
  let playerPDA: PublicKey; // player account PDA
  let playerBump: number;

  before(async () => {
    // TODO: create first initialize script
    // Initialize storage
    // admin = authority = storage
    storage = Keypair.generate();
    // Airdrop storage
    await getAirdrop(anchorProvider.connection, storage.publicKey, 10);
    // Initialize token mint
    const mintResult = await initializeMint(anchorProvider.connection, storage);

    mintAddress = mintResult.mintAddress;
    storageTokenAddress = mintResult.associatedTokenAddress;

    // Get storage account PDA
    const [_storagePDA, _storageBump] =
      await anchor.web3.PublicKey.findProgramAddress(
        [Buffer.from(VAULT_PDA_SEED)],
        program.programId
      );

    storagePDA = _storagePDA;

    // Generate player account PDA
    const [_playerPDA, _playerBump] =
      await anchor.web3.PublicKey.findProgramAddress(
        [Buffer.from('player'), anchorProvider.wallet.publicKey.toBuffer()],
        program.programId
      );

    playerPDA = _playerPDA;
    playerBump = _playerBump;

    // Estimate rent exemption for accounts (in SOL)
    const playerAccountMaxRent =
      await anchorProvider.connection.getMinimumBalanceForRentExemption(
        MAX_PLAYER_SIZE
      );
    const rewardAccountMaxRent =
      await anchorProvider.connection.getMinimumBalanceForRentExemption(
        MAX_SIZE_REWARD
      );
    console.log({
      MAX_PLAYER_SIZE,
      MAX_SIZE_REWARD,
      playerAccountMaxRent: playerAccountMaxRent / LAMPORTS_PER_SOL,
      rewardAccountMaxRent: rewardAccountMaxRent / LAMPORTS_PER_SOL
    });
  });

  it('Can initialize mint', async () => {
    const mintInfo = await getMint(anchorProvider.connection, mintAddress);

    const associatedTokenAddress = await findAssociatedTokenAddress(
      storage.publicKey,
      mintAddress
    );

    const tokenBalance = await anchorProvider.connection.getTokenAccountBalance(
      associatedTokenAddress
    );

    const tokenAccount = await getAccount(
      anchorProvider.connection,
      associatedTokenAddress
    );

    expect(mintInfo.decimals).to.equal(PLASMA_DECIMALS);
    expect(mintInfo.mintAuthority.toBase58()).to.equal(
      storage.publicKey.toBase58()
    );
    expect(associatedTokenAddress.toBase58()).to.equal(
      storageTokenAddress.toBase58()
    );
    expect(tokenBalance.value.decimals).to.equal(PLASMA_DECIMALS);
    expect(parseFloat(tokenBalance.value.uiAmountString)).to.equal(
      PLASMA_INITIAL_SUPPLY
    );
    expect(tokenAccount.mint.toBase58()).to.equal(mintAddress.toBase58());
    expect(tokenAccount.owner.toBase58()).to.equal(
      storage.publicKey.toBase58()
    );
    expect(tokenAccount.amount).to.equal(
      BigInt(adjustSupply(PLASMA_INITIAL_SUPPLY, PLASMA_DECIMALS))
    );
  });

  it('Can transfer authority to the PDA', async () => {
    const mintInfoBefore = await getMint(
      anchorProvider.connection,
      mintAddress
    );
    const tokenAccountBefore = await getAccount(
      anchorProvider.connection,
      storageTokenAddress
    );

    expect(mintInfoBefore.mintAuthority.toBase58()).to.equal(
      storage.publicKey.toBase58()
    );
    expect(tokenAccountBefore.owner.toBase58()).to.equal(
      storage.publicKey.toBase58()
    );

    await program.methods
      .transferAuthority()
      .accounts({
        storage: storage.publicKey,
        storageTokenAccount: storageTokenAddress,
        mint: mintAddress,
        tokenProgram: TOKEN_PROGRAM_ID
      })
      .signers([storage])
      .rpc();

    const mintInfoAfter = await getMint(anchorProvider.connection, mintAddress);
    const tokenAccountAfter = await getAccount(
      anchorProvider.connection,
      storageTokenAddress
    );

    expect(mintInfoAfter.mintAuthority.toBase58()).to.equal(
      storagePDA.toBase58()
    );
    expect(tokenAccountAfter.owner.toBase58()).to.equal(storagePDA.toBase58());
  });

  it('Can mint a token', async () => {
    const amountToMint = new anchor.BN(adjustSupply(1000, PLASMA_DECIMALS));

    const storageTokenBalanceBefore =
      await anchorProvider.connection.getTokenAccountBalance(
        storageTokenAddress
      );

    await program.methods
      .mintToken(amountToMint)
      .accounts({
        mint: mintAddress,
        tokenAccount: storageTokenAddress,
        authority: storagePDA,
        tokenProgram: TOKEN_PROGRAM_ID
      })
      .signers([])
      .rpc();

    const storageTokenBalanceAfter =
      await anchorProvider.connection.getTokenAccountBalance(
        storageTokenAddress
      );

    expect(
      new anchor.BN(storageTokenBalanceBefore.value.amount)
        .add(amountToMint)
        .eq(new anchor.BN(storageTokenBalanceAfter.value.amount))
    ).to.be.true;
  });

  it('Can initialize a reward', async () => {
    // Generate reward account
    reward = Keypair.generate();

    await program.methods
      .initializeReward()
      .accounts({
        reward: reward.publicKey,
        payer: storage.publicKey,
        systemProgram
      })
      .signers([storage, reward])
      .rpc();

    // Initial reward
    const rewardAccount = await program.account.reward.fetch(reward.publicKey);

    // Example result
    // {
    //   victory: 8926864892,
    //   topFive: 2231716223,
    //   topTen: 892686489,
    //   kill: 1249761085
    // }

    const rewardMock = calculateInitialRewardParams(PLASMA_DECIMALS);

    for (const key in rewardMock) {
      const result = rewardAccount[key];
      const mock = rewardMock[key];
      expect(result.eq(mock)).to.be.true;
    }
  });

  it('Can initialize a player', async () => {
    const user = anchorProvider.wallet;
    const rating = new anchor.BN(0);

    const storagePDABalanceBefore =
      await anchorProvider.connection.getAccountInfo(storagePDA);

    await program.methods
      .initializePlayer(rating)
      .accounts({
        player: playerPDA,
        authority: storagePDA,
        user: user.publicKey,
        systemProgram
      })
      .signers([])
      .rpc();

    const playerAccount = await program.account.player.fetch(playerPDA);
    const storagePDABalanceAfter =
      await anchorProvider.connection.getAccountInfo(storagePDA);

    expect(playerAccount.identity.toBase58()).to.equal(
      user.publicKey.toBase58()
    );
    expect(playerAccount.rating.toNumber()).to.equal(rating.toNumber());

    const lamportsBefore = storagePDABalanceBefore?.lamports || 0;
    const lamportsAfter = storagePDABalanceAfter?.lamports || 0;

    expect(lamportsBefore + FEE_LAMPORTS).to.equal(lamportsAfter);
  });

  it.skip('Can make a single payout', async () => {
    const { placement, kills } = generateRandomGameResult();

    const playerAccountBefore = await program.account.player.fetch(playerPDA);

    await program.methods
      .calculateReward(placement, kills, playerBump, new anchor.BN(0))
      .accounts({
        reward: reward.publicKey,
        player: playerPDA
      })
      .signers([])
      .rpc();

    const playerAccountAfter = await program.account.player.fetch(playerPDA);

    const rewardMock = calculateInitialRewardParams(PLASMA_DECIMALS);
    const { rewardAmount, ratingChange } = calculatePlayerPayout(
      placement,
      kills,
      playerAccountBefore.rating,
      rewardMock
    );

    const safeRatingChange = playerAccountBefore.rating
      .add(ratingChange)
      .lt(new anchor.BN(0))
      ? new anchor.BN(0)
      : ratingChange;

    expect(
      playerAccountBefore.claimable
        .add(rewardAmount)
        .eq(playerAccountAfter.claimable)
    ).to.be.true;
    expect(
      playerAccountBefore.rating.add(safeRatingChange).toNumber()
    ).to.be.equal(playerAccountAfter.rating.toNumber());
  });

  it('User can claim a reward', async () => {
    const user = anchorProvider.wallet;
    const playerAccountBefore = await program.account.player.fetch(playerPDA);

    const storageTokenBalanceBefore =
      await anchorProvider.connection.getTokenAccountBalance(
        storageTokenAddress
      );
    const userTokenAccount = await getOrCreateAssociatedTokenAccount(
      anchorProvider.connection,
      storage,
      mintAddress,
      user.publicKey
    );
    const userTokenBalanceBefore =
      await anchorProvider.connection.getTokenAccountBalance(
        userTokenAccount.address
      );

    await program.methods
      .userClaim()
      .accounts({
        player: playerPDA,
        user: user.publicKey,
        authority: storagePDA,
        vaultToken: storageTokenAddress,
        playerToken: userTokenAccount.address,
        mint: mintAddress,
        tokenProgram: TOKEN_PROGRAM_ID
      })
      .signers([]) // anchor will set user as a signer by default
      .rpc();

    const storageTokenBalanceAfter =
      await anchorProvider.connection.getTokenAccountBalance(
        storageTokenAddress
      );
    const userTokenBalanceAfter =
      await anchorProvider.connection.getTokenAccountBalance(
        userTokenAccount.address
      );
    const playerAccountAfter = await program.account.player.fetch(playerPDA);

    expect(
      new anchor.BN(storageTokenBalanceAfter.value.amount).eq(
        new anchor.BN(storageTokenBalanceBefore.value.amount).sub(
          playerAccountBefore.claimable
        )
      )
    ).to.be.true;
    expect(
      new anchor.BN(userTokenBalanceAfter.value.amount).eq(
        new anchor.BN(userTokenBalanceBefore.value.amount).add(
          playerAccountBefore.claimable
        )
      )
    ).to.be.true;
    expect(playerAccountAfter.claimable.eq(new anchor.BN(0))).to.be.true;
  });

  it('User can sell the token', async () => {
    const user = anchorProvider.wallet;
    const amountToSell = new anchor.BN(adjustSupply(10, PLASMA_DECIMALS));

    // Mint some tokens to the user

    // Get the token account of the toWallet address, and if it does not exist, create it
    const userTokenAccount = await getOrCreateAssociatedTokenAccount(
      anchorProvider.connection,
      storage,
      mintAddress,
      user.publicKey
    );

    // Make mint
    await program.methods
      .mintToken(amountToSell)
      .accounts({
        mint: mintAddress,
        tokenAccount: userTokenAccount.address,
        authority: storagePDA,
        tokenProgram: TOKEN_PROGRAM_ID
      })
      .signers([])
      .rpc();

    const storageTokenBalanceBefore =
      await anchorProvider.connection.getTokenAccountBalance(
        storageTokenAddress
      );
    const playerTokenBalanceBefore =
      await anchorProvider.connection.getTokenAccountBalance(
        userTokenAccount.address
      );

    await program.methods
      .userSell(amountToSell)
      .accounts({
        player: user.publicKey,
        mint: mintAddress,
        vaultToken: storageTokenAddress,
        playerToken: userTokenAccount.address,
        tokenProgram: TOKEN_PROGRAM_ID,
        authority: storagePDA
      })
      .signers([])
      .rpc();

    const storageTokenBalanceAfter =
      await anchorProvider.connection.getTokenAccountBalance(
        storageTokenAddress
      );
    const playerTokenBalanceAfter =
      await anchorProvider.connection.getTokenAccountBalance(
        userTokenAccount.address
      );

    expect(
      new anchor.BN(storageTokenBalanceAfter.value.amount).eq(
        new anchor.BN(storageTokenBalanceBefore.value.amount).add(
          // Half of the amount is burned
          amountToSell.div(new anchor.BN(2))
        )
      )
    ).to.be.true;
    expect(
      new anchor.BN(playerTokenBalanceAfter.value.amount).eq(
        new anchor.BN(playerTokenBalanceBefore.value.amount).sub(amountToSell)
      )
    ).to.be.true;
  });

  it('Can freeze the authority for mint', async () => {
    const tokenAccountBefore = await getAccount(
      anchorProvider.connection,
      storageTokenAddress
    );

    expect(tokenAccountBefore.isFrozen).to.be.false;

    await program.methods
      .freezeStorage()
      .accounts({
        mint: mintAddress,
        // TODO: INCONSISTENCY rename "storage" account to the storageTokenAccount
        storage: storageTokenAddress,
        authority: storagePDA,
        tokenProgram: TOKEN_PROGRAM_ID
      })
      .signers([])
      .rpc();

    const tokenAccountAfter = await getAccount(
      anchorProvider.connection,
      storageTokenAddress
    );

    expect(tokenAccountAfter.isFrozen).to.be.true;
  });

  it.skip('Can return the authority back to the storage', async () => {
    const mintInfoBefore = await getMint(
      anchorProvider.connection,
      mintAddress
    );
    const tokenAccountBefore = await getAccount(
      anchorProvider.connection,
      storageTokenAddress
    );

    // const storagePDABalanceBefore =
    //   await anchorProvider.connection.getAccountInfo(storagePDA);

    expect(mintInfoBefore.mintAuthority.toBase58()).to.equal(
      storagePDA.toBase58()
    );
    expect(tokenAccountBefore.owner.toBase58()).to.equal(storagePDA.toBase58());

    await program.methods
      .returnAuthority()
      .accounts({
        storage: storage.publicKey,
        mint: mintAddress,
        storageTokenAccount: storageTokenAddress,
        pda: storagePDA,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram
      })
      .signers([])
      .rpc();

    const mintInfoAfter = await getMint(anchorProvider.connection, mintAddress);
    const tokenAccountAfter = await getAccount(
      anchorProvider.connection,
      storageTokenAddress
    );

    expect(mintInfoAfter.mintAuthority.toBase58()).to.equal(
      storage.publicKey.toBase58()
    );
    expect(tokenAccountAfter.owner.toBase58()).to.equal(
      storage.publicKey.toBase58()
    );
  });
});
