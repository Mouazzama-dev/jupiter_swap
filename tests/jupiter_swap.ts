import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { PublicKey, Keypair, SystemProgram, LAMPORTS_PER_SOL } from "@solana/web3.js";
import { assert } from "chai";
import { JupiterSwap } from "../target/types/jupiter_swap";

describe("jupiter_swap", () => {
  // Set up the client and provider
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const program = anchor.workspace.JupiterSwap as Program<JupiterSwap>;

  // Constants
  const AUTHORITY_SEED = Buffer.from("authority");
  const WSOL_SEED = Buffer.from("wsol");

  let user: Keypair;
  let solMint: PublicKey;
  let programAuthority: PublicKey;
  let programWsolAccount: PublicKey;

  before(async () => {
    // Generate user and add some SOL
    user = Keypair.generate();
    await provider.connection.requestAirdrop(user.publicKey, 1 * LAMPORTS_PER_SOL);

    // Fetch the SOL mint address (native mint)
    solMint = anchor.web3.SYSVAR_RENT_PUBKEY;

    // Derive the program authority and wSOL account PDA
    [programAuthority] = await PublicKey.findProgramAddress(
      [AUTHORITY_SEED],
      program.programId
    );
    [programWsolAccount] = await PublicKey.findProgramAddress(
      [WSOL_SEED],
      program.programId
    );
  });

  it("Swaps SOL into three different meme tokens via Jupiter", async () => {
    // Set up the swap data (e.g., route and instructions for Jupiter)
    const swapData = Buffer.from([]); // Placeholder; real swap data would come from Jupiter API

    // Set up remaining accounts for meme token mints and user token accounts
    const memeTokenMints = [
      new PublicKey("MemeTokenMint1PublicKey"), // Replace with real mint addresses
      new PublicKey("MemeTokenMint2PublicKey"),
      new PublicKey("MemeTokenMint3PublicKey"),
    ];
    const userTokenAccounts = [
      new PublicKey("UserTokenAccount1PublicKey"), // Replace with associated accounts for user
      new PublicKey("UserTokenAccount2PublicKey"),
      new PublicKey("UserTokenAccount3PublicKey"),
    ];
    const remainingAccounts = [...memeTokenMints, ...userTokenAccounts].map((pubkey) => ({
      pubkey,
      isSigner: false,
      isWritable: true,
    }));

    // Call the swap_sol_to_memes method
    try {
      await program.methods
        .swapSolToMemes(swapData)
        .accounts({
          programAuthority,
          programWsolAccount,
          userAccount: user.publicKey,
          solMint,
          jupiterProgram: new PublicKey("JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4"), // Jupiter Aggregator ID
          tokenProgram: anchor.spl.token.TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .remainingAccounts(remainingAccounts)
        .signers([user])
        .rpc({ skipPreflight: false });

      console.log("Swap successful!");
    } catch (error) {
      console.error("Error during swap:", error);
    }
  });
});
