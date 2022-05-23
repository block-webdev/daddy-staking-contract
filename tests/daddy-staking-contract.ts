import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { DaddyStakingContract } from "../target/types/daddy_staking_contract";
import { SystemProgram, SYSVAR_RENT_PUBKEY, Transaction, sendAndConfirmTransaction } from "@solana/web3.js";

const PublicKey = anchor.web3.PublicKey;

const GLOBAL_AUTHORITY_SEED = "global-authority-1";
const USER_POOL_SEED = "user-pool-1";


describe("daddy-staking-contract", () => {
  // Configure the client to use the local cluster.
  let provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.DaddyStakingContract as Program<DaddyStakingContract>;
  const user = anchor.web3.Keypair.generate();


  it("Is initialized!", async () => {
    // Add your test here.
    await provider.connection.confirmTransaction(
      await provider.connection.requestAirdrop(user.publicKey, 90000000000),
      "confirmed"
    );

    // let balance = await provider.connection.getBalance(user.publicKey);
    // console.log('11111111111', balance);

    const [globalAuthority, bump] = await PublicKey.findProgramAddress(
      [Buffer.from(GLOBAL_AUTHORITY_SEED)],
      program.programId
    );

    const randKey = anchor.web3.Keypair.generate();
    let [userPool, userBump] = await PublicKey.findProgramAddress(
      [Buffer.from(USER_POOL_SEED), randKey.publicKey.toBuffer()],
      program.programId
    );
    const tx = new Transaction().add(
      await program.methods.initialize(0).accounts({
        userPool: userPool,
        // globalAuthority: globalAuthority,
        owner: user.publicKey,
        systemProgram: SystemProgram.programId,
      }).instruction()
    );
    let txHash = await sendAndConfirmTransaction(provider.connection, tx, [user]);

    console.log("Your transaction signature", tx);
  });
});
