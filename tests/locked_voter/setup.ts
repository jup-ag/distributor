import { AnchorProvider, Program, Wallet, web3, BN } from "@coral-xyz/anchor";
import { Keypair, PublicKey } from "@solana/web3.js";
import { LockedVoter, IDL as LockedVoterIDL } from "./locked_voter";
import { getOrCreateAssociatedTokenAccountWrap } from "../common";

export const LOCKED_VOTER_PROGRAM_ID = new web3.PublicKey(
    "voTpe3tHQ7AjQHMapgSue2HJFAh2cGsdokqN3XqmVSj"
);


export function createLockedVoterProgram(wallet: Wallet): Program<LockedVoter> {
    const provider = new AnchorProvider(AnchorProvider.env().connection, wallet, {
        maxRetries: 3,
    });

    const program = new Program<LockedVoter>(
        LockedVoterIDL,
        LOCKED_VOTER_PROGRAM_ID,
        provider
    );

    return program;
}

export function deriveLocker(
    base: web3.PublicKey,
) {
    let [pk, _] = web3.PublicKey.findProgramAddressSync(
        [Buffer.from("Locker"), base.toBuffer()],
        LOCKED_VOTER_PROGRAM_ID
    );
    return pk
}


export function deriveEscrow(
    locker: web3.PublicKey,
    owner: web3.PublicKey,
) {
    let [pk, _] = web3.PublicKey.findProgramAddressSync(
        [Buffer.from("Escrow"), locker.toBuffer(), owner.toBuffer()],
        LOCKED_VOTER_PROGRAM_ID
    );
    return pk
}


export interface SetupLockerParams {
    payer: Keypair;
    tokenMint: PublicKey,
    maxStakeVoteMultiplier: number;
    minStakeDuration: BN;
    maxStakeDuration: BN;
    proposalActivationMinVotes: BN;
}



export async function setupLocker(params: SetupLockerParams) {
    let { payer, maxStakeVoteMultiplier, minStakeDuration, maxStakeDuration, proposalActivationMinVotes, tokenMint } = params;
    let program = createLockedVoterProgram(new Wallet(payer));
    let base = Keypair.generate();
    let locker = deriveLocker(base.publicKey);
    await program.methods.newLocker({
        maxStakeVoteMultiplier,
        maxStakeDuration,
        minStakeDuration,
        proposalActivationMinVotes
    }).accounts({
        base: base.publicKey,
        locker,
        tokenMint,
        governor: PublicKey.unique(), // just use a random governor for now
        payer: payer.publicKey,
        systemProgram: web3.SystemProgram.programId,
    }).signers([base]).rpc().catch(console.log).then(console.log);
    return locker
}

export interface CreateNewEscrowParams {
    locker: PublicKey;
    escrowOwner: Keypair;
}

export async function createNewEscrowWithMaxLock(params: CreateNewEscrowParams) {
    let { locker, escrowOwner } = params;
    let program = createLockedVoterProgram(new Wallet(escrowOwner));
    let lockerState = await program.account.locker.fetch(locker);
    let escrow = deriveEscrow(locker, escrowOwner.publicKey);
    await program.methods.newEscrow().accounts({
        locker,
        escrow,
        escrowOwner: escrowOwner.publicKey,
        payer: escrowOwner.publicKey,
        systemProgram: web3.SystemProgram.programId,
    }).rpc().catch(console.log).then(console.log);

    await program.methods.toggleMaxLock(true).accounts({
        locker,
        escrow,
        escrowOwner: escrowOwner.publicKey,
    }).rpc().catch(console.log).then(console.log);

    await getOrCreateAssociatedTokenAccountWrap(program.provider.connection, escrowOwner, lockerState.tokenMint, escrow);

    return escrow
}