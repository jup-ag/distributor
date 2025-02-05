import * as anchor from "@coral-xyz/anchor";
import { BalanceTree } from "./merkle_tree";
import { web3 } from "@coral-xyz/anchor";
import {
  ADMIN,
  claimAndStake,
  claimLockedAndStake,
  clawBack,
  createCanopyTree,
  createNewDistributor,
  createNewDistributorRoot,
  fundDistributorRoot,
  fundMerkleDistributorFromRoot,
} from "./merkle_distributor";
import {
  createAndFundWallet,
  getBlockTime,
  getOrCreateAssociatedTokenAccountWrap,
  getRandomInt,
  sleep,
} from "./common";
import { BN } from "bn.js";
import { Keypair, PublicKey } from "@solana/web3.js";
import { createMint, mintTo, TOKEN_PROGRAM_ID } from "@solana/spl-token";
import { createNewEscrowWithMaxLock, setupLocker } from "./locked_voter/setup";
const provider = anchor.AnchorProvider.env();

describe("Claim and stake permissionless", () => {
  let admin = Keypair.generate();
  let tree: BalanceTree;
  let maxNumNodes = 5;
  let whitelistedKPs: web3.Keypair[] = [];
  let amountUnlockedArr: anchor.BN[] = [];
  let amountLockedArr: anchor.BN[] = [];
  let totalClaim = new BN(0);
  let mint: PublicKey;
  let locker: PublicKey;
  let escrow: PublicKey;
  let depth = 2;
  let maxClaimAmount: anchor.BN;
  let maxDistributor: anchor.BN;
  before(async () => {
    let escrowOwner = Keypair.generate();
    await createAndFundWallet(provider.connection, ADMIN);
    await createAndFundWallet(provider.connection, admin);
    await createAndFundWallet(provider.connection, escrowOwner);

    for (let i = 0; i < maxNumNodes; i++) {
      const result = await createAndFundWallet(provider.connection);
      whitelistedKPs.push(result.keypair);
      let amountUnlocked = new BN(getRandomInt(1000, 20000));
      let amountLocked = new BN(getRandomInt(1000, 20000));

      amountUnlockedArr.push(amountUnlocked);
      amountLockedArr.push(amountLocked);
      totalClaim = totalClaim.add(amountUnlocked).add(amountLocked);
    }

    maxClaimAmount = totalClaim;
    maxDistributor = new anchor.BN(maxNumNodes);

    tree = new BalanceTree(
      whitelistedKPs.map((kp, index) => {
        return {
          account: kp.publicKey,
          amountUnlocked: amountUnlockedArr[index],
          amountLocked: amountLockedArr[index],
        };
      })
    );

    mint = await createMint(
      provider.connection,
      ADMIN,
      ADMIN.publicKey,
      null,
      6,
      web3.Keypair.generate(),
      {
        commitment: "confirmed",
      },
      TOKEN_PROGRAM_ID
    );

    // mint
    console.log("Mint to admin");
    const adminTokenAccount = await getOrCreateAssociatedTokenAccountWrap(
      provider.connection,
      admin,
      mint,
      admin.publicKey
    );

    await mintTo(
      provider.connection,
      ADMIN,
      mint,
      adminTokenAccount,
      ADMIN,
      totalClaim.toNumber()
    );

    console.log("create locker");
    locker = await setupLocker({
      payer: ADMIN,
      tokenMint: mint,
      maxStakeVoteMultiplier: 1,
      minStakeDuration: new BN(10),
      maxStakeDuration: new BN(10000),
      proposalActivationMinVotes: new BN(100),
    });

    console.log("create escrow");
    escrow = await createNewEscrowWithMaxLock({
      locker,
      escrowOwner,
    });
  });
  it("Full flow", async () => {
    console.log("create distributor");
    let currentTime = await getBlockTime(provider.connection);
    let startVestingTs = new BN(currentTime + 3);
    let endVestingTs = new BN(currentTime + 6);
    let clawbackStartTs = new BN(currentTime + 7);
    let activationType = 1; // timestamp
    let activationPoint = new BN(currentTime + 2);
    let closable = false;
    let totalBonus = new BN(0);
    let bonusVestingDuration = new BN(0);
    let claimType = 2;
    let operator = web3.SystemProgram.programId;
    let canopyBufNodes = tree.getCanopyNodes(depth);
    ////
    let canopyNodes = [];
    canopyBufNodes.forEach(function (value) {
      canopyNodes.push(Array.from(new Uint8Array(value)));
    });

    let clawbackReceiver = await getOrCreateAssociatedTokenAccountWrap(
      provider.connection,
      ADMIN,
      mint,
      ADMIN.publicKey
    );

    // create distributor root
    let { distributorRoot, distributorRootVault } =
      await createNewDistributorRoot({
        admin,
        mint,
        maxClaimAmount,
        maxDistributor,
      });

    let { distributor, tokenVault } = await createNewDistributor({
      admin,
      version: 0,
      totalClaim,
      maxNumNodes: new BN(maxNumNodes),
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
      mint,
      clawbackReceiver,
      distributorRoot,
    });

    // create canopy tree correspond with distributor
    let canopyTree = await createCanopyTree({
      admin,
      distributor,
      depth,
      root: Array.from(new Uint8Array(tree.getRoot())),
      canopyNodes,
    });

    // fund to distributor root
    await fundDistributorRoot({
      admin,
      payer: admin,
      distributorRoot,
      mint,
      maxAmount: maxClaimAmount,
    });
    // fund to distributor from root
    await fundMerkleDistributorFromRoot({
      admin,
      distributorRoot,
      distributorRootVault,
      distributor,
      distributorVault: tokenVault,
    });

    while (true) {
      const currentTime = await getBlockTime(provider.connection);
      if (currentTime > activationPoint.toNumber()) {
        break;
      } else {
        await sleep(1000);
        console.log("Wait until activationPoint");
      }
    }

    console.log("claim and stake");
    for (let i = 0; i < maxNumNodes - 1; i++) {
      var proofBuffers = tree.getPartialProof(
        whitelistedKPs[i].publicKey,
        amountUnlockedArr[i],
        amountLockedArr[i],
        depth
      );
      let proof = [];
      proofBuffers.proof.forEach(function (value) {
        proof.push(Array.from(new Uint8Array(value)));
      });
      console.log("claim index: ", i);
      await claimAndStake({
        distributor,
        claimant: whitelistedKPs[i],
        amountUnlocked: amountUnlockedArr[i],
        amountLocked: amountLockedArr[i],
        proof,
        escrow,
        leafIndex: proofBuffers.index,
      });
    }

    while (true) {
      const currentTime = await getBlockTime(provider.connection);
      if (currentTime > startVestingTs.toNumber()) {
        break;
      } else {
        await sleep(1000);
        console.log("Wait until startVestingTs");
      }
    }
    console.log("claim locked");
    for (let i = 0; i < maxNumNodes - 1; i++) {
      console.log("claim locked index: ", i);
      await claimLockedAndStake({
        distributor,
        claimant: whitelistedKPs[i],
        escrow,
      });
    }

    while (true) {
      const currentTime = await getBlockTime(provider.connection);
      if (currentTime > clawbackStartTs.toNumber()) {
        break;
      } else {
        await sleep(1000);
        console.log("Wait until clawbackStartTs");
      }
    }
    console.log("clawback");
    await clawBack({
      distributor,
      payer: ADMIN,
    });
  });
});
