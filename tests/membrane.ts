import * as anchor from '@project-serum/anchor';
import * as spl from '@solana/spl-token';
import { Program } from '@project-serum/anchor';
import { Keypair } from '@solana/web3.js';
import { Membrane } from '../target/types/membrane';
import { getAirdrop } from './utils';
import { expect } from 'chai';
import { calculateInitialRewardParams } from './utils/mocks';

describe('Membrane', () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  // const self = anchorProvider.wallet.publicKey;
  const program = anchor.workspace.Membrane as Program<Membrane>;
  const anchorProvider = program.provider as anchor.AnchorProvider;

  const systemProgram = anchor.web3.SystemProgram.programId;
  const tokenProgram = spl.TOKEN_PROGRAM_ID;

  let storage: Keypair;
  let reward: Keypair;

  before(async () => {
    // TODO: create first initialize script
    // Initialize storage
    // admin = authority = storage
    storage = Keypair.generate();
    // Airdrop storage
    await getAirdrop(program.provider.connection, storage.publicKey);
    // Generate reward
    reward = Keypair.generate();
  });

  it.skip('Can initialize mint', async () => {
    const mint = Keypair.generate();
    const rent = Keypair.generate();

    await program.methods
      .initializeMint()
      .accounts({
        mint: mint.publicKey,
        rent: rent.publicKey,
        authority: storage.publicKey,
        tokenProgram
      })
      .signers([])
      .rpc();
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
    //   victory: 8.927076289218453,
    //   topFive: 2.2317690723046133,
    //   topTen: 0.8927076289218454,
    //   kill: 1.2497906804905836
    // }

    const rewardMock = calculateInitialRewardParams();

    for (const key in rewardMock) {
      const result = rewardAccount[key];
      const mock = rewardMock[key];
      expect(result).to.be.a('number').and.to.equal(mock);
    }
  });
});
