use soroban_sdk::{contracterror, contracttype, Address, BytesN, Vec};

pub const BPS: i128 = 10_000;
pub const SCALE: i128 = 1_000_000_000_000_000_000;
pub const PRICE_SCALE: i128 = 1_000_000_000_000;
pub const YEAR: u64 = 31_536_000;
pub const SEED_ASSETS: i128 = 10_000_000_000; // 1,000 USDC, seven decimals
pub const SHARE_MULTIPLIER: i128 = 100_000; // twelve share decimals
pub const MAX_AMOUNT: i128 = 1_000_000_000_000_000_000_000_000_000_000;
pub const MAX_MANAGEMENT_BPS: u32 = 200;
pub const MAX_PERFORMANCE_BPS: u32 = 2_000;
pub const ROLE_DELAY: u32 = 34_560; // two days at five seconds per ledger

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum Error {
    Invalid = 1,
    Overflow = 2,
    Unauthorized = 3,
    InsufficientBalance = 4,
    Expired = 5,
    Paused = 6,
    Pricing = 7,
    Insolvent = 8,
    Liquidity = 9,
    ExternalClaim = 10,
    CatchUp = 11,
    Reentrant = 12,
    SeedLocked = 13,
    Slippage = 14,
    Target = 15,
    Replay = 16,
    Limit = 17,
    Timelock = 18,
    BorrowingDisabled = 19,
    Rewards = 20,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AssetKind {
    Settlement,
    Xlm,
    Risk,
    Reserve,
    Reward,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Asset {
    pub address: Address,
    pub decimals: u32,
    pub kind: AssetKind,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Fees {
    pub management_bps: u32,
    pub performance_bps: u32,
    pub recipient: Address,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Config {
    pub assets: Vec<Asset>,
    pub usdc: Address,
    pub oracle: Address,
    pub router: Address,
    pub route_contracts: Vec<Address>,
    pub pool: Option<Address>,
    pub pool_assets: Vec<Address>,
    pub admin: Address,
    pub executor: Address,
    pub guardian: Address,
    pub fees: Fees,
    pub borrowing: bool,
    pub max_price_age: u64,
    pub max_divergence_bps: u32,
    pub max_slippage_bps: u32,
    pub max_leg_bps: u32,
    pub max_turnover_bps: u32,
    pub max_loss_bps: u32,
    pub cooldown: u32,
    pub max_debt_bps: u32,
    pub min_health_bps: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct State {
    pub supply: i128,
    pub high_water: i128,
    pub last_fee: u64,
    pub management_dust: i128,
    pub performance_dust: i128,
    pub nonce: u64,
    pub version: u32,
    pub paused: bool,
    pub last_execution: u32,
    pub window_ledger: u32,
    pub window_equity: i128,
    pub turnover: i128,
    pub loss: i128,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Mark {
    pub price: i128,     // Reflector USDC per whole asset, 12 decimals
    pub reference: i128, // Three-observation Soroswap sampled TWAP
    pub timestamp: u64,
    pub ready: bool,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Holding {
    pub asset: Address,
    pub spot: i128,
    pub supply: i128,
    pub collateral: i128,
    pub debt: i128,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Target {
    pub network: BytesN<32>,
    pub vault: Address,
    pub epoch: u64,
    pub expiry: u64,
    pub model: BytesN<32>,
    pub dataset: BytesN<32>,
    pub holdings: BytesN<32>,
    pub assets: Vec<Address>,
    pub weights: Vec<u32>, // total spot plus supply weights; XLM includes its buffer
    pub yield_bps: u32,    // fraction of residual reserve, <= 1,000 bps
    pub equity: i128,
    pub supply: i128,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Protocol {
    Soroswap = 0,
    Phoenix = 1,
    Aqua = 2,
    Comet = 3,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DexDistribution {
    pub protocol_id: Protocol,
    pub path: Vec<Address>,
    pub parts: u32,
    pub bytes: Option<Vec<BytesN<32>>>,
}

#[contracttype]
#[derive(Clone)]
pub struct Swap {
    pub auth: Vec<RouteAuth>,
    pub token_in: Address,
    pub token_out: Address,
    pub amount: i128,
    pub minimum: i128,
    pub distribution: Vec<DexDistribution>,
}

#[contracttype]
#[derive(Clone)]
pub enum Action {
    Swap(Swap),
    Supply(Address, i128),
    Withdraw(Address, i128),
    Collateral(Address, i128),
    Release(Address, i128),
    Borrow(Address, i128),
    Repay(Address, i128),
}

#[contracttype]
#[derive(Clone)]
pub struct Plan {
    pub target: BytesN<32>,
    pub epoch: u64,
    pub nonce: u64,
    pub deadline: u64,
    pub version: u32,
    pub actions: Vec<Action>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Role {
    Admin,
    Executor,
    Guardian,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Change {
    Fees(Fees),
    Role(Role, Address),
    Resume,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Pending {
    pub hash: BytesN<32>,
    pub earliest: u32,
    pub change: Change,
}

#[contracttype]
#[derive(Clone)]
pub enum RouteAuth {
    Transfer(Address, Address, i128), // token, recipient, exact amount; sender is the vault
    Invoke(RouteInvocation),
}
#[contracttype]
#[derive(Clone)]
pub struct RouteInvocation {
    pub contract: Address,
    pub function: soroban_sdk::Symbol,
    pub args: Vec<soroban_sdk::Val>,
    pub children: Vec<RouteAuth>,
}
