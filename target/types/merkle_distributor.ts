export type MerkleDistributor = {
  "version": "0.1.0",
  "name": "merkle_distributor",
  "instructions": [
    {
      "name": "newParentAccount",
      "docs": [
        "ADMIN FUNCTIONS ////"
      ],
      "accounts": [
        {
          "name": "parentAccount",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "[ParentAccount]"
          ],
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "type": "string",
                "value": "ParentAccount"
              },
              {
                "kind": "account",
                "type": "publicKey",
                "account": "Mint",
                "path": "mint"
              }
            ]
          }
        },
        {
          "name": "parentVault",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "Parent vault",
            "Should create previously"
          ]
        },
        {
          "name": "mint",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "The mint to distribute."
          ]
        },
        {
          "name": "admin",
          "isMut": true,
          "isSigner": true,
          "docs": [
            "Admin wallet, responsible for creating the distributor and paying for the transaction.",
            "Also has the authority to set the clawback receiver and change itself."
          ]
        },
        {
          "name": "systemProgram",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "The [System] program."
          ]
        },
        {
          "name": "tokenProgram",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "The [Token] program."
          ]
        }
      ],
      "args": []
    },
    {
      "name": "newDistributor",
      "accounts": [
        {
          "name": "distributor",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "[MerkleDistributor]."
          ],
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "type": "string",
                "value": "MerkleDistributor"
              },
              {
                "kind": "account",
                "type": "publicKey",
                "path": "base"
              },
              {
                "kind": "account",
                "type": "publicKey",
                "account": "Mint",
                "path": "mint"
              },
              {
                "kind": "arg",
                "type": "u64",
                "path": "version"
              }
            ]
          }
        },
        {
          "name": "partialMerkleTree",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "[MerkleDistributor]."
          ],
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "type": "string",
                "value": "PartialMerkleTree"
              },
              {
                "kind": "account",
                "type": "publicKey",
                "path": "distributor"
              }
            ]
          }
        },
        {
          "name": "base",
          "isMut": false,
          "isSigner": true,
          "docs": [
            "Base key of the distributor."
          ]
        },
        {
          "name": "clawbackReceiver",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "Clawback receiver token account"
          ]
        },
        {
          "name": "mint",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "The mint to distribute."
          ]
        },
        {
          "name": "tokenVault",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "Token vault",
            "Should create previously"
          ]
        },
        {
          "name": "admin",
          "isMut": true,
          "isSigner": true,
          "docs": [
            "Admin wallet, responsible for creating the distributor and paying for the transaction.",
            "Also has the authority to set the clawback receiver and change itself."
          ]
        },
        {
          "name": "systemProgram",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "The [System] program."
          ]
        },
        {
          "name": "tokenProgram",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "The [Token] program."
          ]
        }
      ],
      "args": [
        {
          "name": "params",
          "type": {
            "defined": "NewDistributorParams"
          }
        }
      ]
    },
    {
      "name": "distributeVault",
      "accounts": [
        {
          "name": "parentAccount",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "The [ParentState]."
          ],
          "relations": [
            "admin"
          ]
        },
        {
          "name": "parentVault",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "Parent vault containing the tokens to distribute to distributor vault."
          ]
        },
        {
          "name": "admin",
          "isMut": false,
          "isSigner": true,
          "docs": [
            "Admin"
          ]
        },
        {
          "name": "tokenProgram",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "SPL [Token] program."
          ]
        }
      ],
      "args": []
    },
    {
      "name": "closeDistributor",
      "docs": [
        "only available in test phase"
      ],
      "accounts": [
        {
          "name": "distributor",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "[MerkleDistributor]."
          ],
          "relations": [
            "admin",
            "token_vault"
          ]
        },
        {
          "name": "tokenVault",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "Clawback receiver token account"
          ]
        },
        {
          "name": "admin",
          "isMut": true,
          "isSigner": true,
          "docs": [
            "Admin wallet, responsible for creating the distributor and paying for the transaction.",
            "Also has the authority to set the clawback receiver and change itself."
          ]
        },
        {
          "name": "destinationTokenAccount",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "account receive token back"
          ]
        },
        {
          "name": "tokenProgram",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "The [Token] program."
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
      "accounts": [
        {
          "name": "claimStatus",
          "isMut": true,
          "isSigner": false,
          "relations": [
            "claimant",
            "admin"
          ]
        },
        {
          "name": "claimant",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "admin",
          "isMut": false,
          "isSigner": true
        }
      ],
      "args": []
    },
    {
      "name": "setActivationPoint",
      "accounts": [
        {
          "name": "distributor",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "[MerkleDistributor]."
          ],
          "relations": [
            "admin"
          ]
        },
        {
          "name": "admin",
          "isMut": true,
          "isSigner": true,
          "docs": [
            "Payer to create the distributor."
          ]
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
      "name": "clawback",
      "accounts": [
        {
          "name": "distributor",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "The [MerkleDistributor]."
          ],
          "relations": [
            "clawback_receiver"
          ]
        },
        {
          "name": "from",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "Distributor ATA containing the tokens to distribute."
          ]
        },
        {
          "name": "clawbackReceiver",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "The Clawback token account."
          ]
        },
        {
          "name": "tokenProgram",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "SPL [Token] program."
          ]
        }
      ],
      "args": []
    },
    {
      "name": "setClawbackReceiver",
      "accounts": [
        {
          "name": "distributor",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "The [MerkleDistributor]."
          ],
          "relations": [
            "admin"
          ]
        },
        {
          "name": "newClawbackAccount",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "New clawback account"
          ]
        },
        {
          "name": "admin",
          "isMut": false,
          "isSigner": true,
          "docs": [
            "Admin signer"
          ]
        }
      ],
      "args": []
    },
    {
      "name": "setAdmin",
      "accounts": [
        {
          "name": "distributor",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "The [MerkleDistributor]."
          ],
          "relations": [
            "admin"
          ]
        },
        {
          "name": "admin",
          "isMut": false,
          "isSigner": true,
          "docs": [
            "Admin signer"
          ]
        },
        {
          "name": "newAdmin",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "New admin account"
          ]
        }
      ],
      "args": []
    },
    {
      "name": "setOperator",
      "accounts": [
        {
          "name": "distributor",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "The [MerkleDistributor]."
          ],
          "relations": [
            "admin"
          ]
        },
        {
          "name": "admin",
          "isMut": false,
          "isSigner": true,
          "docs": [
            "Admin signer"
          ]
        }
      ],
      "args": [
        {
          "name": "newOperator",
          "type": "publicKey"
        }
      ]
    },
    {
      "name": "newClaim",
      "docs": [
        "USER FUNCTIONS /////"
      ],
      "accounts": [
        {
          "name": "distributor",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "The [MerkleDistributor]."
          ]
        },
        {
          "name": "partialMerkleTree",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "The [PartialMerkleTree]."
          ],
          "relations": [
            "distributor"
          ]
        },
        {
          "name": "claimStatus",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "Claim status PDA"
          ],
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "type": "string",
                "value": "ClaimStatus"
              },
              {
                "kind": "account",
                "type": "publicKey",
                "path": "claimant"
              },
              {
                "kind": "account",
                "type": "publicKey",
                "path": "distributor"
              }
            ]
          }
        },
        {
          "name": "from",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "Distributor ATA containing the tokens to distribute."
          ]
        },
        {
          "name": "to",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "Account to send the claimed tokens to."
          ]
        },
        {
          "name": "claimant",
          "isMut": true,
          "isSigner": true,
          "docs": [
            "Who is claiming the tokens."
          ]
        },
        {
          "name": "operator",
          "isMut": false,
          "isSigner": true,
          "isOptional": true,
          "docs": [
            "operator"
          ]
        },
        {
          "name": "tokenProgram",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "SPL [Token] program."
          ]
        },
        {
          "name": "systemProgram",
          "isMut": false,
          "isSigner": false,
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
        },
        {
          "name": "initialIndex",
          "type": "u8"
        }
      ]
    },
    {
      "name": "claimLocked",
      "accounts": [
        {
          "name": "distributor",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "The [MerkleDistributor]."
          ]
        },
        {
          "name": "claimStatus",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "Claim Status PDA"
          ],
          "relations": [
            "distributor",
            "claimant"
          ]
        },
        {
          "name": "from",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "Distributor ATA containing the tokens to distribute."
          ]
        },
        {
          "name": "to",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "Account to send the claimed tokens to."
          ]
        },
        {
          "name": "claimant",
          "isMut": false,
          "isSigner": true,
          "docs": [
            "Who is claiming the tokens."
          ]
        },
        {
          "name": "operator",
          "isMut": false,
          "isSigner": true,
          "isOptional": true,
          "docs": [
            "operator"
          ]
        },
        {
          "name": "tokenProgram",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "SPL [Token] program."
          ]
        }
      ],
      "args": []
    },
    {
      "name": "newClaimAndStake",
      "accounts": [
        {
          "name": "distributor",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "The [MerkleDistributor]."
          ],
          "relations": [
            "locker"
          ]
        },
        {
          "name": "partialMerkleTree",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "The [PartialMerkleTree]."
          ],
          "relations": [
            "distributor"
          ]
        },
        {
          "name": "claimStatus",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "Claim status PDA"
          ],
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "type": "string",
                "value": "ClaimStatus"
              },
              {
                "kind": "account",
                "type": "publicKey",
                "path": "claimant"
              },
              {
                "kind": "account",
                "type": "publicKey",
                "path": "distributor"
              }
            ]
          }
        },
        {
          "name": "from",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "Distributor ATA containing the tokens to distribute."
          ]
        },
        {
          "name": "claimant",
          "isMut": true,
          "isSigner": true,
          "docs": [
            "Who is claiming the tokens."
          ]
        },
        {
          "name": "operator",
          "isMut": false,
          "isSigner": true,
          "isOptional": true,
          "docs": [
            "operator"
          ]
        },
        {
          "name": "tokenProgram",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "SPL [Token] program."
          ]
        },
        {
          "name": "systemProgram",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "The [System] program."
          ]
        },
        {
          "name": "voterProgram",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "Voter program"
          ]
        },
        {
          "name": "locker",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "escrow",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "escrowTokens",
          "isMut": true,
          "isSigner": false
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
        },
        {
          "name": "initialIndex",
          "type": "u8"
        }
      ]
    },
    {
      "name": "claimLockedAndStake",
      "accounts": [
        {
          "name": "distributor",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "The [MerkleDistributor]."
          ],
          "relations": [
            "locker"
          ]
        },
        {
          "name": "claimStatus",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "Claim Status PDA"
          ],
          "relations": [
            "distributor",
            "claimant"
          ]
        },
        {
          "name": "from",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "Distributor ATA containing the tokens to distribute."
          ]
        },
        {
          "name": "claimant",
          "isMut": false,
          "isSigner": true,
          "docs": [
            "Who is claiming the tokens."
          ]
        },
        {
          "name": "operator",
          "isMut": false,
          "isSigner": true,
          "isOptional": true,
          "docs": [
            "operator"
          ]
        },
        {
          "name": "tokenProgram",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "SPL [Token] program."
          ]
        },
        {
          "name": "voterProgram",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "Voter program"
          ]
        },
        {
          "name": "locker",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "escrow",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "escrowTokens",
          "isMut": true,
          "isSigner": false
        }
      ],
      "args": []
    }
  ],
  "accounts": [
    {
      "name": "claimStatus",
      "docs": [
        "Holds whether or not a claimant has claimed tokens."
      ],
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "admin",
            "docs": [
              "admin of merkle tree, store for for testing purpose"
            ],
            "type": "publicKey"
          },
          {
            "name": "distributor",
            "docs": [
              "distributor"
            ],
            "type": "publicKey"
          },
          {
            "name": "claimant",
            "docs": [
              "Authority that claimed the tokens."
            ],
            "type": "publicKey"
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
            "name": "bonusAmount",
            "docs": [
              "Bonus amount"
            ],
            "type": "u64"
          },
          {
            "name": "closable",
            "docs": [
              "indicate that whether admin can close this account, for testing purpose"
            ],
            "type": "u8"
          },
          {
            "name": "padding0",
            "docs": [
              "padding 0"
            ],
            "type": {
              "array": [
                "u8",
                7
              ]
            }
          },
          {
            "name": "padding1",
            "docs": [
              "padding 1"
            ],
            "type": "u64"
          },
          {
            "name": "buffer",
            "docs": [
              "buffer"
            ],
            "type": "u128"
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
            "name": "mint",
            "docs": [
              "[Mint] of the token to be distributed."
            ],
            "type": "publicKey"
          },
          {
            "name": "base",
            "docs": [
              "base key of distributor."
            ],
            "type": "publicKey"
          },
          {
            "name": "tokenVault",
            "docs": [
              "Token Address of the vault"
            ],
            "type": "publicKey"
          },
          {
            "name": "clawbackReceiver",
            "docs": [
              "Clawback receiver"
            ],
            "type": "publicKey"
          },
          {
            "name": "admin",
            "docs": [
              "Admin wallet"
            ],
            "type": "publicKey"
          },
          {
            "name": "locker",
            "docs": [
              "locker, for claim type claim and stake"
            ],
            "type": "publicKey"
          },
          {
            "name": "operator",
            "docs": [
              "operator for signing in permissioned merkle tree"
            ],
            "type": "publicKey"
          },
          {
            "name": "parentAccount",
            "docs": [
              "Parent account use to distribute fund to all distributor"
            ],
            "type": "publicKey"
          },
          {
            "name": "partialMerkleTree",
            "docs": [
              "Partial merkle tree account"
            ],
            "type": "publicKey"
          },
          {
            "name": "version",
            "docs": [
              "Version of the airdrop"
            ],
            "type": "u64"
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
            "name": "activationPoint",
            "docs": [
              "this merkle tree is activated from this slot or timestamp"
            ],
            "type": "u64"
          },
          {
            "name": "activationType",
            "docs": [
              "activation type, 0 means slot, 1 means timestamp"
            ],
            "type": "u8"
          },
          {
            "name": "claimType",
            "docs": [
              "claim type"
            ],
            "type": "u8"
          },
          {
            "name": "bump",
            "docs": [
              "Bump seed."
            ],
            "type": "u8"
          },
          {
            "name": "clawedBack",
            "docs": [
              "Whether or not the distributor has been clawed back"
            ],
            "type": "u8"
          },
          {
            "name": "closable",
            "docs": [
              "indicate that whether admin can close this pool, for testing purpose"
            ],
            "type": "u8"
          },
          {
            "name": "padding0",
            "docs": [
              "Padding 0"
            ],
            "type": {
              "array": [
                "u8",
                3
              ]
            }
          },
          {
            "name": "airdropBonus",
            "type": {
              "defined": "AirdropBonus"
            }
          },
          {
            "name": "padding2",
            "type": {
              "array": [
                "u8",
                8
              ]
            }
          },
          {
            "name": "buffer",
            "type": {
              "array": [
                "u128",
                5
              ]
            }
          }
        ]
      }
    },
    {
      "name": "parentAccount",
      "docs": [
        "Parent Account: Authority of parent vault use to distribute fund to all distributors"
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
            "name": "padding0",
            "docs": [
              "padding 0"
            ],
            "type": {
              "array": [
                "u8",
                7
              ]
            }
          },
          {
            "name": "admin",
            "docs": [
              "Admin of merkle tree, store for for testing purpose"
            ],
            "type": "publicKey"
          },
          {
            "name": "mint",
            "docs": [
              "Mint of the token to be distributed."
            ],
            "type": "publicKey"
          },
          {
            "name": "parentVault",
            "docs": [
              "Token Address of parent vault"
            ],
            "type": "publicKey"
          },
          {
            "name": "totalDistributed",
            "docs": [
              "Total value distributed to distributor vault"
            ],
            "type": "u64"
          },
          {
            "name": "buffer",
            "docs": [
              "Buffer for future use or alignment."
            ],
            "type": {
              "array": [
                "u128",
                5
              ]
            }
          }
        ]
      }
    },
    {
      "name": "partialMerkleTree",
      "docs": [
        "Holds whether or not a claimant has claimed tokens."
      ],
      "type": {
        "kind": "struct",
        "fields": [
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
            "name": "depth",
            "docs": [
              "The number of layers in the tree that are stored on-chain, excluding the root layer."
            ],
            "type": "u8"
          },
          {
            "name": "totalNodes",
            "docs": [
              "The total number of nodes stored on-chain. This is used for breadth-first search (BFS) traversal."
            ],
            "type": "u8"
          },
          {
            "name": "nodes",
            "docs": [
              "A vector of node hashes representing a subset of the Merkle tree, excluding the root.",
              "The length of this vector should be equal to `total_nodes`."
            ],
            "type": {
              "vec": {
                "array": [
                  "u8",
                  32
                ]
              }
            }
          },
          {
            "name": "distributor",
            "docs": [
              "The distributor associated with this Merkle tree"
            ],
            "type": "publicKey"
          },
          {
            "name": "buffer",
            "docs": [
              "Buffer"
            ],
            "type": {
              "array": [
                "u64",
                5
              ]
            }
          }
        ]
      }
    }
  ],
  "types": [
    {
      "name": "NewDistributorParams",
      "type": {
        "kind": "struct",
        "fields": [
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
            "name": "totalNodes",
            "type": "u8"
          },
          {
            "name": "depth",
            "type": "u8"
          },
          {
            "name": "nodes",
            "type": {
              "vec": {
                "array": [
                  "u8",
                  32
                ]
              }
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
          },
          {
            "name": "claimType",
            "type": "u8"
          },
          {
            "name": "operator",
            "type": "publicKey"
          },
          {
            "name": "locker",
            "type": "publicKey"
          },
          {
            "name": "parentAccount",
            "type": "publicKey"
          }
        ]
      }
    },
    {
      "name": "AirdropBonus",
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
      "name": "ActivationType",
      "docs": [
        "Type of the activation"
      ],
      "type": {
        "kind": "enum",
        "variants": [
          {
            "name": "Slot"
          },
          {
            "name": "Timestamp"
          }
        ]
      }
    },
    {
      "name": "ClaimType",
      "docs": [
        "Type of the activation"
      ],
      "type": {
        "kind": "enum",
        "variants": [
          {
            "name": "Permissionless"
          },
          {
            "name": "Permissioned"
          },
          {
            "name": "PermissionlessWithStaking"
          },
          {
            "name": "PermissionedWithStaking"
          }
        ]
      }
    }
  ],
  "events": [
    {
      "name": "NewClaimEvent",
      "fields": [
        {
          "name": "claimant",
          "type": "publicKey",
          "index": false
        },
        {
          "name": "timestamp",
          "type": "i64",
          "index": false
        }
      ]
    },
    {
      "name": "ClaimedEvent",
      "fields": [
        {
          "name": "claimant",
          "type": "publicKey",
          "index": false
        },
        {
          "name": "amount",
          "type": "u64",
          "index": false
        }
      ]
    }
  ],
  "errors": [
    {
      "code": 6000,
      "name": "InsufficientUnlockedTokens",
      "msg": "Insufficient unlocked tokens"
    },
    {
      "code": 6001,
      "name": "StartTooFarInFuture",
      "msg": "Deposit Start too far in future"
    },
    {
      "code": 6002,
      "name": "InvalidProof",
      "msg": "Invalid Merkle proof."
    },
    {
      "code": 6003,
      "name": "ExceededMaxClaim",
      "msg": "Exceeded maximum claim amount"
    },
    {
      "code": 6004,
      "name": "MaxNodesExceeded",
      "msg": "Exceeded maximum node count"
    },
    {
      "code": 6005,
      "name": "Unauthorized",
      "msg": "Account is not authorized to execute this instruction"
    },
    {
      "code": 6006,
      "name": "OwnerMismatch",
      "msg": "Token account owner did not match intended owner"
    },
    {
      "code": 6007,
      "name": "ClawbackDuringVesting",
      "msg": "Clawback cannot be before vesting ends"
    },
    {
      "code": 6008,
      "name": "ClawbackBeforeStart",
      "msg": "Attempted clawback before start"
    },
    {
      "code": 6009,
      "name": "ClawbackAlreadyClaimed",
      "msg": "Clawback already claimed"
    },
    {
      "code": 6010,
      "name": "InsufficientClawbackDelay",
      "msg": "Clawback start must be at least one day after vesting end"
    },
    {
      "code": 6011,
      "name": "SameClawbackReceiver",
      "msg": "New and old Clawback receivers are identical"
    },
    {
      "code": 6012,
      "name": "SameAdmin",
      "msg": "New and old admin are identical"
    },
    {
      "code": 6013,
      "name": "ClaimExpired",
      "msg": "Claim window expired"
    },
    {
      "code": 6014,
      "name": "ArithmeticError",
      "msg": "Arithmetic Error (overflow/underflow)"
    },
    {
      "code": 6015,
      "name": "StartTimestampAfterEnd",
      "msg": "Start Timestamp cannot be after end Timestamp"
    },
    {
      "code": 6016,
      "name": "TimestampsNotInFuture",
      "msg": "Timestamps cannot be in the past"
    },
    {
      "code": 6017,
      "name": "InvalidVersion",
      "msg": "Airdrop Version Mismatch"
    },
    {
      "code": 6018,
      "name": "ClaimingIsNotStarted",
      "msg": "Claiming is not started"
    },
    {
      "code": 6019,
      "name": "CannotCloseDistributor",
      "msg": "Cannot close distributor"
    },
    {
      "code": 6020,
      "name": "CannotCloseClaimStatus",
      "msg": "Cannot close claim status"
    },
    {
      "code": 6021,
      "name": "InvalidActivationType",
      "msg": "Invalid activation type"
    },
    {
      "code": 6022,
      "name": "TypeCastedError",
      "msg": "Type casted error"
    },
    {
      "code": 6023,
      "name": "InvalidOperator",
      "msg": "Invalid operator"
    },
    {
      "code": 6024,
      "name": "InvalidClaimType",
      "msg": "Invalid claim type"
    },
    {
      "code": 6025,
      "name": "SameOperator",
      "msg": "Same operator"
    },
    {
      "code": 6026,
      "name": "InvalidLocker",
      "msg": "Invalid locker"
    },
    {
      "code": 6027,
      "name": "EscrowIsNotMaxLock",
      "msg": "Escrow is not max lock"
    },
    {
      "code": 6028,
      "name": "InvalidRemainingAccounts",
      "msg": "Invalid remaining accounts"
    },
    {
      "code": 6029,
      "name": "InvalidAccount",
      "msg": "Invalid account"
    },
    {
      "code": 6030,
      "name": "TestLog",
      "msg": "testlog"
    }
  ]
};

export const IDL: MerkleDistributor = {
  "version": "0.1.0",
  "name": "merkle_distributor",
  "instructions": [
    {
      "name": "newParentAccount",
      "docs": [
        "ADMIN FUNCTIONS ////"
      ],
      "accounts": [
        {
          "name": "parentAccount",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "[ParentAccount]"
          ],
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "type": "string",
                "value": "ParentAccount"
              },
              {
                "kind": "account",
                "type": "publicKey",
                "account": "Mint",
                "path": "mint"
              }
            ]
          }
        },
        {
          "name": "parentVault",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "Parent vault",
            "Should create previously"
          ]
        },
        {
          "name": "mint",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "The mint to distribute."
          ]
        },
        {
          "name": "admin",
          "isMut": true,
          "isSigner": true,
          "docs": [
            "Admin wallet, responsible for creating the distributor and paying for the transaction.",
            "Also has the authority to set the clawback receiver and change itself."
          ]
        },
        {
          "name": "systemProgram",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "The [System] program."
          ]
        },
        {
          "name": "tokenProgram",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "The [Token] program."
          ]
        }
      ],
      "args": []
    },
    {
      "name": "newDistributor",
      "accounts": [
        {
          "name": "distributor",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "[MerkleDistributor]."
          ],
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "type": "string",
                "value": "MerkleDistributor"
              },
              {
                "kind": "account",
                "type": "publicKey",
                "path": "base"
              },
              {
                "kind": "account",
                "type": "publicKey",
                "account": "Mint",
                "path": "mint"
              },
              {
                "kind": "arg",
                "type": "u64",
                "path": "version"
              }
            ]
          }
        },
        {
          "name": "partialMerkleTree",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "[MerkleDistributor]."
          ],
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "type": "string",
                "value": "PartialMerkleTree"
              },
              {
                "kind": "account",
                "type": "publicKey",
                "path": "distributor"
              }
            ]
          }
        },
        {
          "name": "base",
          "isMut": false,
          "isSigner": true,
          "docs": [
            "Base key of the distributor."
          ]
        },
        {
          "name": "clawbackReceiver",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "Clawback receiver token account"
          ]
        },
        {
          "name": "mint",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "The mint to distribute."
          ]
        },
        {
          "name": "tokenVault",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "Token vault",
            "Should create previously"
          ]
        },
        {
          "name": "admin",
          "isMut": true,
          "isSigner": true,
          "docs": [
            "Admin wallet, responsible for creating the distributor and paying for the transaction.",
            "Also has the authority to set the clawback receiver and change itself."
          ]
        },
        {
          "name": "systemProgram",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "The [System] program."
          ]
        },
        {
          "name": "tokenProgram",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "The [Token] program."
          ]
        }
      ],
      "args": [
        {
          "name": "params",
          "type": {
            "defined": "NewDistributorParams"
          }
        }
      ]
    },
    {
      "name": "distributeVault",
      "accounts": [
        {
          "name": "parentAccount",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "The [ParentState]."
          ],
          "relations": [
            "admin"
          ]
        },
        {
          "name": "parentVault",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "Parent vault containing the tokens to distribute to distributor vault."
          ]
        },
        {
          "name": "admin",
          "isMut": false,
          "isSigner": true,
          "docs": [
            "Admin"
          ]
        },
        {
          "name": "tokenProgram",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "SPL [Token] program."
          ]
        }
      ],
      "args": []
    },
    {
      "name": "closeDistributor",
      "docs": [
        "only available in test phase"
      ],
      "accounts": [
        {
          "name": "distributor",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "[MerkleDistributor]."
          ],
          "relations": [
            "admin",
            "token_vault"
          ]
        },
        {
          "name": "tokenVault",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "Clawback receiver token account"
          ]
        },
        {
          "name": "admin",
          "isMut": true,
          "isSigner": true,
          "docs": [
            "Admin wallet, responsible for creating the distributor and paying for the transaction.",
            "Also has the authority to set the clawback receiver and change itself."
          ]
        },
        {
          "name": "destinationTokenAccount",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "account receive token back"
          ]
        },
        {
          "name": "tokenProgram",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "The [Token] program."
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
      "accounts": [
        {
          "name": "claimStatus",
          "isMut": true,
          "isSigner": false,
          "relations": [
            "claimant",
            "admin"
          ]
        },
        {
          "name": "claimant",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "admin",
          "isMut": false,
          "isSigner": true
        }
      ],
      "args": []
    },
    {
      "name": "setActivationPoint",
      "accounts": [
        {
          "name": "distributor",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "[MerkleDistributor]."
          ],
          "relations": [
            "admin"
          ]
        },
        {
          "name": "admin",
          "isMut": true,
          "isSigner": true,
          "docs": [
            "Payer to create the distributor."
          ]
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
      "name": "clawback",
      "accounts": [
        {
          "name": "distributor",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "The [MerkleDistributor]."
          ],
          "relations": [
            "clawback_receiver"
          ]
        },
        {
          "name": "from",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "Distributor ATA containing the tokens to distribute."
          ]
        },
        {
          "name": "clawbackReceiver",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "The Clawback token account."
          ]
        },
        {
          "name": "tokenProgram",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "SPL [Token] program."
          ]
        }
      ],
      "args": []
    },
    {
      "name": "setClawbackReceiver",
      "accounts": [
        {
          "name": "distributor",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "The [MerkleDistributor]."
          ],
          "relations": [
            "admin"
          ]
        },
        {
          "name": "newClawbackAccount",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "New clawback account"
          ]
        },
        {
          "name": "admin",
          "isMut": false,
          "isSigner": true,
          "docs": [
            "Admin signer"
          ]
        }
      ],
      "args": []
    },
    {
      "name": "setAdmin",
      "accounts": [
        {
          "name": "distributor",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "The [MerkleDistributor]."
          ],
          "relations": [
            "admin"
          ]
        },
        {
          "name": "admin",
          "isMut": false,
          "isSigner": true,
          "docs": [
            "Admin signer"
          ]
        },
        {
          "name": "newAdmin",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "New admin account"
          ]
        }
      ],
      "args": []
    },
    {
      "name": "setOperator",
      "accounts": [
        {
          "name": "distributor",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "The [MerkleDistributor]."
          ],
          "relations": [
            "admin"
          ]
        },
        {
          "name": "admin",
          "isMut": false,
          "isSigner": true,
          "docs": [
            "Admin signer"
          ]
        }
      ],
      "args": [
        {
          "name": "newOperator",
          "type": "publicKey"
        }
      ]
    },
    {
      "name": "newClaim",
      "docs": [
        "USER FUNCTIONS /////"
      ],
      "accounts": [
        {
          "name": "distributor",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "The [MerkleDistributor]."
          ]
        },
        {
          "name": "partialMerkleTree",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "The [PartialMerkleTree]."
          ],
          "relations": [
            "distributor"
          ]
        },
        {
          "name": "claimStatus",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "Claim status PDA"
          ],
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "type": "string",
                "value": "ClaimStatus"
              },
              {
                "kind": "account",
                "type": "publicKey",
                "path": "claimant"
              },
              {
                "kind": "account",
                "type": "publicKey",
                "path": "distributor"
              }
            ]
          }
        },
        {
          "name": "from",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "Distributor ATA containing the tokens to distribute."
          ]
        },
        {
          "name": "to",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "Account to send the claimed tokens to."
          ]
        },
        {
          "name": "claimant",
          "isMut": true,
          "isSigner": true,
          "docs": [
            "Who is claiming the tokens."
          ]
        },
        {
          "name": "operator",
          "isMut": false,
          "isSigner": true,
          "isOptional": true,
          "docs": [
            "operator"
          ]
        },
        {
          "name": "tokenProgram",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "SPL [Token] program."
          ]
        },
        {
          "name": "systemProgram",
          "isMut": false,
          "isSigner": false,
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
        },
        {
          "name": "initialIndex",
          "type": "u8"
        }
      ]
    },
    {
      "name": "claimLocked",
      "accounts": [
        {
          "name": "distributor",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "The [MerkleDistributor]."
          ]
        },
        {
          "name": "claimStatus",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "Claim Status PDA"
          ],
          "relations": [
            "distributor",
            "claimant"
          ]
        },
        {
          "name": "from",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "Distributor ATA containing the tokens to distribute."
          ]
        },
        {
          "name": "to",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "Account to send the claimed tokens to."
          ]
        },
        {
          "name": "claimant",
          "isMut": false,
          "isSigner": true,
          "docs": [
            "Who is claiming the tokens."
          ]
        },
        {
          "name": "operator",
          "isMut": false,
          "isSigner": true,
          "isOptional": true,
          "docs": [
            "operator"
          ]
        },
        {
          "name": "tokenProgram",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "SPL [Token] program."
          ]
        }
      ],
      "args": []
    },
    {
      "name": "newClaimAndStake",
      "accounts": [
        {
          "name": "distributor",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "The [MerkleDistributor]."
          ],
          "relations": [
            "locker"
          ]
        },
        {
          "name": "partialMerkleTree",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "The [PartialMerkleTree]."
          ],
          "relations": [
            "distributor"
          ]
        },
        {
          "name": "claimStatus",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "Claim status PDA"
          ],
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "type": "string",
                "value": "ClaimStatus"
              },
              {
                "kind": "account",
                "type": "publicKey",
                "path": "claimant"
              },
              {
                "kind": "account",
                "type": "publicKey",
                "path": "distributor"
              }
            ]
          }
        },
        {
          "name": "from",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "Distributor ATA containing the tokens to distribute."
          ]
        },
        {
          "name": "claimant",
          "isMut": true,
          "isSigner": true,
          "docs": [
            "Who is claiming the tokens."
          ]
        },
        {
          "name": "operator",
          "isMut": false,
          "isSigner": true,
          "isOptional": true,
          "docs": [
            "operator"
          ]
        },
        {
          "name": "tokenProgram",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "SPL [Token] program."
          ]
        },
        {
          "name": "systemProgram",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "The [System] program."
          ]
        },
        {
          "name": "voterProgram",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "Voter program"
          ]
        },
        {
          "name": "locker",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "escrow",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "escrowTokens",
          "isMut": true,
          "isSigner": false
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
        },
        {
          "name": "initialIndex",
          "type": "u8"
        }
      ]
    },
    {
      "name": "claimLockedAndStake",
      "accounts": [
        {
          "name": "distributor",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "The [MerkleDistributor]."
          ],
          "relations": [
            "locker"
          ]
        },
        {
          "name": "claimStatus",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "Claim Status PDA"
          ],
          "relations": [
            "distributor",
            "claimant"
          ]
        },
        {
          "name": "from",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "Distributor ATA containing the tokens to distribute."
          ]
        },
        {
          "name": "claimant",
          "isMut": false,
          "isSigner": true,
          "docs": [
            "Who is claiming the tokens."
          ]
        },
        {
          "name": "operator",
          "isMut": false,
          "isSigner": true,
          "isOptional": true,
          "docs": [
            "operator"
          ]
        },
        {
          "name": "tokenProgram",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "SPL [Token] program."
          ]
        },
        {
          "name": "voterProgram",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "Voter program"
          ]
        },
        {
          "name": "locker",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "escrow",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "escrowTokens",
          "isMut": true,
          "isSigner": false
        }
      ],
      "args": []
    }
  ],
  "accounts": [
    {
      "name": "claimStatus",
      "docs": [
        "Holds whether or not a claimant has claimed tokens."
      ],
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "admin",
            "docs": [
              "admin of merkle tree, store for for testing purpose"
            ],
            "type": "publicKey"
          },
          {
            "name": "distributor",
            "docs": [
              "distributor"
            ],
            "type": "publicKey"
          },
          {
            "name": "claimant",
            "docs": [
              "Authority that claimed the tokens."
            ],
            "type": "publicKey"
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
            "name": "bonusAmount",
            "docs": [
              "Bonus amount"
            ],
            "type": "u64"
          },
          {
            "name": "closable",
            "docs": [
              "indicate that whether admin can close this account, for testing purpose"
            ],
            "type": "u8"
          },
          {
            "name": "padding0",
            "docs": [
              "padding 0"
            ],
            "type": {
              "array": [
                "u8",
                7
              ]
            }
          },
          {
            "name": "padding1",
            "docs": [
              "padding 1"
            ],
            "type": "u64"
          },
          {
            "name": "buffer",
            "docs": [
              "buffer"
            ],
            "type": "u128"
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
            "name": "mint",
            "docs": [
              "[Mint] of the token to be distributed."
            ],
            "type": "publicKey"
          },
          {
            "name": "base",
            "docs": [
              "base key of distributor."
            ],
            "type": "publicKey"
          },
          {
            "name": "tokenVault",
            "docs": [
              "Token Address of the vault"
            ],
            "type": "publicKey"
          },
          {
            "name": "clawbackReceiver",
            "docs": [
              "Clawback receiver"
            ],
            "type": "publicKey"
          },
          {
            "name": "admin",
            "docs": [
              "Admin wallet"
            ],
            "type": "publicKey"
          },
          {
            "name": "locker",
            "docs": [
              "locker, for claim type claim and stake"
            ],
            "type": "publicKey"
          },
          {
            "name": "operator",
            "docs": [
              "operator for signing in permissioned merkle tree"
            ],
            "type": "publicKey"
          },
          {
            "name": "parentAccount",
            "docs": [
              "Parent account use to distribute fund to all distributor"
            ],
            "type": "publicKey"
          },
          {
            "name": "partialMerkleTree",
            "docs": [
              "Partial merkle tree account"
            ],
            "type": "publicKey"
          },
          {
            "name": "version",
            "docs": [
              "Version of the airdrop"
            ],
            "type": "u64"
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
            "name": "activationPoint",
            "docs": [
              "this merkle tree is activated from this slot or timestamp"
            ],
            "type": "u64"
          },
          {
            "name": "activationType",
            "docs": [
              "activation type, 0 means slot, 1 means timestamp"
            ],
            "type": "u8"
          },
          {
            "name": "claimType",
            "docs": [
              "claim type"
            ],
            "type": "u8"
          },
          {
            "name": "bump",
            "docs": [
              "Bump seed."
            ],
            "type": "u8"
          },
          {
            "name": "clawedBack",
            "docs": [
              "Whether or not the distributor has been clawed back"
            ],
            "type": "u8"
          },
          {
            "name": "closable",
            "docs": [
              "indicate that whether admin can close this pool, for testing purpose"
            ],
            "type": "u8"
          },
          {
            "name": "padding0",
            "docs": [
              "Padding 0"
            ],
            "type": {
              "array": [
                "u8",
                3
              ]
            }
          },
          {
            "name": "airdropBonus",
            "type": {
              "defined": "AirdropBonus"
            }
          },
          {
            "name": "padding2",
            "type": {
              "array": [
                "u8",
                8
              ]
            }
          },
          {
            "name": "buffer",
            "type": {
              "array": [
                "u128",
                5
              ]
            }
          }
        ]
      }
    },
    {
      "name": "parentAccount",
      "docs": [
        "Parent Account: Authority of parent vault use to distribute fund to all distributors"
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
            "name": "padding0",
            "docs": [
              "padding 0"
            ],
            "type": {
              "array": [
                "u8",
                7
              ]
            }
          },
          {
            "name": "admin",
            "docs": [
              "Admin of merkle tree, store for for testing purpose"
            ],
            "type": "publicKey"
          },
          {
            "name": "mint",
            "docs": [
              "Mint of the token to be distributed."
            ],
            "type": "publicKey"
          },
          {
            "name": "parentVault",
            "docs": [
              "Token Address of parent vault"
            ],
            "type": "publicKey"
          },
          {
            "name": "totalDistributed",
            "docs": [
              "Total value distributed to distributor vault"
            ],
            "type": "u64"
          },
          {
            "name": "buffer",
            "docs": [
              "Buffer for future use or alignment."
            ],
            "type": {
              "array": [
                "u128",
                5
              ]
            }
          }
        ]
      }
    },
    {
      "name": "partialMerkleTree",
      "docs": [
        "Holds whether or not a claimant has claimed tokens."
      ],
      "type": {
        "kind": "struct",
        "fields": [
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
            "name": "depth",
            "docs": [
              "The number of layers in the tree that are stored on-chain, excluding the root layer."
            ],
            "type": "u8"
          },
          {
            "name": "totalNodes",
            "docs": [
              "The total number of nodes stored on-chain. This is used for breadth-first search (BFS) traversal."
            ],
            "type": "u8"
          },
          {
            "name": "nodes",
            "docs": [
              "A vector of node hashes representing a subset of the Merkle tree, excluding the root.",
              "The length of this vector should be equal to `total_nodes`."
            ],
            "type": {
              "vec": {
                "array": [
                  "u8",
                  32
                ]
              }
            }
          },
          {
            "name": "distributor",
            "docs": [
              "The distributor associated with this Merkle tree"
            ],
            "type": "publicKey"
          },
          {
            "name": "buffer",
            "docs": [
              "Buffer"
            ],
            "type": {
              "array": [
                "u64",
                5
              ]
            }
          }
        ]
      }
    }
  ],
  "types": [
    {
      "name": "NewDistributorParams",
      "type": {
        "kind": "struct",
        "fields": [
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
            "name": "totalNodes",
            "type": "u8"
          },
          {
            "name": "depth",
            "type": "u8"
          },
          {
            "name": "nodes",
            "type": {
              "vec": {
                "array": [
                  "u8",
                  32
                ]
              }
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
          },
          {
            "name": "claimType",
            "type": "u8"
          },
          {
            "name": "operator",
            "type": "publicKey"
          },
          {
            "name": "locker",
            "type": "publicKey"
          },
          {
            "name": "parentAccount",
            "type": "publicKey"
          }
        ]
      }
    },
    {
      "name": "AirdropBonus",
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
      "name": "ActivationType",
      "docs": [
        "Type of the activation"
      ],
      "type": {
        "kind": "enum",
        "variants": [
          {
            "name": "Slot"
          },
          {
            "name": "Timestamp"
          }
        ]
      }
    },
    {
      "name": "ClaimType",
      "docs": [
        "Type of the activation"
      ],
      "type": {
        "kind": "enum",
        "variants": [
          {
            "name": "Permissionless"
          },
          {
            "name": "Permissioned"
          },
          {
            "name": "PermissionlessWithStaking"
          },
          {
            "name": "PermissionedWithStaking"
          }
        ]
      }
    }
  ],
  "events": [
    {
      "name": "NewClaimEvent",
      "fields": [
        {
          "name": "claimant",
          "type": "publicKey",
          "index": false
        },
        {
          "name": "timestamp",
          "type": "i64",
          "index": false
        }
      ]
    },
    {
      "name": "ClaimedEvent",
      "fields": [
        {
          "name": "claimant",
          "type": "publicKey",
          "index": false
        },
        {
          "name": "amount",
          "type": "u64",
          "index": false
        }
      ]
    }
  ],
  "errors": [
    {
      "code": 6000,
      "name": "InsufficientUnlockedTokens",
      "msg": "Insufficient unlocked tokens"
    },
    {
      "code": 6001,
      "name": "StartTooFarInFuture",
      "msg": "Deposit Start too far in future"
    },
    {
      "code": 6002,
      "name": "InvalidProof",
      "msg": "Invalid Merkle proof."
    },
    {
      "code": 6003,
      "name": "ExceededMaxClaim",
      "msg": "Exceeded maximum claim amount"
    },
    {
      "code": 6004,
      "name": "MaxNodesExceeded",
      "msg": "Exceeded maximum node count"
    },
    {
      "code": 6005,
      "name": "Unauthorized",
      "msg": "Account is not authorized to execute this instruction"
    },
    {
      "code": 6006,
      "name": "OwnerMismatch",
      "msg": "Token account owner did not match intended owner"
    },
    {
      "code": 6007,
      "name": "ClawbackDuringVesting",
      "msg": "Clawback cannot be before vesting ends"
    },
    {
      "code": 6008,
      "name": "ClawbackBeforeStart",
      "msg": "Attempted clawback before start"
    },
    {
      "code": 6009,
      "name": "ClawbackAlreadyClaimed",
      "msg": "Clawback already claimed"
    },
    {
      "code": 6010,
      "name": "InsufficientClawbackDelay",
      "msg": "Clawback start must be at least one day after vesting end"
    },
    {
      "code": 6011,
      "name": "SameClawbackReceiver",
      "msg": "New and old Clawback receivers are identical"
    },
    {
      "code": 6012,
      "name": "SameAdmin",
      "msg": "New and old admin are identical"
    },
    {
      "code": 6013,
      "name": "ClaimExpired",
      "msg": "Claim window expired"
    },
    {
      "code": 6014,
      "name": "ArithmeticError",
      "msg": "Arithmetic Error (overflow/underflow)"
    },
    {
      "code": 6015,
      "name": "StartTimestampAfterEnd",
      "msg": "Start Timestamp cannot be after end Timestamp"
    },
    {
      "code": 6016,
      "name": "TimestampsNotInFuture",
      "msg": "Timestamps cannot be in the past"
    },
    {
      "code": 6017,
      "name": "InvalidVersion",
      "msg": "Airdrop Version Mismatch"
    },
    {
      "code": 6018,
      "name": "ClaimingIsNotStarted",
      "msg": "Claiming is not started"
    },
    {
      "code": 6019,
      "name": "CannotCloseDistributor",
      "msg": "Cannot close distributor"
    },
    {
      "code": 6020,
      "name": "CannotCloseClaimStatus",
      "msg": "Cannot close claim status"
    },
    {
      "code": 6021,
      "name": "InvalidActivationType",
      "msg": "Invalid activation type"
    },
    {
      "code": 6022,
      "name": "TypeCastedError",
      "msg": "Type casted error"
    },
    {
      "code": 6023,
      "name": "InvalidOperator",
      "msg": "Invalid operator"
    },
    {
      "code": 6024,
      "name": "InvalidClaimType",
      "msg": "Invalid claim type"
    },
    {
      "code": 6025,
      "name": "SameOperator",
      "msg": "Same operator"
    },
    {
      "code": 6026,
      "name": "InvalidLocker",
      "msg": "Invalid locker"
    },
    {
      "code": 6027,
      "name": "EscrowIsNotMaxLock",
      "msg": "Escrow is not max lock"
    },
    {
      "code": 6028,
      "name": "InvalidRemainingAccounts",
      "msg": "Invalid remaining accounts"
    },
    {
      "code": 6029,
      "name": "InvalidAccount",
      "msg": "Invalid account"
    },
    {
      "code": 6030,
      "name": "TestLog",
      "msg": "testlog"
    }
  ]
};
