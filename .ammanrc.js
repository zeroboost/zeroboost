const splToken = require("@solana/spl-token");
const { web3 } = require("@coral-xyz/anchor");

module.exports = {
  validator: {
    killRunningValidators: true,
    accountsCluster: web3.clusterApiUrl("devnet"),
    programs: [
      {
        label: "Raydium CP Pool Program",
        programId: "CPMDWBwJDtYax9qW7AyRuVC19Cc4L4Vcy4n2BHAbHkCW",
        deployPath: "./.amman/targets/raydium_cp_swap.so",
      },
      {
        label: "Metaplex Metadata Program",
        programId: "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s",
        deployPath: "./.amman/targets/mpl_token_metadata.so",
      },
    ],
    accounts: [
      {
        label: "Pyth Solana USD feed",
        accountId: "J83w4HKfqxwcq3BEMMkPFSppX3gqekLyLJBexebFVkix",
      },
      {
        label: "Raydium Pool admin",
        accountId: "adMCyoCgfkg7bQiJ9aBJ59H3BXLY3r5LNLfPpQfMzBe",
      },
      {
        label: "Raydium Pool fee Receiver",
        accountId: "G11FKBRaAkHAKuLCgLM6K6NUc9rTjPAznRCjZifrTQe2",
      },
      {
        label: "Raydium Pool config",
        accountId: "9zSzfkYy6awexsHvmggeH36pfVUdDGyCcwmjT3AQPBj6",
      },
      {
        label: "Zeroboost Admin",
        accountId: "9meGAekj5fSks2oYbv5RmVoxUam5d9T1RaxPhofnHmV2"
      },
      {
        label: "Zeroboost Metadata Fee Reciever",
        accountId: "2nAn6RP1zbSNDgkmh3atTJZn84oKkLnDDDdbruBTu4Lz"
      }
    ],
    websocketUrl: "",
    commitment: "confirmed",
    ledgerDir: "./.amman/ledger",
    verifyFees: false,
    detached: false,
    resetLedger: true,
  },
  relay: {
    enabled: true,
    killlRunningRelay: true,
  },
  storage: {
    enabled: true,
    storageId: "mock-storage",
    clearOnStart: true,
  },
};
