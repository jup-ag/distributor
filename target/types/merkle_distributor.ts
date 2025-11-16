/**
 * Program IDL in camelCase format in order to be used in JS/TS.
 *
 * Note that this is only a type helper and is not the actual IDL. The original
 * IDL can be found at `target/idl/merkle_distributor.json`.
 */
export type MerkleDistributor = {
  "address": "DiSLRwcSFvtwvMWSs7ubBMvYRaYNYupa76ZSuYLe6D7j",
  "metadata": {
    "name": "merkleDistributor",
    "version": "0.1.0",
    "spec": "0.1.0",
    "description": "A Solana program for distributing tokens according to a Merkle root."
  },
  "instructions": [
    {
      "name": "anchorClaimLocked",
      "docs": [
        "Anchor-only fallback for the slow path."
      ],
      "discriminator": [
        57,
        128,
        21,
        141,
        148,
        218,
        149,
        22
      ],
      "accounts": [
        {
          "name": "distributor",
          "docs": [
            "The [MerkleDistributor]."
          ],
          "writable": true
        },
        {
          "name": "claimStatus",
          "docs": [
            "Claim Status PDA"
          ],
          "writable": true
        },
        {
          "name": "from",
          "docs": [
            "Distributor ATA containing the tokens to distribute."
          ],
          "writable": true
        },
        {
          "name": "to",
          "docs": [
            "Account to send the claimed tokens to.",
            "Claimant must sign the transaction and can only claim on behalf of themself"
          ],
          "writable": true
        },
        {
          "name": "claimant",
          "docs": [
            "Who is claiming the tokens."
          ],
          "writable": true,
          "signer": true
        },
        {
          "name": "tokenProgram",
          "docs": [
            "SPL [Token] program."
          ]
        }
      ],
      "args": []
    },
    {
      "name": "anchorNewClaim",
      "docs": [
        "Anchor-only version of `new_claim`"
      ],
      "discriminator": [
        107,
        242,
        91,
        229,
        172,
        185,
        145,
        183
      ],
      "accounts": [
        {
          "name": "distributor",
          "docs": [
            "The [MerkleDistributor]."
          ],
          "writable": true
        },
        {
          "name": "claimStatus",
          "docs": [
            "Claim status PDA"
          ],
          "writable": true
        },
        {
          "name": "from",
          "docs": [
            "Distributor ATA containing the tokens to distribute."
          ],
          "writable": true
        },
        {
          "name": "to",
          "docs": [
            "Account to send the claimed tokens to."
          ],
          "writable": true
        },
        {
          "name": "claimant",
          "docs": [
            "Who is claiming the tokens."
          ],
          "writable": true,
          "signer": true
        },
        {
          "name": "tokenProgram",
          "docs": [
            "SPL [Token] program."
          ]
        },
        {
          "name": "systemProgram",
          "docs": [
            "The [System] program."
          ]
        }
      ],
      "args": [
        {
          "name": "amountUnlocked",
          "type": "u64"
        },
        {
          "name": "amountLocked",
          "type": "u64"
        },
        {
          "name": "proof",
          "type": {
            "vec": {
              "array": [
                "u8",
                32
              ]
            }
          }
        }
      ]
    },
    {
      "name": "anchorNewDistributor",
      "docs": [
        "Anchor-only version of `new_distributor` that always routes through",
        "the regular handler"
      ],
      "discriminator": [
        167,
        138,
        6,
        103,
        1,
        159,
        225,
        186
      ],
      "accounts": [
        {
          "name": "distributor",
          "docs": [
            "[MerkleDistributor]."
          ],
          "writable": true
        },
        {
          "name": "base",
          "docs": [
            "Base key of the distributor."
          ],
          "signer": true
        },
        {
          "name": "clawbackReceiver",
          "docs": [
            "Clawback receiver token account"
          ],
          "writable": true
        },
        {
          "name": "mint",
          "docs": [
            "The mint to distribute."
          ]
        },
        {
          "name": "tokenVault",
          "docs": [
            "Token vault",
            "Should create previously"
          ]
        },
        {
          "name": "admin",
          "docs": [
            "Admin wallet, responsible for creating the distributor and paying for the transaction.",
            "Also has the authority to set the clawback receiver and change itself."
          ],
          "writable": true,
          "signer": true
        },
        {
          "name": "systemProgram",
          "docs": [
            "The [System] program."
          ]
        },
        {
          "name": "associatedTokenProgram",
          "docs": [
            "The [Associated Token] program."
          ]
        },
        {
          "name": "tokenProgram",
          "docs": [
            "The [Token] program."
          ]
        }
      ],
      "args": [
        {
          "name": "version",
          "type": "u64"
        },
        {
          "name": "root",
          "type": {
            "array": [
              "u8",
              32
            ]
          }
        },
        {
          "name": "maxTotalClaim",
          "type": "u64"
        },
        {
          "name": "maxNumNodes",
          "type": "u64"
        },
        {
          "name": "startVestingTs",
          "type": "i64"
        },
        {
          "name": "endVestingTs",
          "type": "i64"
        },
        {
          "name": "clawbackStartTs",
          "type": "i64"
        },
        {
          "name": "activationPoint",
          "type": "u64"
        },
        {
          "name": "activationType",
          "type": "u8"
        },
        {
          "name": "closable",
          "type": "bool"
        }
      ]
    },
    {
      "name": "claimLocked",
      "discriminator": [
        34,
        206,
        181,
        23,
        11,
        207,
        147,
        90
      ],
      "accounts": [
        {
          "name": "distributor",
          "docs": [
            "The [MerkleDistributor]."
          ],
          "writable": true
        },
        {
          "name": "claimStatus",
          "docs": [
            "Claim Status PDA"
          ],
          "writable": true
        },
        {
          "name": "from",
          "docs": [
            "Distributor ATA containing the tokens to distribute."
          ],
          "writable": true
        },
        {
          "name": "to",
          "docs": [
            "Account to send the claimed tokens to.",
            "Claimant must sign the transaction and can only claim on behalf of themself"
          ],
          "writable": true
        },
        {
          "name": "claimant",
          "docs": [
            "Who is claiming the tokens."
          ],
          "writable": true,
          "signer": true
        },
        {
          "name": "tokenProgram",
          "docs": [
            "SPL [Token] program."
          ]
        }
      ],
      "args": []
    },
    {
      "name": "clawback",
      "discriminator": [
        111,
        92,
        142,
        79,
        33,
        234,
        82,
        27
      ],
      "accounts": [
        {
          "name": "distributor",
          "docs": [
            "The [MerkleDistributor]."
          ],
          "writable": true
        },
        {
          "name": "from",
          "docs": [
            "Distributor ATA containing the tokens to distribute."
          ],
          "writable": true
        },
        {
          "name": "to",
          "docs": [
            "The Clawback token account."
          ],
          "writable": true
        },
        {
          "name": "claimant",
          "docs": [
            "Claimant account",
            "Anyone can claw back the funds"
          ],
          "signer": true
        },
        {
          "name": "systemProgram",
          "docs": [
            "The [System] program."
          ]
        },
        {
          "name": "tokenProgram",
          "docs": [
            "SPL [Token] program."
          ]
        }
      ],
      "args": []
    },
    {
      "name": "closeClaimStatus",
      "docs": [
        "only available in test phase"
      ],
      "discriminator": [
        163,
        214,
        191,
        165,
        245,
        188,
        17,
        185
      ],
      "accounts": [
        {
          "name": "claimStatus",
          "writable": true
        },
        {
          "name": "claimant",
          "writable": true
        },
        {
          "name": "admin",
          "signer": true
        }
      ],
      "args": []
    },
    {
      "name": "closeDistributor",
      "docs": [
        "only available in test phase"
      ],
      "discriminator": [
        202,
        56,
        180,
        143,
        46,
        104,
        106,
        112
      ],
      "accounts": [
        {
          "name": "distributor",
          "docs": [
            "[MerkleDistributor]."
          ],
          "writable": true
        },
        {
          "name": "tokenVault",
          "docs": [
            "Clawback receiver token account"
          ],
          "writable": true
        },
        {
          "name": "admin",
          "docs": [
            "Admin wallet, responsible for creating the distributor and paying for the transaction.",
            "Also has the authority to set the clawback receiver and change itself."
          ],
          "writable": true,
          "signer": true
        },
        {
          "name": "destinationTokenAccount",
          "docs": [
            "account receive token back"
          ],
          "writable": true
        },
        {
          "name": "tokenProgram",
          "docs": [
            "The [Token] program."
          ]
        }
      ],
      "args": []
    },
    {
      "name": "newClaim",
      "discriminator": [
        78,
        177,
        98,
        123,
        210,
        21,
        187,
        83
      ],
      "accounts": [
        {
          "name": "distributor",
          "docs": [
            "The [MerkleDistributor]."
          ],
          "writable": true
        },
        {
          "name": "claimStatus",
          "docs": [
            "Claim status PDA"
          ],
          "writable": true
        },
        {
          "name": "from",
          "docs": [
            "Distributor ATA containing the tokens to distribute."
          ],
          "writable": true
        },
        {
          "name": "to",
          "docs": [
            "Account to send the claimed tokens to."
          ],
          "writable": true
        },
        {
          "name": "claimant",
          "docs": [
            "Who is claiming the tokens."
          ],
          "writable": true,
          "signer": true
        },
        {
          "name": "tokenProgram",
          "docs": [
            "SPL [Token] program."
          ]
        },
        {
          "name": "systemProgram",
          "docs": [
            "The [System] program."
          ]
        }
      ],
      "args": [
        {
          "name": "amountUnlocked",
          "type": "u64"
        },
        {
          "name": "amountLocked",
          "type": "u64"
        },
        {
          "name": "proof",
          "type": {
            "vec": {
              "array": [
                "u8",
                32
              ]
            }
          }
        }
      ]
    },
    {
      "name": "newDistributor",
      "discriminator": [
        32,
        139,
        112,
        171,
        0,
        2,
        225,
        155
      ],
      "accounts": [
        {
          "name": "distributor",
          "docs": [
            "[MerkleDistributor]."
          ],
          "writable": true
        },
        {
          "name": "base",
          "docs": [
            "Base key of the distributor."
          ],
          "signer": true
        },
        {
          "name": "clawbackReceiver",
          "docs": [
            "Clawback receiver token account"
          ],
          "writable": true
        },
        {
          "name": "mint",
          "docs": [
            "The mint to distribute."
          ]
        },
        {
          "name": "tokenVault",
          "docs": [
            "Token vault",
            "Should create previously"
          ]
        },
        {
          "name": "admin",
          "docs": [
            "Admin wallet, responsible for creating the distributor and paying for the transaction.",
            "Also has the authority to set the clawback receiver and change itself."
          ],
          "writable": true,
          "signer": true
        },
        {
          "name": "systemProgram",
          "docs": [
            "The [System] program."
          ]
        },
        {
          "name": "associatedTokenProgram",
          "docs": [
            "The [Associated Token] program."
          ]
        },
        {
          "name": "tokenProgram",
          "docs": [
            "The [Token] program."
          ]
        }
      ],
      "args": [
        {
          "name": "version",
          "type": "u64"
        },
        {
          "name": "root",
          "type": {
            "array": [
              "u8",
              32
            ]
          }
        },
        {
          "name": "maxTotalClaim",
          "type": "u64"
        },
        {
          "name": "maxNumNodes",
          "type": "u64"
        },
        {
          "name": "startVestingTs",
          "type": "i64"
        },
        {
          "name": "endVestingTs",
          "type": "i64"
        },
        {
          "name": "clawbackStartTs",
          "type": "i64"
        },
        {
          "name": "activationPoint",
          "type": "u64"
        },
        {
          "name": "activationType",
          "type": "u8"
        },
        {
          "name": "closable",
          "type": "bool"
        }
      ]
    },
    {
      "name": "newDistributor2",
      "discriminator": [
        69,
        91,
        34,
        247,
        112,
        178,
        21,
        251
      ],
      "accounts": [
        {
          "name": "distributor",
          "docs": [
            "[MerkleDistributor]."
          ],
          "writable": true
        },
        {
          "name": "base",
          "docs": [
            "Base key of the distributor."
          ],
          "signer": true
        },
        {
          "name": "clawbackReceiver",
          "docs": [
            "Clawback receiver token account"
          ],
          "writable": true
        },
        {
          "name": "mint",
          "docs": [
            "The mint to distribute."
          ]
        },
        {
          "name": "tokenVault",
          "docs": [
            "Token vault",
            "Should create previously"
          ]
        },
        {
          "name": "admin",
          "docs": [
            "Admin wallet, responsible for creating the distributor and paying for the transaction.",
            "Also has the authority to set the clawback receiver and change itself."
          ],
          "writable": true,
          "signer": true
        },
        {
          "name": "systemProgram",
          "docs": [
            "The [System] program."
          ]
        },
        {
          "name": "associatedTokenProgram",
          "docs": [
            "The [Associated Token] program."
          ]
        },
        {
          "name": "tokenProgram",
          "docs": [
            "The [Token] program."
          ]
        }
      ],
      "args": [
        {
          "name": "version",
          "type": "u64"
        },
        {
          "name": "root",
          "type": {
            "array": [
              "u8",
              32
            ]
          }
        },
        {
          "name": "totalClaim",
          "type": "u64"
        },
        {
          "name": "maxNumNodes",
          "type": "u64"
        },
        {
          "name": "startVestingTs",
          "type": "i64"
        },
        {
          "name": "endVestingTs",
          "type": "i64"
        },
        {
          "name": "clawbackStartTs",
          "type": "i64"
        },
        {
          "name": "activationPoint",
          "type": "u64"
        },
        {
          "name": "activationType",
          "type": "u8"
        },
        {
          "name": "closable",
          "type": "bool"
        },
        {
          "name": "totalBonus",
          "type": "u64"
        },
        {
          "name": "bonusVestingDuration",
          "type": "u64"
        }
      ]
    },
    {
      "name": "setActivationPoint",
      "discriminator": [
        91,
        249,
        15,
        165,
        26,
        129,
        254,
        125
      ],
      "accounts": [
        {
          "name": "distributor",
          "docs": [
            "[MerkleDistributor]."
          ],
          "writable": true
        },
        {
          "name": "admin",
          "docs": [
            "Payer to create the distributor."
          ],
          "writable": true,
          "signer": true
        }
      ],
      "args": [
        {
          "name": "activationPoint",
          "type": "u64"
        }
      ]
    },
    {
      "name": "setAdmin",
      "discriminator": [
        251,
        163,
        0,
        52,
        91,
        194,
        187,
        92
      ],
      "accounts": [
        {
          "name": "distributor",
          "docs": [
            "The [MerkleDistributor]."
          ],
          "writable": true
        },
        {
          "name": "admin",
          "docs": [
            "Admin signer"
          ],
          "writable": true,
          "signer": true
        },
        {
          "name": "newAdmin",
          "docs": [
            "New admin account"
          ],
          "writable": true
        }
      ],
      "args": []
    },
    {
      "name": "setClawbackReceiver",
      "discriminator": [
        153,
        217,
        34,
        20,
        19,
        29,
        229,
        75
      ],
      "accounts": [
        {
          "name": "distributor",
          "docs": [
            "The [MerkleDistributor]."
          ],
          "writable": true
        },
        {
          "name": "newClawbackAccount",
          "docs": [
            "New clawback account"
          ]
        },
        {
          "name": "admin",
          "docs": [
            "Admin signer"
          ],
          "writable": true,
          "signer": true
        }
      ],
      "args": []
    }
  ],
  "accounts": [
    {
      "name": "claimStatus",
      "discriminator": [
        22,
        183,
        249,
        157,
        247,
        95,
        150,
        96
      ]
    },
    {
      "name": "merkleDistributor",
      "discriminator": [
        77,
        119,
        139,
        70,
        84,
        247,
        12,
        26
      ]
    }
  ],
  "events": [
    {
      "name": "claimedEvent",
      "discriminator": [
        144,
        172,
        209,
        86,
        144,
        87,
        84,
        115
      ]
    },
    {
      "name": "newClaimEvent",
      "discriminator": [
        244,
        3,
        231,
        151,
        60,
        101,
        55,
        55
      ]
    }
  ],
  "errors": [
    {
      "code": 6000,
      "name": "insufficientUnlockedTokens",
      "msg": "Insufficient unlocked tokens"
    },
    {
      "code": 6001,
      "name": "startTooFarInFuture",
      "msg": "Deposit Start too far in future"
    },
    {
      "code": 6002,
      "name": "invalidProof",
      "msg": "Invalid Merkle proof."
    },
    {
      "code": 6003,
      "name": "exceededMaxClaim",
      "msg": "Exceeded maximum claim amount"
    },
    {
      "code": 6004,
      "name": "maxNodesExceeded",
      "msg": "Exceeded maximum node count"
    },
    {
      "code": 6005,
      "name": "unauthorized",
      "msg": "Account is not authorized to execute this instruction"
    },
    {
      "code": 6006,
      "name": "ownerMismatch",
      "msg": "Token account owner did not match intended owner"
    },
    {
      "code": 6007,
      "name": "clawbackDuringVesting",
      "msg": "Clawback cannot be before vesting ends"
    },
    {
      "code": 6008,
      "name": "clawbackBeforeStart",
      "msg": "Attempted clawback before start"
    },
    {
      "code": 6009,
      "name": "clawbackAlreadyClaimed",
      "msg": "Clawback already claimed"
    },
    {
      "code": 6010,
      "name": "insufficientClawbackDelay",
      "msg": "Clawback start must be at least one day after vesting end"
    },
    {
      "code": 6011,
      "name": "sameClawbackReceiver",
      "msg": "New and old Clawback receivers are identical"
    },
    {
      "code": 6012,
      "name": "sameAdmin",
      "msg": "New and old admin are identical"
    },
    {
      "code": 6013,
      "name": "claimExpired",
      "msg": "Claim window expired"
    },
    {
      "code": 6014,
      "name": "arithmeticError",
      "msg": "Arithmetic Error (overflow/underflow)"
    },
    {
      "code": 6015,
      "name": "startTimestampAfterEnd",
      "msg": "Start Timestamp cannot be after end Timestamp"
    },
    {
      "code": 6016,
      "name": "timestampsNotInFuture",
      "msg": "Timestamps cannot be in the past"
    },
    {
      "code": 6017,
      "name": "invalidVersion",
      "msg": "Airdrop Version Mismatch"
    },
    {
      "code": 6018,
      "name": "claimingIsNotStarted",
      "msg": "Claiming is not started"
    },
    {
      "code": 6019,
      "name": "cannotCloseDistributor",
      "msg": "Cannot close distributor"
    },
    {
      "code": 6020,
      "name": "cannotCloseClaimStatus",
      "msg": "Cannot close claim status"
    },
    {
      "code": 6021,
      "name": "invalidActivationType",
      "msg": "Invalid activation type"
    }
  ],
  "types": [
    {
      "name": "airdropBonus",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "totalBonus",
            "docs": [
              "total bonus"
            ],
            "type": "u64"
          },
          {
            "name": "vestingDuration",
            "type": "u64"
          },
          {
            "name": "totalClaimedBonus",
            "docs": [
              "total bonus"
            ],
            "type": "u64"
          }
        ]
      }
    },
    {
      "name": "claimStatus",
      "docs": [
        "Holds whether or not a claimant has claimed tokens."
      ],
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "claimant",
            "docs": [
              "Authority that claimed the tokens."
            ],
            "type": "pubkey"
          },
          {
            "name": "lockedAmount",
            "docs": [
              "Locked amount"
            ],
            "type": "u64"
          },
          {
            "name": "lockedAmountWithdrawn",
            "docs": [
              "Locked amount withdrawn"
            ],
            "type": "u64"
          },
          {
            "name": "unlockedAmount",
            "docs": [
              "Unlocked amount"
            ],
            "type": "u64"
          },
          {
            "name": "closable",
            "docs": [
              "indicate that whether admin can close this account, for testing purpose"
            ],
            "type": "bool"
          },
          {
            "name": "admin",
            "docs": [
              "admin of merkle tree, store for for testing purpose"
            ],
            "type": "pubkey"
          }
        ]
      }
    },
    {
      "name": "claimedEvent",
      "docs": [
        "Emitted when tokens are claimed."
      ],
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "claimant",
            "docs": [
              "User that claimed."
            ],
            "type": "pubkey"
          },
          {
            "name": "amount",
            "docs": [
              "Amount of tokens to distribute."
            ],
            "type": "u64"
          }
        ]
      }
    },
    {
      "name": "merkleDistributor",
      "docs": [
        "State for the account which distributes tokens."
      ],
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "bump",
            "docs": [
              "Bump seed."
            ],
            "type": "u8"
          },
          {
            "name": "version",
            "docs": [
              "Version of the airdrop"
            ],
            "type": "u64"
          },
          {
            "name": "root",
            "docs": [
              "The 256-bit merkle root."
            ],
            "type": {
              "array": [
                "u8",
                32
              ]
            }
          },
          {
            "name": "mint",
            "docs": [
              "[Mint] of the token to be distributed."
            ],
            "type": "pubkey"
          },
          {
            "name": "base",
            "docs": [
              "base key of distributor."
            ],
            "type": "pubkey"
          },
          {
            "name": "tokenVault",
            "docs": [
              "Token Address of the vault"
            ],
            "type": "pubkey"
          },
          {
            "name": "maxTotalClaim",
            "docs": [
              "Maximum number of tokens that can ever be claimed from this [MerkleDistributor]."
            ],
            "type": "u64"
          },
          {
            "name": "maxNumNodes",
            "docs": [
              "Maximum number of nodes in [MerkleDistributor]."
            ],
            "type": "u64"
          },
          {
            "name": "totalAmountClaimed",
            "docs": [
              "Total amount of tokens that have been claimed."
            ],
            "type": "u64"
          },
          {
            "name": "numNodesClaimed",
            "docs": [
              "Number of nodes that have been claimed."
            ],
            "type": "u64"
          },
          {
            "name": "startTs",
            "docs": [
              "Lockup time start (Unix Timestamp)"
            ],
            "type": "i64"
          },
          {
            "name": "endTs",
            "docs": [
              "Lockup time end (Unix Timestamp)"
            ],
            "type": "i64"
          },
          {
            "name": "clawbackStartTs",
            "docs": [
              "Clawback start (Unix Timestamp)"
            ],
            "type": "i64"
          },
          {
            "name": "clawbackReceiver",
            "docs": [
              "Clawback receiver"
            ],
            "type": "pubkey"
          },
          {
            "name": "admin",
            "docs": [
              "Admin wallet"
            ],
            "type": "pubkey"
          },
          {
            "name": "clawedBack",
            "docs": [
              "Whether or not the distributor has been clawed back"
            ],
            "type": "bool"
          },
          {
            "name": "activationPoint",
            "docs": [
              "this merkle tree is activated from this slot or timestamp"
            ],
            "type": "u64"
          },
          {
            "name": "closable",
            "docs": [
              "indicate that whether admin can close this pool, for testing purpose"
            ],
            "type": "bool"
          },
          {
            "name": "airdropBonus",
            "docs": [
              "bonus multiplier"
            ],
            "type": {
              "defined": {
                "name": "airdropBonus"
              }
            }
          },
          {
            "name": "activationType",
            "docs": [
              "activation type, 0 means slot, 1 means timestamp"
            ],
            "type": "u8"
          },
          {
            "name": "buffer0",
            "docs": [
              "Buffer 0"
            ],
            "type": {
              "array": [
                "u8",
                7
              ]
            }
          },
          {
            "name": "buffer1",
            "docs": [
              "Buffer 1"
            ],
            "type": {
              "array": [
                "u8",
                32
              ]
            }
          },
          {
            "name": "buffer2",
            "docs": [
              "Buffer 2"
            ],
            "type": {
              "array": [
                "u8",
                32
              ]
            }
          }
        ]
      }
    },
    {
      "name": "newClaimEvent",
      "docs": [
        "Emitted when a new claim is created."
      ],
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "claimant",
            "docs": [
              "User that claimed."
            ],
            "type": "pubkey"
          },
          {
            "name": "timestamp",
            "docs": [
              "Timestamp."
            ],
            "type": "i64"
          }
        ]
      }
    }
  ]
};
