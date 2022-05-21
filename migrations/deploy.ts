// Migrations are an early feature. Currently, they're nothing more than this
// single deploy script that's invoked from the CLI, injecting a provider
// configured from the workspace's Anchor.toml.

import anchor from '@project-serum/anchor'
import * as spl from '@solana/spl-token'

import { PDAParameters } from '../tests/plats-solana'

module.exports = async function (provider: anchor.Provider) {
  // Configure client to use the provider.
  anchor.setProvider(provider)

  // Helper functions
  const createMint = async (
    connection: anchor.web3.Connection,
    tokenMint: anchor.web3.Keypair = new anchor.web3.Keypair(),
  ): Promise<anchor.web3.PublicKey> => {
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
        {
          mint,
          user,
        },
      )
    }
    return [user, userAssociatedTokenAccount]
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

  // Add your deploy script here.
  let mintAddress = await createMint(provider.connection)

  console.log('[Mint Address]', mintAddress)
  console.log(
    '🆗 With a mint address and an user address, a user token account can be generated',
  )
}
