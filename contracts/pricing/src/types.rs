use soroban_sdk::{contracterror, contracttype, Address, Symbol, Vec};

pub const SCALE: i128 = 1_000_000_000_000;
pub const TTL: u32 = 120_960;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum Error {
    Config = 1,
    Unknown = 2,
    Invocation = 3,
    Identity = 4,
    Missing = 5,
    Invalid = 6,
    Future = 7,
    Stale = 8,
    Skew = 9,
    Depth = 10,
    Spacing = 11,
    Bootstrap = 12,
    Divergence = 13,
    Overflow = 14,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FeedAsset {
    Stellar(Address),
    Other(Symbol),
}
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PriceData {
    pub price: i128,
    pub timestamp: u64,
}
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Feed {
    pub contract: Address,
    pub asset: FeedAsset,
    pub base: FeedAsset,
    pub decimals: u32,
    pub resolution: u32,
    pub max_age: u64,
}
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Source {
    pub asset: Address,
    pub decimals: u32,
    pub numerator: Feed,
    pub denominator: Vec<Feed>, // Zero or one conversion leg.
    pub pool: Address,
    pub asset_is_token0: bool,
    pub min_asset_reserve: i128,
    pub min_usdc_reserve: i128,
    pub min_spacing: u64,
    pub max_gap: u64,
    pub max_age: u64,
    pub max_skew: u64,
    pub divergence_bps: u32,
}
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Config {
    pub usdc: Address,
    pub sources: Vec<Source>,
}
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Observation {
    pub price: i128,
    pub timestamp: u64,
    pub ledger: u32,
    pub asset_reserve: i128,
    pub usdc_reserve: i128,
}
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct History {
    pub completed: Vec<Observation>,
    pub pending: Vec<Observation>, // Zero or one observation.
}
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Mark {
    pub price: i128,
    pub reference: i128,
    pub timestamp: u64,
    pub ready: bool,
}
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Diagnostic {
    pub source: Source,
    pub numerator: Vec<PriceData>,
    pub denominator: Vec<PriceData>,
    pub samples: Vec<Observation>,
    pub mark: Mark,
    pub error: u32, // Zero means ready; otherwise the contract Error discriminant.
}
#[contracttype]
#[derive(Clone)]
pub enum Key {
    Config,
    History(Address),
}
