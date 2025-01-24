import * as anchor from "@coral-xyz/anchor";
import { BalanceTree } from "./merkle_tree";
import { web3 } from "@coral-xyz/anchor";
import {
  ADMIN,
  createNewDistributor,
  createNewParentAccount,
  distributeVault,
} from "./merkle_distributor";
import {
  createAndFundWallet,
  getBlockTime,
  getOrCreateAssociatedTokenAccountWrap,
  getRandomInt,
} from "./common";
import { BN } from "bn.js";
import { AccountMeta, Keypair, PublicKey } from "@solana/web3.js";
import { createMint, mintTo, TOKEN_PROGRAM_ID } from "@solana/spl-token";
import { assert } from "console";
const provider = anchor.AnchorProvider.env();

describe("Distribution vault", () => {
  let admin = Keypair.generate();
  let tree: BalanceTree;
  let maxNumNodes = 5;
  let numDistributor = 5;
  let whitelistedKPs: web3.Keypair[] = [];
  let amountUnlockedArr: anchor.BN[] = [];
  let amountLockedArr: anchor.BN[] = [];
  let totalClaim = new BN(0);
  let mint: PublicKey;
  let sliceLayers = 3;
  let ONE_DAY = 86_400;

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
    let startVestingTs = new BN(currentTime + ONE_DAY);
    let endVestingTs = new BN(currentTime + ONE_DAY * 7);
    let clawbackStartTs = new BN(currentTime + ONE_DAY * 8);
    let activationType = 1; // timestamp
    let activationPoint = new BN(currentTime + ONE_DAY / 2);
    let closable = false;
    let totalBonus = new BN(0);
    let bonusVestingDuration = new BN(0);
    let claimType = 0;
    let operator = web3.SystemProgram.programId;
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

    let { parentAccount, parentVault } = await createNewParentAccount({
      admin,
      mint,
    });

    // mint
    const totalClaimForDistributors = totalClaim.toNumber() * numDistributor;
    await mintTo(
      provider.connection,
      ADMIN,
      mint,
      parentVault,
      ADMIN,
      totalClaimForDistributors
    );
    const preParentVaultBalance = Number(
      (await provider.connection.getTokenAccountBalance(parentVault)).value
        .amount
    );
    assert(totalClaimForDistributors == preParentVaultBalance);

    let remainingAccounts: AccountMeta[] = [];

    for (let i = 0; i < numDistributor; i++) {
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
        operator,
        locker,
        mint,
        clawbackReceiver,
      });

      // remaining account
      const accounts: AccountMeta[] = [
        {
          pubkey: tokenVault,
          isSigner: false,
          isWritable: true,
        },
        {
          pubkey: distributor,
          isSigner: false,
          isWritable: false,
        },
      ];
      remainingAccounts.push(...accounts);
    }

    console.log("Distribute vault");

    let _ = await distributeVault({
      admin,
      parentAccount,
      parentVault,
      remainingAccounts,
    });

    // post distribution
    const postParentVaultBalance = Number(
      (await provider.connection.getTokenAccountBalance(parentVault)).value
        .amount
    );
    assert(postParentVaultBalance == 0);
    for (let i = 0; i < remainingAccounts.length; i += 2) {
      const tokenVault = remainingAccounts[i].pubkey;
      const tokenVaultBalance = Number(
        (await provider.connection.getTokenAccountBalance(tokenVault)).value
          .amount
      );
      assert(tokenVaultBalance == totalClaim.toNumber());
    }
  });
});
