import * as anchor from '@project-serum/anchor';
import * as spl from '@solana/spl-token';
import { Program } from '@project-serum/anchor';
import { Keypair } from '@solana/web3.js';
import { Membrane } from '../target/types/membrane';
import { getAirdrop } from './utils';

describe('Membrane', () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  // const self = anchorProvider.wallet.publicKey;
  const program = anchor.workspace.Membrane as Program<Membrane>;
  const anchorProvider = program.provider as anchor.AnchorProvider;

  const systemProgram = anchor.web3.SystemProgram.programId;
  const tokenProgram = spl.TOKEN_PROGRAM_ID;

  let storage: Keypair;

  before(async () => {
    // TODO: create first initialize script
    // Initialize storage
    // admin = authority = storage
    storage = Keypair.generate();
    // Airdrop storage
    await getAirdrop(program.provider.connection, storage.publicKey);
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
    const reward = Keypair.generate();

    await program.methods
      .initializeReward()
      .accounts({
        reward: reward.publicKey,
        admin: storage.publicKey,
        systemProgram
      })
      .signers([storage, reward])
      .rpc();

    const rewardAccount = await program.account.reward.fetch(reward.publicKey);

    console.log(rewardAccount);
  });
});
