import * as anchor from "@coral-xyz/anchor";
import {Program, ProgramAccount} from "@coral-xyz/anchor";
import {Craffles} from "../target/types/craffles";
import {
    createAssociatedTokenAccountInstruction,
    createSyncNativeInstruction,
    getAccount,
    getAssociatedTokenAddress,
    NATIVE_MINT,
    TOKEN_PROGRAM_ID
} from "@solana/spl-token";
import {
    MetadataArgs,
    PROGRAM_ID as BUBBLEGUM_PROGRAM_ID,
    TokenProgramVersion,
    TokenStandard
} from "@metaplex-foundation/mpl-bubblegum";
import {
    getConcurrentMerkleTreeAccountSize,
    SPL_ACCOUNT_COMPRESSION_PROGRAM_ID,
    SPL_NOOP_PROGRAM_ID,
    ValidDepthSizePair
} from "@solana/spl-account-compression";
import {SystemProgram} from "@solana/web3.js";

describe("craffles", () => {
    // Configure the client to use the local cluster.
    anchor.setProvider(anchor.AnchorProvider.env());

    const program = anchor.workspace.Craffles as Program<Craffles>;

    it("Is initialized!", async () => {
        const merkle_tree = anchor.web3.Keypair.generate();

        const [raffle] = anchor.web3.PublicKey.findProgramAddressSync(
            [anchor.utils.bytes.utf8.encode("raffle"), merkle_tree.publicKey.toBuffer()],
            program.programId
        );
        const [proceeds] = anchor.web3.PublicKey.findProgramAddressSync(
            [anchor.utils.bytes.utf8.encode("proceeds"), raffle.toBuffer()],
            program.programId
        );

        const [treeAuthority] = anchor.web3.PublicKey.findProgramAddressSync(
            [merkle_tree.publicKey.toBuffer()],
            BUBBLEGUM_PROGRAM_ID,
        );

        const depthSizePair: ValidDepthSizePair = {
            maxDepth: 14,
            maxBufferSize: 64
        }

        const space = getConcurrentMerkleTreeAccountSize(depthSizePair.maxDepth, depthSizePair.maxBufferSize);

        try {
            // Add your test here.
            const tx = await program.methods
                .createRaffle(
                    new anchor.BN(1684377928),
                    new anchor.BN(0.001 * anchor.web3.LAMPORTS_PER_SOL),
                    depthSizePair
                )
                .accounts({
                    raffle,
                    proceeds,
                    proceedsMint: NATIVE_MINT,
                    merkleTree: merkle_tree.publicKey,
                    treeAuthority,
                    creator: program.provider.publicKey!,
                    logWrapper: SPL_NOOP_PROGRAM_ID,
                    compressionProgram: SPL_ACCOUNT_COMPRESSION_PROGRAM_ID,
                    bubblegumProgram: BUBBLEGUM_PROGRAM_ID,
                    tokenProgram: TOKEN_PROGRAM_ID,
                    systemProgram: SystemProgram.programId
                })
                .preInstructions([
                    SystemProgram.createAccount({
                        fromPubkey: program.provider.publicKey!,
                        lamports: await program.provider.connection.getMinimumBalanceForRentExemption(space),
                        newAccountPubkey: merkle_tree.publicKey,
                        programId: SPL_ACCOUNT_COMPRESSION_PROGRAM_ID,
                        space,
                    }),
                ])
                .signers([merkle_tree])
                .rpc();

            console.log("Your transaction signature", tx);
        } catch (err) {
            console.log(err);
        }
    });

    it.only("Should buy a ticket", async () => {
        /* This raffle has to be bought with SOL */

        try {
            const raffle: ProgramAccount<Awaited<ReturnType<Program<Craffles>["account"]["raffle"]["fetch"]>>> = (await program.account.raffle.all())[0];
            const [proceeds] = anchor.web3.PublicKey.findProgramAddressSync(
                [anchor.utils.bytes.utf8.encode("proceeds"), raffle.publicKey.toBuffer()],
                program.programId
            );
            const [treeAuthority] = anchor.web3.PublicKey.findProgramAddressSync(
                [raffle.account.merkleTree.toBuffer()],
                BUBBLEGUM_PROGRAM_ID
            );
            const ticketPrize = raffle.account.ticketPrice.toNumber();

            const proceedsAccount = await getAccount(program.provider.connection, proceeds);
            const userAta = await getAssociatedTokenAddress(proceedsAccount.mint, program.provider.publicKey!);

            const metadata: MetadataArgs = {
                collection: undefined,
                creators: [{address: program.provider.publicKey!, verified: false, share: 100}],
                editionNonce: 0,
                isMutable: true,
                name: "The Raffle Project",
                primarySaleHappened: true,
                sellerFeeBasisPoints: 0,
                symbol: "RAFF",
                tokenProgramVersion: TokenProgramVersion.Original,
                tokenStandard: TokenStandard.NonFungible,
                uri: "https://nftstorage.link/ipfs/bafkreibyqogqxglj7nchdcz5h742yqxpnr5ikfoemeff7skhftdqjg3ymq",
                uses: undefined
            }

            const buyTickets = await program.methods.buyTickets(1, metadata)
                .accounts({
                    raffle: raffle.publicKey,
                    proceeds,
                    treeAuthority,
                    leafOwner: program.provider.publicKey!, // User to mint the cNFT
                    leafDelegate: program.provider.publicKey!,
                    merkleTree: raffle.account.merkleTree,
                    buyerTokenAccount: userAta,
                    buyer: program.provider.publicKey!,
                    bubblegumProgram: BUBBLEGUM_PROGRAM_ID,
                    logWrapper: SPL_NOOP_PROGRAM_ID,
                    compressionProgram: SPL_ACCOUNT_COMPRESSION_PROGRAM_ID,
                    tokenProgram: TOKEN_PROGRAM_ID,
                    systemProgram: SystemProgram.programId
                });

            if (proceedsAccount.mint.equals(NATIVE_MINT)) {
                try {
                    await getAccount(program.provider.connection, userAta);
                    buyTickets.preInstructions([
                        anchor.web3.SystemProgram.transfer({
                            fromPubkey: program.provider.publicKey!,
                            toPubkey: userAta,
                            lamports: ticketPrize * anchor.web3.LAMPORTS_PER_SOL
                        }),
                        createSyncNativeInstruction(userAta, TOKEN_PROGRAM_ID)
                    ])
                } catch (err) {
                    buyTickets.preInstructions([
                        createAssociatedTokenAccountInstruction(
                            program.provider.publicKey!,
                            userAta,
                            program.provider.publicKey!,
                            proceedsAccount.mint,
                            TOKEN_PROGRAM_ID
                        ),
                        anchor.web3.SystemProgram.transfer({
                            fromPubkey: program.provider.publicKey!,
                            toPubkey: userAta,
                            lamports: ticketPrize * anchor.web3.LAMPORTS_PER_SOL
                        }),
                        createSyncNativeInstruction(userAta, TOKEN_PROGRAM_ID)
                    ])
                }
            }

            const tx = await buyTickets.rpc();
            console.log("Signature", tx);
        } catch (err) {
            console.log(err);
        }
    });
});
