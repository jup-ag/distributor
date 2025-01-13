import { BN, web3 } from "@coral-xyz/anchor";
import { sha256 } from "js-sha256";

import { MerkleTree } from "./merkle-tree";


export class BalanceTree {
  private readonly _tree: MerkleTree;
  constructor(balances: { account: web3.PublicKey; amountUnlocked: BN, amountLocked: BN }[]) {
    this._tree = new MerkleTree(
      balances.map(({ account, amountUnlocked, amountLocked }, index) => {
        return BalanceTree.toNode(account, amountUnlocked, amountLocked);
      })
    );
  }

  static verifyProof(
    account: web3.PublicKey,
    amountUnlocked: BN,
    amountLocked: BN,
    proof: Buffer[],
    root: Buffer
  ): boolean {
    let pair = BalanceTree.toNode(account, amountUnlocked, amountLocked);
    for (const item of proof) {
      pair = MerkleTree.combinedHash(pair, item);
    }

    return pair.equals(root);
  }

  // keccak256(abi.encode(index, account, amount))
  static toNode(account: web3.PublicKey, amountUnlocked: BN, amountLocked: BN): Buffer {
    const buf = Buffer.concat([
      account.toBuffer(),
      new BN(amountUnlocked).toArrayLike(Buffer, "le", 8),
      new BN(amountLocked).toArrayLike(Buffer, "le", 8),
    ]);

    const hashedBuff = Buffer.from(sha256(buf), "hex");
    const bufWithPrefix = Buffer.concat([
      Buffer.from([0]),
      hashedBuff
    ]);

    return Buffer.from(sha256(bufWithPrefix), "hex");
  }

  getHexRoot(): string {
    return this._tree.getHexRoot();
  }

  // returns the hex bytes32 values of the proof
  getHexProof(account: web3.PublicKey, amountUnlocked: BN, amountLocked: BN): string[] {
    return this._tree.getHexProof(BalanceTree.toNode(account, amountUnlocked, amountLocked));
  }

  getRoot(): Buffer {
    return this._tree.getRoot();
  }

  getProof(account: web3.PublicKey, amountUnlocked: BN, amountLocked: BN): Buffer[] {
    return this._tree.getProof(BalanceTree.toNode(account, amountUnlocked, amountLocked));
  }
}
