import { web3, Wallet, BN } from "@coral-xyz/anchor";
import {
  ASSOCIATED_TOKEN_PROGRAM_ID,
  createMint,
  createSyncNativeInstruction,
  getAssociatedTokenAddressSync,
  getOrCreateAssociatedTokenAccount,
  mintTo,
  NATIVE_MINT,
  TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
import {
  Connection,
  Keypair,
  sendAndConfirmTransaction,
  SystemProgram,
  Transaction,
} from "@solana/web3.js";

export * from "./asserter";

export async function createAndFundWallet(
  connection: web3.Connection,
  keypair?: web3.Keypair
) {
  if (!keypair) {
    keypair = web3.Keypair.generate();
  }

  const tx = await connection.requestAirdrop(
    keypair.publicKey,
    1000 * web3.LAMPORTS_PER_SOL
  );

  await connection.confirmTransaction(tx, "confirmed");

  const wallet = new Wallet(keypair);
  return {
    keypair,
    wallet,
  };
}

export const encodeU32 = (num: number): Buffer => {
  const buf = Buffer.alloc(4);
  buf.writeUInt32LE(num);
  return buf;
};

export const encodeU64 = (num: number): Buffer => {
  const buf = Buffer.alloc(8);
  buf.writeBigUint64LE(BigInt(num));
  return buf;
};

export async function sleep(ms: number) {
  return new Promise((res) => setTimeout(res, ms));
}

export const SET_COMPUTE_UNIT_LIMIT_IX =
  web3.ComputeBudgetProgram.setComputeUnitLimit({
    units: 1_400_000,
  });

export const createMintIfNotExists = async (
  connection: web3.Connection,
  payer: web3.Keypair,
  mintAuthority: web3.PublicKey,
  decimals: number,
  mintKeypair: web3.Keypair,
  tokenProgramId: web3.PublicKey
) => {
  const mint = await connection.getAccountInfo(mintKeypair.publicKey);
  if (!mint) {
    return await createMint(
      connection,
      payer,
      mintAuthority,
      null,
      decimals,
      mintKeypair,
      null,
      tokenProgramId
    );
  }
  return mintKeypair.publicKey;
};
export const mintTokenTo = async (
  connection: web3.Connection,
  mintAuthority: web3.Keypair,
  tokenMint: web3.PublicKey,
  owner: web3.PublicKey,
  amount: number
) => {
  const userToken = await getOrCreateAssociatedTokenAccount(
    connection,
    mintAuthority,
    tokenMint,
    owner,
    false
  );

  await mintTo(
    connection,
    mintAuthority,
    tokenMint,
    userToken.address,
    mintAuthority.publicKey,
    amount,
    []
  );
};

export const getBlockTime = async (connection: web3.Connection) => {
  let slot = await connection.getSlot();
  let blockTime = await connection.getBlockTime(slot);
  return blockTime;
};

export const wrapSOL = async (
  connection: Connection,
  amount: BN,
  payer: Keypair
) => {
  const ata = getAssociatedTokenAddressSync(NATIVE_MINT, payer.publicKey);
  const account = await connection.getAccountInfo(ata);

  if (!account) {
    await getOrCreateAssociatedTokenAccount(
      connection,
      payer,
      NATIVE_MINT,
      payer.publicKey
    );
  }

  const transaction = new Transaction()
    .add(
      SystemProgram.transfer({
        fromPubkey: payer.publicKey,
        toPubkey: ata,
        lamports: BigInt(amount.toString()),
      })
    )
    .add(createSyncNativeInstruction(ata));

  await sendAndConfirmTransaction(connection, transaction, [payer]);
};


export function getRandomInt(min, max) {
  const minCeiled = Math.ceil(min);
  const maxFloored = Math.floor(max);
  return Math.floor(Math.random() * (maxFloored - minCeiled) + minCeiled); // The maximum is exclusive and the minimum is inclusive
}




export const getOrCreateAssociatedTokenAccountWrap = async (connection: web3.Connection, payer: web3.Keypair, tokenMint: web3.PublicKey, owner: web3.PublicKey) => {
  return (await getOrCreateAssociatedTokenAccount(
    connection,
    payer,
    tokenMint,
    owner,
    true,
    "confirmed",
    {
      commitment: "confirmed",
    },
    TOKEN_PROGRAM_ID,
    ASSOCIATED_TOKEN_PROGRAM_ID
  )).address;
}

