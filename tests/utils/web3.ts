import {
  Connection,
  Keypair,
  LAMPORTS_PER_SOL,
  PublicKey,
  TransactionSignature
} from '@solana/web3.js';
import {
  createMint,
  getOrCreateAssociatedTokenAccount,
  mintTo,
  transfer,
  Account,
  getAccount,
  TOKEN_PROGRAM_ID,
  ASSOCIATED_TOKEN_PROGRAM_ID
} from '@solana/spl-token';

export const getAirdrop = async (
  connection: Connection,
  publicKey: PublicKey,
  amount = 1
) => {
  const signature = await connection.requestAirdrop(
    publicKey,
    LAMPORTS_PER_SOL * amount
  );
  await connection.confirmTransaction(signature);
};

export const createToken = async (
  connection: Connection,
  payer: Keypair,
  mintAuthority: PublicKey,
  freezeAuthority: PublicKey = null,
  decimals: number
): Promise<PublicKey> => {
  // Create new token mint
  return createMint(
    connection,
    payer,
    mintAuthority,
    freezeAuthority,
    decimals
  );
};

export const mintToken = async (
  connection: Connection,
  mintAddress: PublicKey,
  payer: Keypair,
  mintAuthority: PublicKey,
  destination: PublicKey,
  amount: number
): Promise<Account> => {
  // Get the token account of the mintTo address, and if it does not exist, create it
  const tokenAccount = await getOrCreateAssociatedTokenAccount(
    connection,
    payer,
    mintAddress,
    destination
  );

  // Mint new token to the tokenAccount we just created
  await mintTo(
    connection,
    payer,
    mintAddress,
    tokenAccount.address,
    mintAuthority,
    amount
  );

  // Get the amount of tokens in the tokenAccount
  return getAccount(connection, tokenAccount.address);
};

export const findAssociatedTokenAddress = async (
  walletAddress: PublicKey,
  mintAddress: PublicKey
): Promise<PublicKey> => {
  return (
    await PublicKey.findProgramAddress(
      [
        walletAddress.toBuffer(),
        TOKEN_PROGRAM_ID.toBuffer(),
        mintAddress.toBuffer()
      ],
      ASSOCIATED_TOKEN_PROGRAM_ID
    )
  )[0];
};

export const sendToken = async (
  connection: Connection,
  mintAddress: PublicKey,
  fromWallet: Keypair,
  toWallet: PublicKey,
  amount: number
): Promise<TransactionSignature> => {
  // Get the token account of the fromWallet address
  const fromTokenAccount = await findAssociatedTokenAddress(
    fromWallet.publicKey,
    mintAddress
  );

  // Get the token account of the toWallet address, and if it does not exist, create it
  const toTokenAccount = await getOrCreateAssociatedTokenAccount(
    connection,
    fromWallet,
    mintAddress,
    toWallet
  );

  // Make transfer
  return transfer(
    connection,
    fromWallet,
    fromTokenAccount,
    toTokenAccount.address,
    fromWallet.publicKey,
    amount
  );
};
