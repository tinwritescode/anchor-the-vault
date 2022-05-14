import * as anchor from '@project-serum/anchor'
import { Program } from '@project-serum/anchor'
import { PlatsSolana } from '../target/types/plats_solana'
import * as spl from '@solana/spl-token'
import { assert } from 'chai'

const SAMPLE_PRIZE = new anchor.BN(1000)

interface PDAParameters {
  taskVaultAccount: anchor.web3.PublicKey
  taskVaultBump: number
  taskVaultTreasurer: anchor.web3.PublicKey
  taskVaultTokenAccount: anchor.web3.PublicKey
}

describe('plats-solana', () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env()
  anchor.setProvider(provider)

  const program = anchor.workspace.PlatsSolana as Program<PlatsSolana>

  let mintAddress: anchor.web3.PublicKey
  let pda: PDAParameters
  let [alice, aliceWallet]: [anchor.web3.Keypair, anchor.web3.PublicKey] = [
    null,
    null,
  ]
  let [bob]: [anchor.web3.Keypair] = [null]

  const createMint = async (
    connection: anchor.web3.Connection,
  ): Promise<anchor.web3.PublicKey> => {
    const tokenMint = new anchor.web3.Keypair()
    const lamportsForMint = await provider.connection.getMinimumBalanceForRentExemption(
      spl.MintLayout.span,
    )
    let tx = new anchor.web3.Transaction()

    // Allocate mint
    tx.add(
      anchor.web3.SystemProgram.createAccount({
        programId: spl.TOKEN_PROGRAM_ID,
        space: spl.MintLayout.span,
        fromPubkey: provider.wallet.publicKey,
        newAccountPubkey: tokenMint.publicKey,
        lamports: lamportsForMint,
      }),
    )
    // Allocate wallet account
    tx.add(
      spl.createInitializeMintInstruction(
        tokenMint.publicKey,
        6,
        provider.wallet.publicKey,
        provider.wallet.publicKey,
        spl.TOKEN_PROGRAM_ID,
      ),
    )
    const signature = await provider.sendAndConfirm(tx, [tokenMint])

    console.log(
      `[${tokenMint.publicKey}] Created new mint account at ${signature}`,
    )
    return tokenMint.publicKey
  }

  const createAssociatedWalletAndMintSomeTokens = async ({ mint, user }) => {
    // Create a token account for the user and mint some tokens
    let userAssociatedTokenAccount = await spl.getAssociatedTokenAddress(
      mint,
      user.publicKey,
      true,
      spl.TOKEN_PROGRAM_ID,
      spl.ASSOCIATED_TOKEN_PROGRAM_ID,
    )

    const txFundTokenAccount = new anchor.web3.Transaction()
    txFundTokenAccount.add(
      spl.createAssociatedTokenAccountInstruction(
        user.publicKey,
        userAssociatedTokenAccount,
        user.publicKey,
        mint,
        spl.TOKEN_PROGRAM_ID,
        spl.ASSOCIATED_TOKEN_PROGRAM_ID,
      ),
    )
    txFundTokenAccount.add(
      spl.createMintToInstruction(
        mint,
        userAssociatedTokenAccount,
        provider.wallet.publicKey,
        1337000000,
        [],
        spl.TOKEN_PROGRAM_ID,
      ),
    )

    const txFundTokenSig = await provider.sendAndConfirm(txFundTokenAccount, [
      user,
    ])
    console.log(
      `[${userAssociatedTokenAccount.toBase58()}] New associated account for mint ${mint.toBase58()}: ${txFundTokenSig}`,
    )

    return userAssociatedTokenAccount
  }

  const createUserAndAssociatedWallet = async (
    connection: anchor.web3.Connection,
    mint?: anchor.web3.PublicKey,
  ): Promise<[anchor.web3.Keypair, anchor.web3.PublicKey | undefined]> => {
    const user = new anchor.web3.Keypair()
    let userAssociatedTokenAccount:
      | anchor.web3.PublicKey
      | undefined = undefined

    // Fund user with some SOL
    let txFund = new anchor.web3.Transaction()
    txFund.add(
      anchor.web3.SystemProgram.transfer({
        fromPubkey: provider.wallet.publicKey,
        toPubkey: user.publicKey,
        lamports: 5 * anchor.web3.LAMPORTS_PER_SOL,
      }),
    )
    const sigTxFund = await provider.sendAndConfirm(txFund)
    console.log(
      `[${user.publicKey.toBase58()}] Funded new account with 5 SOL: ${sigTxFund}`,
    )

    if (mint) {
      userAssociatedTokenAccount = await createAssociatedWalletAndMintSomeTokens(
        { mint, user },
      )
    }
    return [user, userAssociatedTokenAccount]
  }

  const getPdaParams = async (
    connection: anchor.web3.Connection,
    alice: anchor.web3.PublicKey,
    bob: anchor.web3.PublicKey,
    mint: anchor.web3.PublicKey,
  ): Promise<PDAParameters> => {
    // const uid = new anchor.BN(parseInt((Date.now() / 1000).toString()))
    // const uidBuffer = uid.toBuffer('le', 8)

    let [
      taskVaultAccount,
      taskVaultAccountBump,
    ] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from('task_vault')],
      program.programId,
    )

    let [taskVaultTreasurer] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from('treasurer'), taskVaultAccount.toBuffer()],
      program.programId,
    )

    let taskVaultTokenAccount = await anchor.utils.token.associatedAddress({
      owner: taskVaultTreasurer,
      mint: mint,
    })

    return {
      taskVaultAccount,
      taskVaultBump: taskVaultAccountBump,
      taskVaultTreasurer,
      taskVaultTokenAccount,
    }
  }

  const readAccount = async (
    accountPublicKey: anchor.web3.PublicKey,
    provider: anchor.Provider,
  ): Promise<[spl.RawAccount, string]> => {
    const tokenInfoLol = await provider.connection.getAccountInfo(
      accountPublicKey,
    )
    const data = Buffer.from(tokenInfoLol.data)
    const accountInfo: spl.RawAccount = spl.AccountLayout.decode(data)
    const amount = accountInfo.amount

    return [accountInfo, amount.toString()]
  }

  beforeEach(async () => {
    mintAddress = await createMint(provider.connection)
    ;[alice, aliceWallet] = await createUserAndAssociatedWallet(
      provider.connection,
      mintAddress,
    )
    let _rest: any
    ;[bob, ..._rest] = await createUserAndAssociatedWallet(provider.connection)

    pda = await getPdaParams(
      provider.connection,
      alice.publicKey,
      bob.publicKey,
      mintAddress,
    )
  })

  /* it('Initialize a task vault!', async () => {
    const [, aliceBalancePre] = await readAccount(aliceWallet, provider)
    assert.equal(aliceBalancePre, '1337000000')

    const amount = new anchor.BN(20000000)

    await program.methods
      .initializeTaskvault(pda.taskVaultBump, SAMPLE_PRIZE, amount)
      .accounts({
        authority: alice.publicKey,
        authorityTokenAccount: aliceWallet,

        taskVault: pda.taskVaultAccount,
        treasurer: pda.taskVaultTreasurer,
        taskVaultTokenAccount: pda.taskVaultTokenAccount,

        mintOfTokenBeingSent: mintAddress,

        // programs
        systemProgram: anchor.web3.SystemProgram.programId,
        tokenProgram: spl.TOKEN_PROGRAM_ID,
        associatedTokenProgram: spl.ASSOCIATED_TOKEN_PROGRAM_ID,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
      })
      .signers([alice])
      .rpc()

    const accountInfo = await program.account.taskVault.fetch(
      pda.taskVaultAccount,
    )
    console.log(accountInfo)
  }) */

  it('Initialize the vault (existing deposit) -> Deposit more token -> Withdraw !', async () => {
    const [, aliceBalancePre] = await readAccount(aliceWallet, provider)
    assert.equal(aliceBalancePre, '1337000000')

    const amount = new anchor.BN(20000000)

    await program.methods
      .initializeTaskvault(pda.taskVaultBump, SAMPLE_PRIZE, amount)
      .accounts({
        authority: alice.publicKey,
        authorityTokenAccount: aliceWallet,

        taskVault: pda.taskVaultAccount,
        treasurer: pda.taskVaultTreasurer,
        taskVaultTokenAccount: pda.taskVaultTokenAccount,

        mintOfTokenBeingSent: mintAddress,

        // programs
        systemProgram: anchor.web3.SystemProgram.programId,
        tokenProgram: spl.TOKEN_PROGRAM_ID,
        associatedTokenProgram: spl.ASSOCIATED_TOKEN_PROGRAM_ID,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
      })
      .signers([alice])
      .rpc()

    {
      // Deposit to the vault
      let amount = new anchor.BN(10000000)

      await program.methods
        .depositToTheVault(amount)
        .accounts({
          authority: alice.publicKey,
          authorityTokenAccount: aliceWallet,

          taskVault: pda.taskVaultAccount,
          treasurer: pda.taskVaultTreasurer,
          taskVaultTokenAccount: pda.taskVaultTokenAccount,

          mintOfTokenBeingSent: mintAddress,

          // programs
          systemProgram: anchor.web3.SystemProgram.programId,
          tokenProgram: spl.TOKEN_PROGRAM_ID,
        })
        .signers([alice])
        .rpc()

      const accountInfo = await program.account.taskVault.fetch(
        pda.taskVaultAccount,
      )
      console.log(accountInfo)
    }

    {
      // Withdraw from the vault
      let amount = new anchor.BN(10000002)

      /*       await program.methods
        .withdrawFromTheVault(amount)
        .accounts({
          authority: alice.publicKey,
          authorityTokenAccount: aliceWallet,

          taskVault: pda.taskVaultAccount,
          treasurer: pda.taskVaultTreasurer,
          taskVaultTokenAccount: pda.taskVaultTokenAccount,

          mintOfTokenBeingSent: mintAddress,

          // programs
          systemProgram: anchor.web3.SystemProgram.programId,
          tokenProgram: spl.TOKEN_PROGRAM_ID,
        })
        .signers([alice])
        .rpc() */

      const accountInfo = await program.account.taskVault.fetch(
        pda.taskVaultAccount,
      )
      console.log(accountInfo)
    }
  })
})
