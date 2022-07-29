import * as anchor from '@project-serum/anchor';
import { Program } from '@project-serum/anchor';
import * as spl from '@solana/spl-token';
import {
  getAccount,
  getMint,
  getOrCreateAssociatedTokenAccount,
  TOKEN_PROGRAM_ID
} from '@solana/spl-token';
import { Keypair, PublicKey, SystemProgram } from '@solana/web3.js';
import { Membrane } from '../target/types/membrane';
import {
  adjustSupply,
  findAssociatedTokenAddress,
  getAirdrop
} from './utils/web3';
import { expect } from 'chai';
import {
  calculateInitialRewardParams,
  generateRandomGameResult,
  initializeMint,
  updateNftMultiplier
} from './utils/mocks';
import {
  AVG_STATS_SAMPLE,
  DEFAULT_NFT_MULTIPLIER,
  FEE_LAMPORTS,
  NFT_STATS_SAMPLE,
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
  let nftMultiplier: Keypair;
  let mintAddress: PublicKey;
  let storageTokenAddress: PublicKey;
  let player: PublicKey;
  let playerPDA: PublicKey; // player account PDA
  let playersStats: Keypair;
  let playerBump: number;
  let gamePDA: PublicKey;
  let gameBump: number;
  const identifier: anchor.BN = new anchor.BN(0);

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

    // Generate game account PDA
    const [_gamePDA, _gameBump] =
      await anchor.web3.PublicKey.findProgramAddress(
        [Buffer.from('game'), Buffer.from(identifier.toString())],
        program.programId
      );

    gamePDA = _gamePDA;
    gameBump = _gameBump;

    // Estimate rent exemption for accounts (in SOL)
    // const playerAccountMaxRent =
    //   await anchorProvider.connection.getMinimumBalanceForRentExemption(
    //     MAX_PLAYER_SIZE
    //   );
    // const rewardAccountMaxRent =
    //   await anchorProvider.connection.getMinimumBalanceForRentExemption(
    //     MAX_SIZE_REWARD
    //   );
    // console.log({
    //   MAX_PLAYER_SIZE,
    //   MAX_SIZE_REWARD,
    //   playerAccountMaxRent: playerAccountMaxRent / LAMPORTS_PER_SOL,
    //   rewardAccountMaxRent: rewardAccountMaxRent / LAMPORTS_PER_SOL
    // });
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

  it('Can initialize the NFT multiplier', async () => {
    // Generate nftMultiplier account
    nftMultiplier = Keypair.generate();

    await program.methods
      .initializeNftMultiplier()
      .accounts({
        nftMultiplier: nftMultiplier.publicKey,
        payer: storage.publicKey,
        systemProgram
      })
      .signers([storage, nftMultiplier])
      .rpc();

    const nftMultiplierAccountBeforeUpdate =
      await program.account.qualityMultiplier.fetch(nftMultiplier.publicKey);

    expect(nftMultiplierAccountBeforeUpdate).to.be.an('object');

    for (const key in DEFAULT_NFT_MULTIPLIER) {
      const multiplier = nftMultiplierAccountBeforeUpdate[key];
      const mock = DEFAULT_NFT_MULTIPLIER[key];
      expect(multiplier?.toNumber()).to.be.equal(mock);
    }
  });

  it('Can update the NFT multiplier', async () => {
    await program.methods
      .updateNftMultiplier(
        // @ts-ignore
        AVG_STATS_SAMPLE,
        NFT_STATS_SAMPLE
      )
      .accounts({
        nftMultiplier: nftMultiplier.publicKey,
        storage: storage.publicKey
      })
      .signers([storage])
      .rpc();

    const nftMultiplierAccountAfterUpdate =
      await program.account.qualityMultiplier.fetch(nftMultiplier.publicKey);

    const nftMultiplierMock = updateNftMultiplier(
      AVG_STATS_SAMPLE,
      NFT_STATS_SAMPLE
    );

    for (const key in nftMultiplierMock) {
      const multiplier = nftMultiplierAccountAfterUpdate[key];
      const mock = nftMultiplierMock[key];
      expect(multiplier?.eq(mock)).to.be.true;
    }
  });

  it('Can initialize a reward', async () => {
    // Generate reward account
    reward = Keypair.generate();

    await program.methods
      .initializeReward()
      .accounts({
        reward: reward.publicKey,
        nftMultiplier: nftMultiplier.publicKey,
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

    const nftMultiplierAccount = await program.account.qualityMultiplier.fetch(
      nftMultiplier.publicKey
    );

    const rewardMock = calculateInitialRewardParams(
      nftMultiplierAccount.common.toNumber(),
      PLASMA_DECIMALS
    );

    for (const key in rewardMock) {
      const result = rewardAccount[key];
      const mock = rewardMock[key];
      expect(result.eq(mock)).to.be.true;
    }
  });

  it('Can initialize a player', async () => {
    const user = anchorProvider.wallet;
    const rating = new anchor.BN(0);

    const storageBalanceBefore = await anchorProvider.connection.getAccountInfo(
      storage.publicKey
    );

    await program.methods
      .initializePlayer(rating)
      .accounts({
        player: playerPDA,
        authority: storagePDA,
        storage: storage.publicKey,
        user: user.publicKey,
        systemProgram
      })
      .signers([])
      .rpc();

    const playerAccount = await program.account.player.fetch(playerPDA);
    const storageBalanceAfter = await anchorProvider.connection.getAccountInfo(
      storage.publicKey
    );

    expect(playerAccount?.identity.toBase58()).to.equal(
      user.publicKey.toBase58()
    );
    expect(playerAccount?.rating.toNumber()).to.equal(rating.toNumber());

    const lamportsBefore = storageBalanceBefore?.lamports || 0;
    const lamportsAfter = storageBalanceAfter?.lamports || 0;

    expect(lamportsBefore + FEE_LAMPORTS).to.equal(lamportsAfter);
  });

  it.skip('Can start a game', async () => {
    // await program.methods
    //   .startGame(identifier)
    //   .accounts({
    //     game: gamePDA,
    //     storage: storage.publicKey,
    //     systemProgram
    //   })
    //   .signers([storage])
    //   .rpc();
    //
    // const gameAccount = await program.account.gameStart.fetch(gamePDA);
    //
    // expect(gameAccount?.timestamp.toNumber()).to.be.a('number');
  });

  it('Can make a single payout', async () => {
    const user = anchorProvider.wallet;
    const { placement, kills } = generateRandomGameResult();
    const { placement: placement1, kills: kills1 } = generateRandomGameResult();

    console.log({
      placement: placement.toNumber(),
      kills: kills.toNumber(),
      placement1: placement1.toNumber(),
      kills1: kills1.toNumber()
    });

    const accountSize = 1808; // Should be enough
    const lamportForRent =
      await anchorProvider.connection.getMinimumBalanceForRentExemption(
        accountSize
      );

    console.log({ accountSize, lamportForRent });

    playersStats = Keypair.generate();

    try {
      await program.methods
        .createPlayerStats()
        .accounts({
          playerStats: playersStats.publicKey
        })
        .preInstructions([
          // await program.account.playersStats.createInstruction(playersStats, accountSize)
          SystemProgram.createAccount({
            fromPubkey: storage.publicKey,
            newAccountPubkey: playersStats.publicKey,
            space: accountSize,
            lamports: lamportForRent,
            programId: program.programId
          })
        ])
        .signers([playersStats, storage])
        .rpc();
    } catch (e) {
      console.error(e);
    }

    const playersStatsAccount = await program.account.playersStats.fetch(
      playersStats.publicKey
    );

    // console.log(playersStatsAccount);

    const id = Keypair.generate();

    try {
      await program.methods
        .calculateReward(placement, kills, identifier)
        .accounts({
          reward: reward.publicKey,
          player: playerPDA,
          nftMultiplier: nftMultiplier.publicKey,
          storage: storage.publicKey,
          playersStats: playersStats.publicKey,
          systemProgram
        })
        .signers([storage])
        .rpc();
    } catch (e) {
      console.error(e);
    }

    try {
      await program.methods
        .calculateReward(placement1, kills1, identifier)
        .accounts({
          reward: reward.publicKey,
          player: playerPDA,
          nftMultiplier: nftMultiplier.publicKey,
          storage: storage.publicKey,
          playersStats: playersStats.publicKey,
          systemProgram
        })
        .signers([storage])
        .rpc();
    } catch (e) {
      console.error(e);
    }

    const playersStatsAccountAfter = await program.account.playersStats.fetch(
      playersStats.publicKey
    );

    console.log(user.publicKey.toBase58());
    console.log('------------ 1 -------------')
    console.log(playersStatsAccountAfter.players[0]);
    console.log(playersStatsAccountAfter.players[0].id.toBase58());
    console.log(playersStatsAccountAfter.players[0].reward.toString());
    console.log('------------ 2 -------------')
    console.log(playersStatsAccountAfter.players[1]);
    console.log(playersStatsAccountAfter.players[1].id.toBase58());
    console.log(playersStatsAccountAfter.players[1].reward.toString());
    console.log('------------ total -------------')
    console.log(playersStatsAccountAfter.counter.toString());

    // // Generate players stats account
    // playersStats = Keypair.generate();
    // const accountSize = 1560; // Should be enough
    // const lamportForRent = await anchorProvider.connection.getMinimumBalanceForRentExemption(
    //   accountSize
    // );
    //
    // console.log({ accountSize, lamportForRent });
    //
    // try {
    //   await program.methods
    //     .createPlayerStats()
    //     .accounts({
    //       playerStats: playersStats.publicKey
    //     })
    //     .preInstructions([
    //       // await program.account.playersStats.createInstruction(playersStats, accountSize)
    //       SystemProgram.createAccount({
    //         fromPubkey: storage.publicKey,
    //         newAccountPubkey: playersStats.publicKey,
    //         space: accountSize,
    //         lamports: lamportForRent,
    //         programId: program.programId
    //       })
    //     ])
    //     .signers([playersStats, storage])
    //     .rpc();
    // } catch (e) {
    //   console.error(e);
    // }
    //
    // const playersStatsAccount = await program.account.playersStats.fetch(
    //   playersStats.publicKey
    // );
    //
    // console.log(playersStats.publicKey.toBase58(), playersStatsAccount);
    //
    // // const playerAccountBefore = await program.account.player.fetch(playerPDA);
    //
    // try {
    //   await program.methods
    //     .calculateReward(placement, kills, identifier)
    //     .accounts({
    //       // reward: reward.publicKey,
    //       // player: playerPDA,
    //       // nftMultiplier: nftMultiplier.publicKey,
    //       storage: storage.publicKey,
    //       playersStats: playersStats.publicKey,
    //       systemProgram
    //     })
    //     .signers([storage])
    //     .rpc();
    // } catch (e) {
    //   console.error(e);
    // }
    //
    // const playerAccountAfter = await program.account.player.fetch(playerPDA);
    //
    // const playersStatsAccountAfter = await program.account.playersStats.fetch(
    //   playersStats.publicKey
    // );
    //
    // // console.log(user.publicKey.toBase58());
    // console.log(playersStats.publicKey.toBase58(), playersStatsAccountAfter);
    // console.log('playersStats [0]', playersStatsAccountAfter.players[0].reward.toString());

    // const nftMultiplierAccount = await program.account.qualityMultiplier.fetch(
    //   nftMultiplier.publicKey
    // );
    //
    // const rewardMock = calculateInitialRewardParams(
    //   nftMultiplierAccount.common.toNumber(),
    //   PLASMA_DECIMALS
    // );
    // const { rewardAmount, ratingChange } = calculatePlayerPayout(
    //   placement,
    //   kills,
    //   playerAccountBefore.rating,
    //   rewardMock
    // );
    //
    // const safeRatingChange = playerAccountBefore.rating
    //   .add(ratingChange)
    //   .lt(new anchor.BN(0))
    //   ? new anchor.BN(0)
    //   : ratingChange;

    // const stat = ((playersStatsAccountAfter?.players || []) as Stat[]).find(
    //   (stat) => {
    //     return stat.id.toBase58() === user.publicKey.toBase58();
    //   }
    // );

    // expect(playersStatsAccountAfter?.players).to.be.an('array');
    // expect(stat).to.be.an('object');
    // expect(stat?.placement).to.be.equal(placement.toNumber());
    // expect(stat?.kills).to.be.equal(kills.toNumber());
    // expect(
    //   stat?.reward.eq(
    //     playerAccountAfter.claimable.sub(playerAccountBefore.claimable)
    //   )
    // ).to.be.true;

    // expect(
    //   playerAccountBefore.claimable
    //     .add(rewardAmount)
    //     .eq(playerAccountAfter.claimable)
    // ).to.be.true;
    // expect(
    //   playerAccountBefore.rating.add(safeRatingChange).toNumber()
    // ).to.be.equal(playerAccountAfter.rating.toNumber());
  });

  it.skip('Can end the game', async () => {
    // Generate history account
    // const history = Keypair.generate();
    //
    // await program.methods
    //   .createHistoryAccount()
    //   .accounts({
    //     history: history.publicKey
    //   })
    //   .signers([storage])
    //   .rpc();
    //
    // await program.methods
    //   .endGame(identifier)
    //   .accounts({
    //     game: gamePDA,
    //     history: history.publicKey,
    //     playersStats: playersStats.publicKey,
    //     storage: storage.publicKey,
    //     systemProgram
    //   })
    //   .signers([storage])
    //   .rpc();
    //
    // const gameAccount = await program.account.gameStart.fetch(gamePDA);
    //
    // expect(gameAccount?.timestamp.toNumber()).to.be.a('number');
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

  it('Can return the authority back to the storage', async () => {
    const mintInfoBefore = await getMint(
      anchorProvider.connection,
      mintAddress
    );
    const tokenAccountBefore = await getAccount(
      anchorProvider.connection,
      storageTokenAddress
    );
    const storagePDABalanceBefore =
      await anchorProvider.connection.getAccountInfo(storagePDA);
    const storageBalanceBefore = await anchorProvider.connection.getAccountInfo(
      storage.publicKey
    );
    const storagePDALamportsBefore = storagePDABalanceBefore?.lamports || 0;
    const storageLamportsBefore = storageBalanceBefore?.lamports || 0;

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
    const storagePDABalanceAfter =
      await anchorProvider.connection.getAccountInfo(storagePDA);
    const storageBalanceAfter = await anchorProvider.connection.getAccountInfo(
      storage.publicKey
    );
    const storagePDALamportsAfter = storagePDABalanceAfter?.lamports || 0;
    const storageLamportsAfter = storageBalanceAfter?.lamports || 0;

    expect(mintInfoAfter.mintAuthority.toBase58()).to.equal(
      storage.publicKey.toBase58()
    );
    expect(tokenAccountAfter.owner.toBase58()).to.equal(
      storage.publicKey.toBase58()
    );
    expect(storagePDALamportsAfter).to.equal(0);
    expect(storageLamportsAfter).to.equal(
      storageLamportsBefore + storagePDALamportsBefore
    );
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
        storage: storage.publicKey,
        storageTokenAccount: storageTokenAddress,
        tokenProgram: TOKEN_PROGRAM_ID
      })
      .signers([storage])
      .rpc();

    const tokenAccountAfter = await getAccount(
      anchorProvider.connection,
      storageTokenAddress
    );

    expect(tokenAccountAfter.isFrozen).to.be.true;
  });
});
