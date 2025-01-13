import fs from "fs";
import { Keypair, PublicKey, Transaction } from "@solana/web3.js";
import BN from "bn.js";
import { AnchorProvider, Program, Wallet, web3 } from "@coral-xyz/anchor";
import {
    MerkleDistributor,
    IDL as MerkleDistributorIDL,
} from "../../target/types/merkle_distributor";
import { encodeU64, getOrCreateAssociatedTokenAccountWrap } from "../common";
import { getAssociatedTokenAddressSync, TOKEN_PROGRAM_ID } from "@solana/spl-token";
import { LOCKED_VOTER_PROGRAM_ID } from "../locked_voter/setup";

const MERKLE_DISTRIBUTOR_PROGRAM_ID = new web3.PublicKey(
    "DiS3nNjFVMieMgmiQFm6wgJL7nevk4NrhXKLbtEH1Z2R"
);

const res = fs.readFileSync(
    process.cwd() +
    "/keys/localnet/admin-bossj3JvwiNK7pvjr149DqdtJxf2gdygbcmEPTkb2F1.json",
    "utf8"
);

export function deriveDistributor(
    base: web3.PublicKey,
    mint: web3.PublicKey,
    version: number,
) {
    let [pk, _] = web3.PublicKey.findProgramAddressSync(
        [Buffer.from("MerkleDistributor"), base.toBuffer(), mint.toBuffer(), encodeU64(version)],
        MERKLE_DISTRIBUTOR_PROGRAM_ID
    );
    return pk
}



export function deriveClaimStatus(
    distributor: web3.PublicKey,
    claimant: web3.PublicKey,
) {
    let [pk, _] = web3.PublicKey.findProgramAddressSync(
        [Buffer.from("ClaimStatus"), claimant.toBuffer(), distributor.toBuffer()],
        MERKLE_DISTRIBUTOR_PROGRAM_ID
    );
    return pk
}



export const ADMIN = Keypair.fromSecretKey(new Uint8Array(JSON.parse(res)));

export const ADMIN_PUBKEY = ADMIN.publicKey;

export function createDistributorProgram(wallet: Wallet): Program<MerkleDistributor> {
    const provider = new AnchorProvider(AnchorProvider.env().connection, wallet, {
        maxRetries: 3,
    });
    const program = new Program<MerkleDistributor>(
        MerkleDistributorIDL,
        MERKLE_DISTRIBUTOR_PROGRAM_ID,
        provider
    );
    return program;
}


export interface CreateNewDisitrbutorParams {
    admin: Keypair;
    version: number;
    root: Buffer;
    totalClaim: BN;
    maxNumNodes: BN;
    startVestingTs: BN;
    endVestingTs: BN;
    clawbackStartTs: BN;
    activationPoint: BN;
    activationType: number;
    closable: boolean;
    totalBonus: BN;
    bonusVestingDuration: BN;
    claimType: number;
    operator: PublicKey;
    locker: PublicKey;
    mint: PublicKey;
    clawbackReceiver: PublicKey;
}

export async function createNewDistributor(
    params: CreateNewDisitrbutorParams
) {
    let {
        admin,
        version,
        root,
        totalClaim,
        maxNumNodes,
        startVestingTs,
        endVestingTs,
        clawbackStartTs,
        activationPoint,
        activationType,
        closable, totalBonus, bonusVestingDuration,
        claimType, operator, locker, mint, clawbackReceiver
    } = params;
    const program = createDistributorProgram(new Wallet(admin));

    let base = Keypair.generate();

    let distributor = deriveDistributor(base.publicKey, mint, version);
    let tokenVault = await getOrCreateAssociatedTokenAccountWrap(program.provider.connection, admin, mint, distributor);
    await program.methods
        .newDistributor({
            version: new BN(version),
            root: Array.from(new Uint8Array(root)),
            totalClaim,
            maxNumNodes,
            startVestingTs,
            endVestingTs,
            clawbackStartTs,
            activationPoint,
            activationType,
            closable,
            totalBonus,
            bonusVestingDuration,
            claimType,
            operator,
            locker,
        })
        .accounts({
            distributor,
            mint,
            clawbackReceiver,
            tokenVault,
            admin: admin.publicKey,
            base: base.publicKey,
            systemProgram: web3.SystemProgram.programId,
            tokenProgram: TOKEN_PROGRAM_ID,
        }).signers([base])
        .rpc().catch(console.log).then(console.log);

    return { distributor, tokenVault };
}

export interface ClaimParams {
    claimant: Keypair;
    operator?: Keypair;
    distributor: PublicKey,
    amountUnlocked: BN;
    amountLocked: BN;
    proof: Array<number>[];
}


export async function claim(
    params: ClaimParams
) {
    let { claimant, amountUnlocked, amountLocked, proof, distributor, operator } = params;
    const program = createDistributorProgram(new Wallet(claimant));

    let distributorState = await program.account.merkleDistributor.fetch(distributor);
    let claimStatus = deriveClaimStatus(distributor, claimant.publicKey);
    let to = await getOrCreateAssociatedTokenAccountWrap(program.provider.connection, claimant, distributorState.mint, claimant.publicKey);

    if (operator == null) {
        await program.methods.newClaim(amountUnlocked, amountLocked, proof).accounts({
            distributor,
            claimant: claimant.publicKey,
            claimStatus,
            from: distributorState.tokenVault,
            to,
            systemProgram: web3.SystemProgram.programId,
            tokenProgram: TOKEN_PROGRAM_ID,
            operator: null,
        }).rpc().catch(console.log).then(console.log);
    } else {
        // user sign tx firstly (need to verify signature to avoid spaming)
        let tx = await program.methods.newClaim(amountUnlocked, amountLocked, proof).accounts({
            distributor,
            claimant: claimant.publicKey,
            claimStatus,
            from: distributorState.tokenVault,
            to,
            systemProgram: web3.SystemProgram.programId,
            tokenProgram: TOKEN_PROGRAM_ID,
            operator: operator.publicKey,
        }).transaction();

        // pass tx to operator to sign
        const { blockhash, lastValidBlockHeight } =
            await program.provider.connection.getLatestBlockhash();
        tx.feePayer = claimant.publicKey;
        tx.recentBlockhash = blockhash;
        tx.lastValidBlockHeight = lastValidBlockHeight;
        tx.partialSign(operator);

        // pass back user to sign
        const signedTx = await new Wallet(claimant).signTransaction(tx);
        const txHash = await program.provider.connection.sendRawTransaction(signedTx.serialize());
        console.log(txHash);
    }
}


export interface ClaimAndStakeParams {
    claimant: Keypair;
    escrow: PublicKey,
    operator?: Keypair;
    distributor: PublicKey,
    amountUnlocked: BN;
    amountLocked: BN;
    proof: Array<number>[];
}

export async function claimAndStake(
    params: ClaimAndStakeParams
) {
    let { claimant, amountUnlocked, amountLocked, proof, distributor, operator, escrow } = params;
    const program = createDistributorProgram(new Wallet(claimant));

    let distributorState = await program.account.merkleDistributor.fetch(distributor);
    let claimStatus = deriveClaimStatus(distributor, claimant.publicKey);

    if (operator == null) {
        await program.methods.newClaimAndStake(amountUnlocked, amountLocked, proof).accounts({
            distributor,
            claimant: claimant.publicKey,
            claimStatus,
            from: distributorState.tokenVault,
            systemProgram: web3.SystemProgram.programId,
            tokenProgram: TOKEN_PROGRAM_ID,
            operator: null,
            voterProgram: LOCKED_VOTER_PROGRAM_ID,
            locker: distributorState.locker,
            escrow,
            escrowTokens: getAssociatedTokenAddressSync(distributorState.mint, escrow, true),

        }).rpc().catch(console.log).then(console.log);
    } else {
        await program.methods.newClaimAndStake(amountUnlocked, amountLocked, proof).accounts({
            distributor,
            claimant: claimant.publicKey,
            claimStatus,
            from: distributorState.tokenVault,
            systemProgram: web3.SystemProgram.programId,
            tokenProgram: TOKEN_PROGRAM_ID,
            operator: operator.publicKey,
            voterProgram: LOCKED_VOTER_PROGRAM_ID,
            locker: distributorState.locker,
            escrow,
            escrowTokens: getAssociatedTokenAddressSync(distributorState.mint, escrow, true),
        }).signers([operator]).rpc().catch(console.log).then(console.log);
    }
}


export interface ClaimLockedParams {
    claimant: Keypair;
    operator?: Keypair;
    distributor: PublicKey,
}

export async function claimLocked(
    params: ClaimLockedParams
) {
    let { claimant, distributor, operator } = params;
    const program = createDistributorProgram(new Wallet(claimant));

    let distributorState = await program.account.merkleDistributor.fetch(distributor);
    let claimStatus = deriveClaimStatus(distributor, claimant.publicKey);
    let to = await getOrCreateAssociatedTokenAccountWrap(program.provider.connection, claimant, distributorState.mint, claimant.publicKey);

    if (operator == null) {
        await program.methods.claimLocked().accounts({
            distributor,
            claimant: claimant.publicKey,
            claimStatus,
            from: distributorState.tokenVault,
            to,
            tokenProgram: TOKEN_PROGRAM_ID,
            operator: null,
        }).rpc().catch(console.log).then(console.log);
    } else {
        await program.methods.claimLocked().accounts({
            distributor,
            claimant: claimant.publicKey,
            claimStatus,
            from: distributorState.tokenVault,
            to,
            tokenProgram: TOKEN_PROGRAM_ID,
            operator: operator.publicKey,
        }).signers([operator]).rpc().catch(console.log).then(console.log);
    }
}



export interface ClaimLockedAndStakeParams {
    claimant: Keypair;
    operator?: Keypair;
    distributor: PublicKey,
    escrow: PublicKey,
}

export async function claimLockedAndStake(
    params: ClaimLockedAndStakeParams
) {
    let { claimant, distributor, operator, escrow } = params;
    const program = createDistributorProgram(new Wallet(claimant));

    let distributorState = await program.account.merkleDistributor.fetch(distributor);
    let claimStatus = deriveClaimStatus(distributor, claimant.publicKey);

    if (operator == null) {
        await program.methods.claimLockedAndStake().accounts({
            distributor,
            claimant: claimant.publicKey,
            claimStatus,
            from: distributorState.tokenVault,
            tokenProgram: TOKEN_PROGRAM_ID,
            operator: null,
            voterProgram: LOCKED_VOTER_PROGRAM_ID,
            locker: distributorState.locker,
            escrow,
            escrowTokens: getAssociatedTokenAddressSync(distributorState.mint, escrow, true),
        }).rpc().catch(console.log).then(console.log);
    } else {
        await program.methods.claimLockedAndStake().accounts({
            distributor,
            claimant: claimant.publicKey,
            claimStatus,
            from: distributorState.tokenVault,
            tokenProgram: TOKEN_PROGRAM_ID,
            operator: operator.publicKey,
            voterProgram: LOCKED_VOTER_PROGRAM_ID,
            locker: distributorState.locker,
            escrow,
            escrowTokens: getAssociatedTokenAddressSync(distributorState.mint, escrow, true),
        }).signers([operator]).rpc().catch(console.log).then(console.log);
    }
}


export interface ClawbackParams {
    payer: Keypair;
    distributor: PublicKey,
}

export async function clawBack(
    params: ClawbackParams
) {
    let { payer, distributor } = params;
    const program = createDistributorProgram(new Wallet(payer));

    let distributorState = await program.account.merkleDistributor.fetch(distributor);

    await program.methods.clawback().accounts({
        distributor,
        from: distributorState.tokenVault,
        clawbackReceiver: distributorState.clawbackReceiver,
        tokenProgram: TOKEN_PROGRAM_ID,
    }).rpc().catch(console.log).then(console.log);
}