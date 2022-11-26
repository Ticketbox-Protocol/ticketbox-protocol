import {
  workspace,
  Program,
  web3,
  BN,
  AnchorProvider,
  setProvider,
} from "@project-serum/anchor";
import {
  createAssociatedTokenAccountInstruction,
  createInitializeMintInstruction,
  createMintToInstruction,
  getAssociatedTokenAddress,
  MINT_SIZE,
  TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
import { assert } from "chai";
import { TicketBoxProgram } from "../target/types/ticket_box_program";
import { getPDA, handleAirdrop } from "./utils";
import {
  COLLECTION_ASSET_URL,
  TOKEN_METADATA_PROGRAM_ID,
} from "./contants";

describe("Initialize", () => {
  const provider = AnchorProvider.env();
  setProvider(provider);
  const program = workspace.TicketBoxProgram as Program<TicketBoxProgram>;

  let creator: web3.Keypair;

  let collectionMinKp: web3.Keypair;

  let ticketBoxId = new Date().getTime().toString();

  before(async () => {
    // creator = getKeypairFromFile(
    //   path.resolve(__dirname, "../", "keys/creator1.json")
    // );
    creator = web3.Keypair.generate();
    console.log("creator key: ", creator.publicKey.toBase58());
    await handleAirdrop(provider, creator.publicKey);
  });

  it("Initialize", async () => {
    const ticketBoxName = "Flip Girl #0001";
    const now = Math.floor(new Date().getTime() / 1000);
    const startAt = now + 1000 * 10;
    const endAt = startAt + 10 * 60 * 1000;

    const ticketBoxPda = await getPDA(
      [
        Buffer.from("ticket_box"),
        Buffer.from(ticketBoxId),
        creator.publicKey.toBuffer(),
      ],
      program.programId
    );

    // pre-tx to mint collection
    const ix = [];
    collectionMinKp = web3.Keypair.generate();

    const lamports: number =
      await program.provider.connection.getMinimumBalanceForRentExemption(
        MINT_SIZE
      );
    // create collection token mint
    ix.push(
      web3.SystemProgram.createAccount({
        fromPubkey: creator.publicKey,
        newAccountPubkey: collectionMinKp.publicKey,
        space: MINT_SIZE,
        programId: TOKEN_PROGRAM_ID,
        lamports,
      })
    );

    // init mint token
    ix.push(
      createInitializeMintInstruction(
        collectionMinKp.publicKey,
        0,
        creator.publicKey,
        creator.publicKey
      )
    );

    const collectionTokenAccount = await getAssociatedTokenAddress(
      collectionMinKp.publicKey,
      creator.publicKey
    );
    // create collection token account
    ix.push(
      createAssociatedTokenAccountInstruction(
        creator.publicKey,
        collectionTokenAccount,
        creator.publicKey,
        collectionMinKp.publicKey
      )
    );
    // mint 1 token to creator
    ix.push(
      createMintToInstruction(
        collectionMinKp.publicKey,
        collectionTokenAccount,
        creator.publicKey,
        1
      )
    );

    const collectionMetadataPDA = await getPDA(
      [
        Buffer.from("metadata"),
        TOKEN_METADATA_PROGRAM_ID.toBuffer(),
        collectionMinKp.publicKey.toBuffer(),
      ],
      TOKEN_METADATA_PROGRAM_ID
    );

    const collectionMasterEditionPda = await getPDA(
      [
        Buffer.from("metadata"),
        TOKEN_METADATA_PROGRAM_ID.toBuffer(),
        collectionMinKp.publicKey.toBuffer(),
        Buffer.from("edition"),
      ],
      TOKEN_METADATA_PROGRAM_ID
    );

    const tx = await program.methods
      .initialize(
        ticketBoxId,
        ticketBoxName,
        COLLECTION_ASSET_URL,
        new BN(startAt),
        new BN(endAt), //
        new BN(100), //
        new BN(1), //
        new BN(0.5 * 10 ** 9), // 0.5 SOL
        // new BN(0),
        true
      )
      .accounts({
        creator: creator.publicKey,
        ticketBox: ticketBoxPda,
        wallet: creator.publicKey,
        collectionMint: collectionMinKp.publicKey,
        collectionTokenAccount: collectionTokenAccount,
        collectionMetadata: collectionMetadataPDA,
        collectionMasterEdition: collectionMasterEditionPda,
        systemProgram: web3.SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
        rent: web3.SYSVAR_RENT_PUBKEY,
        tokenMetadataProgram: TOKEN_METADATA_PROGRAM_ID,
      })
      .preInstructions(ix)
      .signers([creator, collectionMinKp])
      .rpc();

    console.log("Initialize tx hash", tx);

    const loadedTicketBoxAccount = await program.account.ticketBox.fetch(
      ticketBoxPda
    );

    assert.strictEqual(loadedTicketBoxAccount.uuid, ticketBoxId, "creator");

    assert.strictEqual(loadedTicketBoxAccount.name, ticketBoxName, "creator");

    assert.strictEqual(
      loadedTicketBoxAccount.creator.toBase58(),
      creator.publicKey.toBase58(),
      "creator"
    );

    assert.strictEqual(
      loadedTicketBoxAccount.creator.toBase58(),
      creator.publicKey.toBase58(),
      "creator"
    );

    assert.strictEqual(loadedTicketBoxAccount.currency, null, "currency");
  });
});
