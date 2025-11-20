use chrono::{DateTime, Duration, Utc};
/// Capability Trading Marketplace
///
/// Dynamic marketplace for agents to buy/sell capabilities with flexible pricing models,
/// SLA guarantees, and smart contract enforcement.
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Pricing model for a capability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PricingModel {
    Fixed { cost_per_use: f64 },
    PerUnit { cost_per_unit: f64 },
    Subscription { cost_per_month: f64 },
    Auction { current_bid: f64 },
}

impl PricingModel {
    /// Calculate cost for using capability
    pub fn calculate_cost(&self, units: f64) -> f64 {
        match self {
            PricingModel::Fixed { cost_per_use } => *cost_per_use,
            PricingModel::PerUnit { cost_per_unit } => cost_per_unit * units,
            PricingModel::Subscription { cost_per_month } => cost_per_month / 30.0 / 24.0, // Per hour
            PricingModel::Auction { current_bid } => *current_bid,
        }
    }
}

/// Service Level Agreement for capability provisioning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceLevelAgreement {
    pub uptime_percent: f64, // 99.9 = 99.9%
    pub max_latency_ms: u64,
    pub availability_window: String, // "24x7" or specific hours
    pub breach_penalty_percent: f64,
}

impl Default for ServiceLevelAgreement {
    fn default() -> Self {
        Self {
            uptime_percent: 99.9,
            max_latency_ms: 100,
            availability_window: "24x7".to_string(),
            breach_penalty_percent: 10.0,
        }
    }
}

/// Capability listing in the marketplace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityListing {
    pub listing_id: String,
    pub provider_id: String,
    pub capability_name: String,
    pub description: String,
    pub pricing: PricingModel,
    pub sla: ServiceLevelAgreement,
    pub available_quantity: Option<u64>,
    pub rating: f64, // 0.0 to 5.0
    pub review_count: u32,
    pub listed_at: DateTime<Utc>,
    pub active: bool,
}

impl CapabilityListing {
    pub fn new(
        provider_id: String,
        capability_name: String,
        pricing: PricingModel,
        sla: ServiceLevelAgreement,
    ) -> Self {
        Self {
            listing_id: uuid::Uuid::new_v4().to_string(),
            provider_id,
            capability_name,
            description: String::new(),
            pricing,
            sla,
            available_quantity: None,
            rating: 3.0,
            review_count: 0,
            listed_at: Utc::now(),
            active: true,
        }
    }

    /// Calculate price-to-quality ratio (lower is better)
    pub fn value_score(&self) -> f64 {
        let base_cost = self.pricing.calculate_cost(1.0);
        let quality = self.rating / 5.0;
        base_cost / (quality + 0.1)
    }
}

/// Smart contract for capability trading
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityContract {
    pub contract_id: String,
    pub buyer_id: String,
    pub seller_id: String,
    pub listing_id: String,
    pub quantity: u64,
    pub total_cost: f64,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub status: ContractStatus,
    pub sla: ServiceLevelAgreement,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ContractStatus {
    Active,
    Suspended,
    Fulfilled,
    Breached,
}

impl CapabilityContract {
    pub fn new(
        buyer_id: String,
        listing: &CapabilityListing,
        quantity: u64,
        duration_days: i64,
    ) -> Self {
        let created_at = Utc::now();
        let expires_at = created_at + Duration::days(duration_days);
        let total_cost = listing.pricing.calculate_cost(quantity as f64);

        Self {
            contract_id: uuid::Uuid::new_v4().to_string(),
            buyer_id,
            seller_id: listing.provider_id.clone(),
            listing_id: listing.listing_id.clone(),
            quantity,
            total_cost,
            created_at,
            expires_at,
            status: ContractStatus::Active,
            sla: listing.sla.clone(),
        }
    }

    /// Check if contract is still valid
    pub fn is_active(&self) -> bool {
        self.status == ContractStatus::Active && Utc::now() < self.expires_at
    }

    /// Remaining time in seconds
    pub fn time_remaining(&self) -> i64 {
        (self.expires_at - Utc::now()).num_seconds()
    }
}

/// Marketplace registry and matching engine
pub struct CapabilityMarket {
    listings: Arc<RwLock<HashMap<String, CapabilityListing>>>,
    contracts: Arc<RwLock<Vec<CapabilityContract>>>,
    trades: Arc<RwLock<Vec<Trade>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trade {
    pub contract_id: String,
    pub timestamp: DateTime<Utc>,
    pub buyer_id: String,
    pub seller_id: String,
    pub amount: f64,
}

impl CapabilityMarket {
    pub fn new() -> Self {
        Self {
            listings: Arc::new(RwLock::new(HashMap::new())),
            contracts: Arc::new(RwLock::new(Vec::new())),
            trades: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// List a capability for sale
    pub async fn list_capability(&self, listing: CapabilityListing) {
        let mut listings = self.listings.write().await;
        listings.insert(listing.listing_id.clone(), listing);
    }

    /// Find listings for a capability
    pub async fn find_capability(&self, capability_name: &str) -> Vec<CapabilityListing> {
        let listings = self.listings.read().await;
        listings
            .values()
            .filter(|l| l.capability_name == capability_name && l.active)
            .cloned()
            .collect()
    }

    /// Get best listing by value (rating per dollar)
    pub async fn find_best_value(&self, capability_name: &str) -> Option<CapabilityListing> {
        let listings = self.find_capability(capability_name).await;
        listings
            .iter()
            .min_by(|a, b| a.value_score().partial_cmp(&b.value_score()).unwrap())
            .cloned()
    }

    /// Get best listing by SLA uptime
    pub async fn find_best_sla(&self, capability_name: &str) -> Option<CapabilityListing> {
        let listings = self.find_capability(capability_name).await;
        listings
            .iter()
            .max_by(|a, b| a.sla.uptime_percent.partial_cmp(&b.sla.uptime_percent).unwrap())
            .cloned()
    }

    /// Create a trading contract
    pub async fn create_contract(
        &self,
        buyer_id: String,
        listing_id: &str,
        quantity: u64,
        duration_days: i64,
    ) -> Option<CapabilityContract> {
        let listings = self.listings.read().await;
        let listing = listings.get(listing_id)?;

        // Check quantity availability
        if let Some(available) = listing.available_quantity {
            if quantity > available {
                return None;
            }
        }

        let contract = CapabilityContract::new(buyer_id, listing, quantity, duration_days);

        // Record the trade
        let trade = Trade {
            contract_id: contract.contract_id.clone(),
            timestamp: Utc::now(),
            buyer_id: contract.buyer_id.clone(),
            seller_id: contract.seller_id.clone(),
            amount: contract.total_cost,
        };

        drop(listings); // Release read lock

        let mut contracts = self.contracts.write().await;
        contracts.push(contract.clone());

        let mut trades = self.trades.write().await;
        trades.push(trade);

        Some(contract)
    }

    /// Get active contracts for an agent
    pub async fn agent_contracts(&self, agent_id: &str) -> Vec<CapabilityContract> {
        let contracts = self.contracts.read().await;
        contracts
            .iter()
            .filter(|c| (c.buyer_id == agent_id || c.seller_id == agent_id) && c.is_active())
            .cloned()
            .collect()
    }

    /// Get trade history
    pub async fn trade_history(&self, limit: usize) -> Vec<Trade> {
        let trades = self.trades.read().await;
        trades.iter().rev().take(limit).cloned().collect()
    }

    /// Get total volume traded
    pub async fn total_volume(&self) -> f64 {
        let trades = self.trades.read().await;
        trades.iter().map(|t| t.amount).sum()
    }

    /// Update contract status (e.g., after SLA breach detection)
    pub async fn update_contract_status(&self, contract_id: &str, status: ContractStatus) {
        let mut contracts = self.contracts.write().await;
        if let Some(contract) = contracts.iter_mut().find(|c| c.contract_id == contract_id) {
            contract.status = status;
        }
    }

    /// Rate a capability provider
    pub async fn rate_provider(&self, listing_id: &str, rating: f64, review: String) {
        let mut listings = self.listings.write().await;
        if let Some(listing) = listings.get_mut(listing_id) {
            let new_rating = (listing.rating * listing.review_count as f64 + rating)
                / (listing.review_count as f64 + 1.0);
            listing.rating = new_rating.max(0.0).min(5.0);
            listing.review_count += 1;
        }
    }

    /// Deactivate a listing
    pub async fn delist(&self, listing_id: &str) {
        let mut listings = self.listings.write().await;
        if let Some(listing) = listings.get_mut(listing_id) {
            listing.active = false;
        }
    }
}

impl Default for CapabilityMarket {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pricing_model() {
        let fixed = PricingModel::Fixed { cost_per_use: 5.0 };
        assert_eq!(fixed.calculate_cost(1.0), 5.0);

        let per_unit = PricingModel::PerUnit { cost_per_unit: 2.0 };
        assert_eq!(per_unit.calculate_cost(10.0), 20.0);
    }

    #[test]
    fn test_capability_listing() {
        let listing = CapabilityListing::new(
            "provider-1".to_string(),
            "database.query".to_string(),
            PricingModel::Fixed { cost_per_use: 0.05 },
            ServiceLevelAgreement::default(),
        );

        assert!(listing.active);
        assert!(listing.value_score() > 0.0);
    }

    #[test]
    fn test_capability_contract() {
        let listing = CapabilityListing::new(
            "provider-1".to_string(),
            "compute".to_string(),
            PricingModel::PerUnit { cost_per_unit: 1.0 },
            ServiceLevelAgreement::default(),
        );

        let contract = CapabilityContract::new("buyer-1".to_string(), &listing, 100, 30);

        assert_eq!(contract.status, ContractStatus::Active);
        assert!(contract.is_active());
    }

    #[tokio::test]
    async fn test_marketplace() {
        let market = CapabilityMarket::new();

        let listing = CapabilityListing::new(
            "provider-1".to_string(),
            "database.query".to_string(),
            PricingModel::Fixed { cost_per_use: 0.05 },
            ServiceLevelAgreement::default(),
        );

        market.list_capability(listing.clone()).await;

        let found = market.find_capability("database.query").await;
        assert_eq!(found.len(), 1);

        let contract =
            market.create_contract("buyer-1".to_string(), &listing.listing_id, 10, 30).await;

        assert!(contract.is_some());
    }
}
