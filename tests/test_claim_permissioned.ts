import * as anchor from "@coral-xyz/anchor";
import { BalanceTree } from "./merkle_tree";
import { Wallet, web3 } from "@coral-xyz/anchor";
import {
  ADMIN,
  claim,
  claimLocked,
  clawBack,
  createNewDistributor,
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
const provider = anchor.AnchorProvider.env();

describe("Claim permissioned", () => {
  let admin = Keypair.generate();
  let operator = Keypair.generate();
  let tree: BalanceTree;
  let maxNumNodes = 5;
  let whitelistedKPs: web3.Keypair[] = [];
  let amountUnlockedArr: anchor.BN[] = [];
  let amountLockedArr: anchor.BN[] = [];
  let totalClaim = new BN(0);
  let mint: PublicKey;
  let sliceLayers = 3;

  before(async () => {
    await createAndFundWallet(provider.connection, ADMIN);
    await createAndFundWallet(provider.connection, admin);

    for (let i = 0; i < maxNumNodes; i++) {
      const result = await createAndFundWallet(provider.connection);
      whitelistedKPs.push(result.keypair);
      let amountUnlocked = new BN(getRandomInt(1000, 20000));
      let amountLocked = new BN(getRandomInt(1000, 20000));

      amountUnlockedArr.push(amountUnlocked);
      amountLockedArr.push(amountLocked);
      totalClaim = totalClaim.add(amountUnlocked).add(amountLocked);
    }

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
    let claimType = 1;
    let locker = web3.SystemProgram.programId;
    let partialMerkleTree = tree.getPartialBfsTree(sliceLayers);
    ////
    let nodes = [];
    partialMerkleTree.forEach(function (value) {
      nodes.push(Array.from(new Uint8Array(value)));
    });

    let clawbackReceiver = await getOrCreateAssociatedTokenAccountWrap(
      provider.connection,
      ADMIN,
      mint,
      ADMIN.publicKey
    );
    let { distributor, tokenVault } = await createNewDistributor({
      admin,
      version: 0,
      root: tree.getRoot(),
      depth: sliceLayers - 1,
      nodes,
      totalNodes: nodes.length,
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
      operator: operator.publicKey,
      locker,
      mint,
      clawbackReceiver,
    });
    // mint
    await mintTo(
      provider.connection,
      ADMIN,
      mint,
      tokenVault,
      ADMIN,
      totalClaim.toNumber()
    );

    while (true) {
      const currentTime = await getBlockTime(provider.connection);
      if (currentTime > activationPoint.toNumber()) {
        break;
      } else {
        await sleep(1000);
        console.log("Wait until activationPoint");
      }
    }

    console.log("claim");
    for (let i = 0; i < maxNumNodes - 1; i++) {
      var proofBuffers = tree.getPartialProof(
        whitelistedKPs[i].publicKey,
        amountUnlockedArr[i],
        amountLockedArr[i],
        sliceLayers
      );
      let proof = [];
      proofBuffers.proof.forEach(function (value) {
        proof.push(Array.from(new Uint8Array(value)));
      });
      console.log("claim index: ", i);
      await claim({
        distributor,
        claimant: whitelistedKPs[i],
        amountUnlocked: amountUnlockedArr[i],
        amountLocked: amountLockedArr[i],
        proof,
        operator,
        initialIndex: proofBuffers.index,
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
      await claimLocked({
        distributor,
        operator,
        claimant: whitelistedKPs[i],
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
