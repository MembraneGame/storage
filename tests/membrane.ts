import * as anchor from '@project-serum/anchor';
import * as spl from '@solana/spl-token';
import { Program } from '@project-serum/anchor';
import { Keypair, PublicKey } from '@solana/web3.js';
import { Membrane } from '../target/types/membrane';
import {
  adjustSupply,
  findAssociatedTokenAddress,
  getAirdrop
} from './utils/web3';
import { expect } from 'chai';
import {calculateInitialRewardParams, calculatePlayerPayout, initializeMint} from './utils/mocks';
import {
  getAccount,
  getMint,
  getOrCreateAssociatedTokenAccount,
  TOKEN_PROGRAM_ID
} from '@solana/spl-token';
import { PLASMA_DECIMALS, PLASMA_INITIAL_SUPPLY } from './utils/constants';

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
    await getAirdrop(anchorProvider.connection, storage.publicKey);
    // Initialize token mint
    const mintResult = await initializeMint(anchorProvider.connection, storage);

    mintAddress = mintResult.mintAddress;
    storageTokenAddress = mintResult.associatedTokenAddress;
    // Generate reward
    reward = Keypair.generate();
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
    expect(parseFloat(tokenBalance.value.uiAmountString)).to.equal(PLASMA_INITIAL_SUPPLY);
    expect(tokenAccount.mint.toBase58()).to.equal(mintAddress.toBase58());
    expect(tokenAccount.owner.toBase58()).to.equal(
      storage.publicKey.toBase58()
    );
    expect(tokenAccount.amount).to.equal(
      BigInt(adjustSupply(PLASMA_INITIAL_SUPPLY, PLASMA_DECIMALS))
    );
  });

  it('Can initialize a reward', async () => {
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
    player = Keypair.generate();
    const user = anchorProvider.wallet;

    await program.methods
      .initializePlayer()
      .accounts({
        player: player.publicKey,
        user: user.publicKey,
        systemProgram
      })
      .signers([player])
      .rpc();

    const playerAccount = await program.account.player.fetch(player.publicKey);

    expect(playerAccount.user.toBase58()).to.equal(user.publicKey.toBase58());
    expect(playerAccount.rating.toNumber()).to.equal(0);
  });

  it('Can make a single payout', async () => {
    const placement = new anchor.BN(15);
    const kills = new anchor.BN(1);

    const rewardAccount = await program.account.reward.fetch(reward.publicKey);
    const playerAccountBefore = await program.account.player.fetch(player.publicKey);

    const playerTokenAccount = await getOrCreateAssociatedTokenAccount(
      anchorProvider.connection,
      storage,
      mintAddress,
      player.publicKey
    );

    console.log('playerTokenAccount', playerTokenAccount);

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

    const paidPlayerTokenAccount = await getAccount(
      anchorProvider.connection,
      playerTokenAccount.address
    );

    console.log('paidPlayerTokenAccount', paidPlayerTokenAccount);

    const storageTokenAccount = await getAccount(
      anchorProvider.connection,
      storageTokenAddress
    );

    console.log('storageTokenAccount', storageTokenAccount);

    console.log('rewardAccount', rewardAccount);

    const storageTokenBalance = await anchorProvider.connection.getTokenAccountBalance(
      storageTokenAddress
    );

    console.log('storageTokenBalance', storageTokenBalance);

    const playerTokenBalance = await anchorProvider.connection.getTokenAccountBalance(
      playerTokenAccount.address
    );

    console.log('playerTokenBalance', playerTokenBalance);

    const playerAccountAfter = await program.account.player.fetch(player.publicKey);

    console.log('playerAccountAfter', playerAccountAfter);

    calculatePlayerPayout(placement, kills, playerAccountBefore.rating, rewardAccount);
  });
});
