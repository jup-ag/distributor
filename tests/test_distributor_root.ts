import * as anchor from "@coral-xyz/anchor";
import { BalanceTree } from "./merkle_tree";
import { web3 } from "@coral-xyz/anchor";
import {
  ADMIN,
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
} from "./common";
import { BN } from "bn.js";
import { Keypair, PublicKey } from "@solana/web3.js";
import { createMint, mintTo, TOKEN_PROGRAM_ID } from "@solana/spl-token";
import { assert } from "console";
const provider = anchor.AnchorProvider.env();

describe("Distribution Root", () => {
  let admin = Keypair.generate();
  let trees: BalanceTree[] = [];
  let maxNumNodes = 5;
  let whitelistedKPs: web3.Keypair[] = [];
  let amountUnlockedArr: anchor.BN[] = [];
  let amountLockedArr: anchor.BN[] = [];
  let totalClaimEachDistributor: anchor.BN[] = [];
  let mint: PublicKey;
  let depth = 2;
  let ONE_DAY = 86_400;
  let maxClaimAmountInAllDistributors = new BN(0);
  let maxDistributor = new BN(5);

  before(async () => {
    await createAndFundWallet(provider.connection, ADMIN);
    await createAndFundWallet(provider.connection, admin);

    for (let i = 0; i < maxDistributor.toNumber(); i++) {
      let totalClaim = new anchor.BN(0);
      for (let i = 0; i < maxNumNodes; i++) {
        const result = await createAndFundWallet(provider.connection);
        whitelistedKPs.push(result.keypair);
        let amountUnlocked = new BN(getRandomInt(1000, 20000));
        let amountLocked = new BN(getRandomInt(1000, 20000));

        amountUnlockedArr.push(amountUnlocked);
        amountLockedArr.push(amountLocked);
        totalClaim = totalClaim.add(amountUnlocked).add(amountLocked);
      }
      totalClaimEachDistributor.push(totalClaim);
      maxClaimAmountInAllDistributors =
        maxClaimAmountInAllDistributors.add(totalClaim);

      trees.push(
        new BalanceTree(
          whitelistedKPs.map((kp, index) => {
            return {
              account: kp.publicKey,
              amountUnlocked: amountUnlockedArr[index],
              amountLocked: amountLockedArr[index],
            };
          })
        )
      );
    }

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

    // mint to admin
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
      maxClaimAmountInAllDistributors.toNumber()
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
        maxClaimAmount: maxClaimAmountInAllDistributors,
        maxDistributor,
      });

    const distributors = [];
    const distributorVaults = [];
    for (let i = 0; i < maxDistributor.toNumber(); i++) {
      let canopyBufNodes = trees[i].getCanopyNodes(depth);
      let canopyNodes = [];
      canopyBufNodes.forEach(function (value) {
        canopyNodes.push(Array.from(new Uint8Array(value)));
      });

      let { distributor, tokenVault } = await createNewDistributor({
        admin,
        version: 0,
        totalClaim: totalClaimEachDistributor[i],
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

      distributors.push(distributor);
      distributorVaults.push(tokenVault);

      // create canopy tree correspond with distributor
      await createCanopyTree({
        admin,
        distributor,
        depth,
        root: Array.from(new Uint8Array(trees[i].getRoot())),
        canopyNodes,
      });
    }

    console.log("Fund to distributor root");
    // fund to distributor root
    await fundDistributorRoot({
      admin,
      payer: admin,
      distributorRoot,
      mint,
      maxAmount: maxClaimAmountInAllDistributors,
    });

    for (let i = 0; i < maxDistributor.toNumber(); i++) {
      // fund to distributor from root
      await fundMerkleDistributorFromRoot({
        admin,
        distributorRoot,
        distributorRootVault,
        distributor: distributors[i],
        distributorVault: distributorVaults[i],
      });

      const distributorVaultBalance = Number(
        (await provider.connection.getTokenAccountBalance(distributorVaults[i]))
          .value.amount
      );
      assert(
        distributorVaultBalance == totalClaimEachDistributor[i].toNumber()
      );
    }

    //
    const distributorRootVaultBalance = Number(
      (await provider.connection.getTokenAccountBalance(distributorRootVault))
        .value.amount
    );
    assert(distributorRootVaultBalance == 0);
  });
});
