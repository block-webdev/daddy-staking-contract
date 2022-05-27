import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { DaddyStakingContract } from "../target/types/daddy_staking_contract";
import { SystemProgram, SYSVAR_RENT_PUBKEY, Transaction, sendAndConfirmTransaction } from "@solana/web3.js";
import { createMint, getOrCreateAssociatedTokenAccount, TOKEN_PROGRAM_ID, ASSOCIATED_TOKEN_PROGRAM_ID, mintTo, getAccount } from "@solana/spl-token";

const PublicKey = anchor.web3.PublicKey;

const GLOBAL_AUTHORITY_SEED = "global-authority-1";
const USER_POOL_SEED = "user-pool-1";

const delay = (delayInms) => {
  return new Promise((resolve) => {
    setTimeout(() => {
      resolve(1);
    }, delayInms);
  });
}

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

    const [globalAuthority, globalBump] = await PublicKey.findProgramAddress(
      [Buffer.from(GLOBAL_AUTHORITY_SEED)],
      program.programId
    );

    const randKey = anchor.web3.Keypair.generate();
    let [userPool, userBump] = await PublicKey.findProgramAddress(
      [randKey.publicKey.toBuffer()],
      program.programId
    );
    let tx = new Transaction().add(
      await program.methods.initialize().accounts({
        globalAuthority: globalAuthority,
        owner: user.publicKey,
        systemProgram: SystemProgram.programId,
      }).instruction()
    );
    let txHash = await sendAndConfirmTransaction(provider.connection, tx, [user]);

    tx = new Transaction().add(
      await program.methods.initUserPool(0).accounts({
        owner: user.publicKey,
        userPool: userPool,
        rand: randKey.publicKey,
        systemProgram: SystemProgram.programId,
      }).instruction()
    );

    txHash = await sendAndConfirmTransaction(provider.connection, tx, [user]);

    /////////// staking 

    let nft_token_mint = await createMint(
      provider.connection,
      user,
      user.publicKey,
      null,
      0
    );
    const tokenAccount = await getOrCreateAssociatedTokenAccount(
      provider.connection,
      user,
      nft_token_mint,
      user.publicKey
    )

    let signature = await mintTo(
      provider.connection,
      user,
      nft_token_mint,
      tokenAccount.address,
      user.publicKey,
      1000000000
    );

    let accountInfo = await getAccount(provider.connection, tokenAccount.address);
    // console.log('11111111', accountInfo);

    const destTokenAccount = await getOrCreateAssociatedTokenAccount(
      provider.connection,
      user,
      nft_token_mint,
      globalAuthority,
      true
    )

    tx = new Transaction();
    for (let i = 0; i < 10; i++) {
      tx = tx.add(
        await program.methods.stakeNft(globalBump, 0).accounts({
          owner: user.publicKey,
          userPool: userPool,
          globalAuthority: globalAuthority,
          nftMint: nft_token_mint,
          sourceNftAccount: tokenAccount.address,
          destNftAccount: destTokenAccount.address,
          systemProgram: SystemProgram.programId,
        }).instruction()
      );
    }

    txHash = await sendAndConfirmTransaction(provider.connection, tx, [user]);

    accountInfo = await getAccount(provider.connection, destTokenAccount.address);
    console.log('staking res : ', accountInfo);

    // claim
    await delay(10000);

    console.log('================================');

    tx = new Transaction().add(
      await program.methods.claimReward(globalBump).accounts({
        owner: user.publicKey,
        userPool: userPool,
        globalAuthority: globalAuthority,
        nftMint: nft_token_mint,
        sourceAccount: destTokenAccount.address,
        destAccount: tokenAccount.address,
        // tokenProgram: TOKEN_PROGRAM_ID,
      }).instruction()
    );

    txHash = await sendAndConfirmTransaction(provider.connection, tx, [user]);
    console.log('*****************************************');

    accountInfo = await getAccount(provider.connection, destTokenAccount.address);
    console.log('claim res : ', accountInfo);

    let poolData = await program.account.userPool.fetch(userPool);
    console.log('user pool data : ', poolData);
    let ss = poolData.rewardAmount.toNumber();
    console.log('1111111111111', ss);

    // unstake
    // tx = new Transaction().add(
    //   await program.methods.unstakeNft(globalBump).accounts({
    //     owner: user.publicKey,
    //     userPool: userPool,
    //     globalAuthority: globalAuthority,
    //     nftMint: nft_token_mint,
    //     sourceNftAccount: destTokenAccount.address,
    //     destNftAccount: tokenAccount.address,
    //     tokenProgram: TOKEN_PROGRAM_ID,
    //   }).instruction()
    // );

    // txHash = await sendAndConfirmTransaction(provider.connection, tx, [user]);

    // accountInfo = await getAccount(provider.connection, destTokenAccount.address);
    // console.log('unstaking res : ', accountInfo);


  });
});
