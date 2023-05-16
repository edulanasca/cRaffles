use anchor_lang::prelude::*;

#[error_code]
pub enum RaffleError {
    #[msg("Max entrants is too large")]
    MaxEntrantsTooLarge,
    #[msg("Raffle has ended")]
    RaffleEnded,
    #[msg("Invalid calculation")]
    InvalidCalculation,
    #[msg("Invalid prize index")]
    InvalidPrizeIndex,
    #[msg("No prize")]
    NoPrize,
    #[msg("Not enough tickets left")]
    NotEnoughTicketsLeft,
    #[msg("Unclaimed prizes")]
    UnclaimedPrizes,
    #[msg("Raffle is still running")]
    RaffleStillRunning,
    #[msg("Winner not drawn")]
    WinnerNotDrawn,
    #[msg("Ticket account not owned by winner")]
    TokenAccountNotOwnedByWinner,
    #[msg("Ticket has not won")]
    TicketHasNotWon,
    #[msg("Winner already drawn")]
    WinnersAlreadyDrawn,
    #[msg("An account's data contents was invalid")]
    InvalidAccountData,
}