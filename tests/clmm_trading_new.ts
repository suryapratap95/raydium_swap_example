import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { ClmmTradingNew } from "../target/types/clmm_trading_new";
import {
  TOKEN_PROGRAM_ID,
  createMint,
  createAssociatedTokenAccount,
  mintTo,
} from "@solana/spl-token";
import { assert } from "chai";
import * as fs from 'fs';

async function main() {
  // Configure the client
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  // Connect to the program
  const program = anchor.workspace.ClmmTradingNew as Program<ClmmTradingNew>;
  console.log("Program ID:", program.programId.toString());

  let addresses: { [key: string]: string } = {
    programId: program.programId.toString()
  };

  try {
    // Create test tokens
    console.log("Creating test tokens...");
    const tokenMint0 = await createMint(
      provider.connection,
      provider.wallet.payer,
      provider.wallet.publicKey,
      provider.wallet.publicKey,
      9
    );
    console.log("Token 0 created:", tokenMint0.toString());
    addresses.tokenMint0 = tokenMint0.toString();

    const tokenMint1 = await createMint(
      provider.connection,
      provider.wallet.payer,
      provider.wallet.publicKey,
      provider.wallet.publicKey,
      9
    );
    console.log("Token 1 created:", tokenMint1.toString());
    addresses.tokenMint1 = tokenMint1.toString();

    // Create token accounts
    console.log("Creating token accounts...");
    const userTokenAccount0 = await createAssociatedTokenAccount(
      provider.connection,
      provider.wallet.payer,
      tokenMint0,
      provider.wallet.publicKey
    );
    console.log("Token Account 0:", userTokenAccount0.toString());
    addresses.userTokenAccount0 = userTokenAccount0.toString();

    const userTokenAccount1 = await createAssociatedTokenAccount(
      provider.connection,
      provider.wallet.payer,
      tokenMint1,
      provider.wallet.publicKey
    );
    console.log("Token Account 1:", userTokenAccount1.toString());
    addresses.userTokenAccount1 = userTokenAccount1.toString();

    // Mint tokens
    console.log("Minting tokens...");
    await mintTo(
      provider.connection,
      provider.wallet.payer,
      tokenMint0,
      userTokenAccount0,
      provider.wallet.publicKey,
      1000000000
    );
    console.log("Minted tokens to account 0");

    await mintTo(
      provider.connection,
      provider.wallet.payer,
      tokenMint1,
      userTokenAccount1,
      provider.wallet.publicKey,
      1000000000
    );
    console.log("Minted tokens to account 1");

    // Initialize pool
    console.log("Initializing pool...");
    const poolState = anchor.web3.Keypair.generate();
    console.log("Pool State address:", poolState.publicKey.toString());
    addresses.poolState = poolState.publicKey.toString();

    await program.methods
      .initializePool(
        new anchor.BN("1000000000000000000"),
        10
      )
      .accounts({
        authority: provider.wallet.publicKey,
        poolState: poolState.publicKey,
        tokenMint0: tokenMint0,
        tokenMint1: tokenMint1,
        systemProgram: anchor.web3.SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
      })
      .signers([poolState])
      .rpc();
    console.log("Pool initialized successfully");

    // Create and save pool vault addresses
    const poolVault0 = await createAssociatedTokenAccount(
      provider.connection,
      provider.wallet.payer,
      tokenMint0,
      poolState.publicKey
    );
    console.log("Pool vault 0 created:", poolVault0.toString());
    addresses.poolVault0 = poolVault0.toString();

    const poolVault1 = await createAssociatedTokenAccount(
      provider.connection,
      provider.wallet.payer,
      tokenMint1,
      poolState.publicKey
    );
    console.log("Pool vault 1 created:", poolVault1.toString());
    addresses.poolVault1 = poolVault1.toString();

    // Test swap
    console.log("Testing swap...");
    const swapTx = await program.methods
      .swap(
        new anchor.BN(1000000),
        new anchor.BN(990000),
        new anchor.BN("1100000000000000000"),
        true
      )
      .accounts({
        user: provider.wallet.publicKey,
        poolState: poolState.publicKey,
        userTokenAccount: userTokenAccount0,
        poolTokenVault: poolVault0,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .rpc();
    console.log("Swap executed successfully. Transaction:", swapTx);
    addresses.lastSwapTx = swapTx;

    // Save wallet address
    addresses.walletAddress = provider.wallet.publicKey.toString();

    // Save all addresses to file
    const timestamp = new Date().toISOString().replace(/:/g, '-');
    const filename = `devnet-addresses-${timestamp}.json`;
    fs.writeFileSync(
      filename,
      JSON.stringify(
        {
          timestamp: new Date().toISOString(),
          network: "devnet",
          addresses: addresses,
        },
        null,
        2
      )
    );
    console.log(`Addresses saved to ${filename}`);

  } catch (error) {
    console.error("Error:", error);
    if (error.logs) {
      console.log("Program logs:", error.logs);
    }
    
    // Save addresses even if there's an error
    try {
      const errorFilename = `devnet-addresses-error-${new Date().toISOString().replace(/:/g, '-')}.json`;
      fs.writeFileSync(
        errorFilename,
        JSON.stringify(
          {
            timestamp: new Date().toISOString(),
            network: "devnet",
            addresses: addresses,
            error: {
              message: error.message,
              logs: error.logs
            }
          },
          null,
          2
        )
      );
      console.log(`Error state saved to ${errorFilename}`);
    } catch (saveError) {
      console.error("Error saving error state:", saveError);
    }
  }
}

main().then(
  () => process.exit(),
  err => {
    console.error(err);
    process.exit(-1);
  }
);