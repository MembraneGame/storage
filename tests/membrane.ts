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
  getAirdrop,
  sendToken
} from './utils/web3';
import { expect } from 'chai';
import {
  calculateInitialRewardParams,
  calculatePlayerPayout,
  generateRandomGameResult,
  initializeMint
} from './utils/mocks';
import {
  MAX_PLAYER_SIZE, MAX_SIZE_REWARD,
  PLASMA_DECIMALS,
  PLASMA_INITIAL_SUPPLY
} from './utils/constants';

describe('Membrane', () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.Membrane as Program<Membrane>;
  const anchorProvider = program.provider as anchor.AnchorProvider;

  const systemProgram = anchor.web3.SystemProgram.programId;
  const tokenProgram = spl.TOKEN_PROGRAM_ID;

  let storage: Keypair;
  let reward: Keypair;
  let mintAddress: PublicKey;
  let storageTokenAddress: PublicKey;
  let player: Keypair;

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
        authority: storage.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID
      })
      .signers([storage])
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
    // Generate player account
    player = Keypair.generate();
    const user = anchorProvider.wallet;
    const rating = new anchor.BN(0);

    await program.methods
      .initializePlayer(rating)
      .accounts({
        player: player.publicKey,
        user: user.publicKey,
        systemProgram
      })
      .signers([player])
      .rpc();

    const playerAccount = await program.account.player.fetch(player.publicKey);

    expect(playerAccount.user.toBase58()).to.equal(user.publicKey.toBase58());
    expect(playerAccount.rating.toNumber()).to.equal(rating.toNumber());
  });

  it('Can make a single payout', async () => {
    const { placement, kills } = generateRandomGameResult();

    const playerAccountBefore = await program.account.player.fetch(
      player.publicKey
    );
    const storageTokenBalanceBefore =
      await anchorProvider.connection.getTokenAccountBalance(
        storageTokenAddress
      );
    const playerTokenAccount = await getOrCreateAssociatedTokenAccount(
      anchorProvider.connection,
      storage,
      mintAddress,
      player.publicKey
    );
    const playerTokenBalanceBefore =
      await anchorProvider.connection.getTokenAccountBalance(
        playerTokenAccount.address
      );

    await program.methods
      .payout(placement, kills)
      .accounts({
        reward: reward.publicKey,
        player: player.publicKey,
        sender: storage.publicKey,
        vaultToken: storageTokenAddress,
        playerToken: playerTokenAccount.address,
        mint: mintAddress,
        tokenProgram: TOKEN_PROGRAM_ID
      })
      .signers([storage])
      .rpc();

    const storageTokenBalanceAfter =
      await anchorProvider.connection.getTokenAccountBalance(
        storageTokenAddress
      );
    const playerTokenBalanceAfter =
      await anchorProvider.connection.getTokenAccountBalance(
        playerTokenAccount.address
      );
    const playerAccountAfter = await program.account.player.fetch(
      player.publicKey
    );

    const rewardMock = calculateInitialRewardParams(PLASMA_DECIMALS);
    const { rewardAmount, ratingChange } = calculatePlayerPayout(
      placement,
      kills,
      playerAccountBefore.rating,
      rewardMock
    );

    expect(
      new anchor.BN(storageTokenBalanceAfter.value.amount).eq(
        new anchor.BN(storageTokenBalanceBefore.value.amount).sub(rewardAmount)
      )
    ).to.be.true;
    expect(
      new anchor.BN(playerTokenBalanceAfter.value.amount).eq(
        new anchor.BN(playerTokenBalanceBefore.value.amount).add(rewardAmount)
      )
    ).to.be.true;

    const safeRatingChange = playerAccountBefore.rating
      .add(ratingChange)
      .lt(new anchor.BN(0))
      ? new anchor.BN(0)
      : ratingChange;

    expect(
      playerAccountBefore.rating.add(safeRatingChange).toNumber()
    ).to.be.equal(playerAccountAfter.rating.toNumber());
  });

  it('User can sell the token', async () => {
    const amountToSell = new anchor.BN(adjustSupply(10, PLASMA_DECIMALS));

    // Send some tokens to the player
    await sendToken(
      anchorProvider.connection,
      mintAddress,
      storage,
      player.publicKey,
      amountToSell.toNumber()
    );

    const storageTokenBalanceBefore =
      await anchorProvider.connection.getTokenAccountBalance(
        storageTokenAddress
      );
    const playerTokenAddress = await findAssociatedTokenAddress(
      player.publicKey,
      mintAddress
    );
    const playerTokenBalanceBefore =
      await anchorProvider.connection.getTokenAccountBalance(
        playerTokenAddress
      );

    await program.methods
      .userSell(amountToSell)
      .accounts({
        player: player.publicKey,
        mint: mintAddress,
        vaultToken: storageTokenAddress,
        playerToken: playerTokenAddress,
        tokenProgram: TOKEN_PROGRAM_ID,
        authority: storage.publicKey
      })
      .signers([player, storage])
      .rpc();

    const storageTokenBalanceAfter =
      await anchorProvider.connection.getTokenAccountBalance(
        storageTokenAddress
      );
    const playerTokenBalanceAfter =
      await anchorProvider.connection.getTokenAccountBalance(
        playerTokenAddress
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
});
