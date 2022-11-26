import { web3, Provider } from "@project-serum/anchor";
import fs from "fs";

export const handleAirdrop = async (
  provider: Provider,
  account: web3.PublicKey,
  amount: number = web3.LAMPORTS_PER_SOL
) => {
  const airdropSignature = await provider.connection.requestAirdrop(
    account,
    amount
  );
  await provider.connection.confirmTransaction(airdropSignature);
};

export const getPDA = async (
  seeds: Buffer[],
  programId: web3.PublicKey
): Promise<web3.PublicKey> => {
  const [pda] = await web3.PublicKey.findProgramAddress(seeds, programId);
  return pda;
};

export const getKeypairFromFile = (file: string): web3.Keypair => {
  const rawdata = fs.readFileSync(file);
  const keyData = JSON.parse(rawdata.toString());
  return web3.Keypair.fromSecretKey(new Uint8Array(keyData));
};

export const getTokenBalance = async (
  pubkey: web3.PublicKey,
  provider: Provider
) => {
  return parseInt(
    (await provider.connection.getTokenAccountBalance(pubkey)).value.amount
  );
};

export const getSolBalance = async (
  pubkey: web3.PublicKey,
  provider: Provider
) => {
  return await provider.connection.getBalance(pubkey);
};
