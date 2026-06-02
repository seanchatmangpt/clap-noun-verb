// Integration examples: c8-time with market operations

#[cfg(test)]
mod market_integration_tests {
    use crate::{MonotonicStamp, VectorClock8, VectorClockCompare};

    /// Example: Multi-venue order flow with causal causality tracking
    ///
    /// Scenario:
    /// - Trader submits order to Venue A (instrument lane 0)
    /// - Venue A propagates to Venue B (venue lane 1)
    /// - Both venues execute concurrently on different instruments
    #[test]
    fn test_multi_venue_order_flow_causality() {
        // Lane 0 = Instrument dimension
        // Lane 1 = Venue A dimension
        // Lane 2 = Venue B dimension

        // Step 1: Trader submits order (instrument observes event)
        let mut trader_clock = VectorClock8::zero();
        trader_clock.tick_lane(0).expect("trader tick on instrument");

        // Step 2: Venue A receives order (receives trader clock, then ticks own lane)
        let mut venue_a_clock = VectorClock8::zero();
        venue_a_clock.merge(&trader_clock); // Observe trader's causality
        venue_a_clock.tick_lane(1).expect("venue A tick");

        // Verify: Venue A causally follows trader
        assert_eq!(
            venue_a_clock.compare(&trader_clock),
            VectorClockCompare::After,
            "Venue A should follow trader"
        );

        // Step 3: Venue B receives order independently (different timing)
        let mut venue_b_clock = VectorClock8::zero();
        venue_b_clock.merge(&trader_clock);
        venue_b_clock.tick_lane(2).expect("venue B tick");

        // Verify: Venue A and Venue B are concurrent (independent streams)
        assert_eq!(
            venue_a_clock.compare(&venue_b_clock),
            VectorClockCompare::Concurrent,
            "Venue A and Venue B execute concurrently"
        );

        // Step 4: Both venues execute on different instruments (lanes 3, 4)
        venue_a_clock.tick_lane(3).expect("venue A executes on instr");
        venue_b_clock.tick_lane(4).expect("venue B executes on instr");

        // Verify: Still concurrent (no new shared causality)
        assert_eq!(venue_a_clock.compare(&venue_b_clock), VectorClockCompare::Concurrent);
    }

    /// Example: Settlement coordination across venues
    ///
    /// Scenario:
    /// - Venue A fills an order and broadcasts settlement notification
    /// - Venue B receives settlement and updates counterparty exposure
    /// - Both must maintain causal ordering for regulatory compliance
    #[test]
    fn test_settlement_coordination_causality() {
        // Lane 0 = Settlement initiator (Venue A)
        // Lane 1 = Settlement responder (Venue B)

        let mut venue_a_settlement_clock = VectorClock8::zero();
        venue_a_settlement_clock.tick_lane(0).expect("fill order");

        let mut settlement_broadcast_clock = VectorClock8::zero();
        settlement_broadcast_clock.merge(&venue_a_settlement_clock);
        settlement_broadcast_clock.tick_lane(0).expect("broadcast settlement");

        let mut venue_b_settlement_clock = VectorClock8::zero();
        venue_b_settlement_clock.merge(&settlement_broadcast_clock);
        venue_b_settlement_clock.tick_lane(1).expect("receive settlement");

        // Verify complete causal chain: A -> broadcast -> B
        assert_eq!(
            settlement_broadcast_clock.compare(&venue_a_settlement_clock),
            VectorClockCompare::After
        );
        assert_eq!(
            venue_b_settlement_clock.compare(&settlement_broadcast_clock),
            VectorClockCompare::After
        );
    }

    /// Example: Monotonic time detection of out-of-order settlements
    ///
    /// Scenario:
    /// - Settlement message T1 arrives with monotonic_stamp=100
    /// - Out-of-order message T2 arrives with monotonic_stamp=90
    /// - System detects causality violation
    #[test]
    fn test_out_of_order_settlement_detection() {
        let t1_stamp = MonotonicStamp::from_value(100);
        let t2_stamp = MonotonicStamp::from_value(90);

        // T2 claims to be before T1 but arrived after: violation
        let result = t1_stamp.assert_not_before(&t2_stamp);
        assert!(result.is_ok(), "T1(100) >= T2(90) should be ok");

        let result = t2_stamp.assert_not_before(&t1_stamp);
        assert!(result.is_err(), "T2(90) < T1(100) is a regression");
    }

    /// Example: Market microstructure - liquidity event sequencing
    ///
    /// Scenario:
    /// - Market maker places limit orders (lane 0)
    /// - Market observes MM orders
    /// - Aggressive buyer comes in and hits orders (lane 1)
    /// - Market updates after trade execution
    /// - Verify correct cause-effect relationships
    #[test]
    fn test_liquidity_microstructure_sequencing() {
        let mut mm_clock = VectorClock8::zero();
        let mut market_clock = VectorClock8::zero();
        let mut buyer_clock = VectorClock8::zero();

        // MM places orders
        mm_clock.tick_lane(0).expect("mm place");

        // Market observes MM orders
        market_clock.merge(&mm_clock);
        market_clock.tick_lane(0).expect("market sees MM");

        // Buyer comes in and hits the order (merges market clock for causality)
        buyer_clock.merge(&market_clock);
        buyer_clock.tick_lane(1).expect("buyer hits");

        // Market updates after trade (merges buyer clock)
        market_clock.merge(&buyer_clock);
        market_clock.tick_lane(2).expect("spread updates");

        // Verify causal dependencies
        assert!(
            mm_clock.happens_before(&market_clock),
            "MM should precede initial market observation"
        );
        assert!(
            buyer_clock.happens_before(&market_clock),
            "Buyer action should precede market's final update after trade"
        );
    }

    /// Example: Conflict detection between concurrent operations
    ///
    /// Scenario:
    /// - Two traders on different venues execute simultaneously
    /// - Their orders do not interact (concurrent)
    /// - System detects no conflict
    #[test]
    fn test_concurrent_trader_orders_no_conflict() {
        let mut trader1_clock = VectorClock8::zero();
        let mut trader2_clock = VectorClock8::zero();

        // Independent executions on different lanes
        trader1_clock.tick_lane(0).expect("trader1");
        trader2_clock.tick_lane(1).expect("trader2");

        // Verify concurrency (no ordering relationship)
        assert_eq!(trader1_clock.compare(&trader2_clock), VectorClockCompare::Concurrent);

        // Safe to execute in any order since they're concurrent
        assert!(trader1_clock.is_concurrent(&trader2_clock));
    }

    /// Example: Regulatory timestamp audit trail
    ///
    /// Scenario:
    /// - Each trade must have monotonic evidence of submission time
    /// - Cannot backdate or reorder submissions
    #[test]
    fn test_regulatory_timestamp_audit() {
        let submission1 = MonotonicStamp::now();
        let submission2 = MonotonicStamp::now();
        let submission3 = MonotonicStamp::now();

        // Verify strict ordering for audit compliance
        assert!(submission1 < submission2);
        assert!(submission2 < submission3);

        // Verify no timestamp regressions possible
        let result = submission3.assert_not_before(&submission1);
        assert!(result.is_ok());

        let result = submission1.assert_strictly_after(&submission3);
        assert!(result.is_err()); // Cannot claim earlier submission was after later
    }

    /// Example: Construct8 causal depth estimation
    ///
    /// The sum of all vector clock lanes provides a rough estimate
    /// of causal depth (how many dependent operations have occurred)
    #[test]
    fn test_construct8_causal_depth() {
        let mut clock = VectorClock8::zero();

        // Single event
        clock.tick_lane(0).expect("tick");
        assert_eq!(clock.sum_lanes(), 1);

        // Causally dependent event
        clock.tick_lane(0).expect("tick");
        assert_eq!(clock.sum_lanes(), 2);

        // Merge from concurrent process
        let mut other = VectorClock8::zero();
        other.tick_lane(1).expect("other tick");
        other.tick_lane(1).expect("other tick");

        clock.merge(&other);
        assert_eq!(clock.sum_lanes(), 4); // 2 from lane 0 + 2 from lane 1
    }
}
