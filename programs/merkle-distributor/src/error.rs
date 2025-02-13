use anchor_lang::error_code;

/// Error codes.
#[error_code]
pub enum ErrorCode {
    #[msg("Insufficient unlocked tokens")]
    InsufficientUnlockedTokens,
    #[msg("Deposit Start too far in future")]
    StartTooFarInFuture,
    #[msg("Invalid Merkle proof.")]
    InvalidProof,
    #[msg("Exceeded maximum claim amount")]
    ExceededMaxClaim,
    #[msg("Exceeded maximum node count")]
    MaxNodesExceeded,
    #[msg("Account is not authorized to execute this instruction")]
    Unauthorized,
    #[msg("Token account owner did not match intended owner")]
    OwnerMismatch,
    #[msg("Clawback cannot be before vesting ends")]
    ClawbackDuringVesting,
    #[msg("Attempted clawback before start")]
    ClawbackBeforeStart,
    #[msg("Clawback already claimed")]
    ClawbackAlreadyClaimed,
    #[msg("Clawback start must be at least one day after vesting end")]
    InsufficientClawbackDelay,
    #[msg("New and old Clawback receivers are identical")]
    SameClawbackReceiver,
    #[msg("New and old admin are identical")]
    SameAdmin,
    #[msg("Claim window expired")]
    ClaimExpired,
    #[msg("Arithmetic Error (overflow/underflow)")]
    ArithmeticError,
    #[msg("Start Timestamp cannot be after end Timestamp")]
    StartTimestampAfterEnd,
    #[msg("Timestamps cannot be in the past")]
    TimestampsNotInFuture,
    #[msg("Airdrop Version Mismatch")]
    InvalidVersion,
    #[msg("Claiming is not started")]
    ClaimingIsNotStarted,
    #[msg("Cannot close distributor")]
    CannotCloseDistributor,
    #[msg("Cannot close claim status")]
    CannotCloseClaimStatus,
    #[msg("Invalid activation type")]
    InvalidActivationType,
    #[msg("Type casted error")]
    TypeCastedError,
    #[msg("Invalid operator")]
    InvalidOperator,
    #[msg("Invalid claim type")]
    InvalidClaimType,
    #[msg("Same operator")]
    SameOperator,
    #[msg("Invalid locker")]
    InvalidLocker,
    #[msg("Escrow is not max lock")]
    EscrowIsNotMaxLock,
}
