import {Connection, LAMPORTS_PER_SOL, PublicKey} from "@solana/web3.js";

export const getAirdrop = async (
  connection: Connection,
  publicKey: PublicKey,
  amount = 1
) => {
  const signature = await connection.requestAirdrop(publicKey, LAMPORTS_PER_SOL * amount);
  await connection.confirmTransaction(signature);
};
