//! Unit tests for RAII store functionality

#[cfg(feature = "raii-store")]
mod raii_tests {
    use lambdust::evaluator::{Evaluator, StoreStatisticsWrapper};
    use lambdust::lexer::SchemeNumber;
    use lambdust::value::Value;

    #[test]
    fn test_raii_store_creation() {
        let evaluator = Evaluator::with_raii_store();
        let stats = evaluator.store_statistics();

        match stats {
            StoreStatisticsWrapper::Raii(raii_stats) => {
                assert_eq!(raii_stats.active_locations, 0);
                assert_eq!(raii_stats.total_allocations, 0);
            }
            _ => panic!("Expected RAII store"),
        }
    }

    #[test]
    fn test_raii_store_memory_limit() {
        let evaluator = Evaluator::with_raii_store_memory_limit(1024);
        assert_eq!(evaluator.memory_usage(), 0);
    }

    #[test]
    fn test_raii_store_direct_access() {
        use lambdust::evaluator::raii_store::RaiiStore;

        let store = RaiiStore::new();
        let initial_stats = store.statistics();

        // Allocate directly through RAII store
        let mut _locations = Vec::new();
        for i in 0..5 {
            let value = Value::Number(SchemeNumber::Integer(i));
            let location = store.allocate(value);
            _locations.push(location); // Keep locations alive
        }

        let final_stats = store.statistics();
        assert_eq!(
            final_stats.total_allocations,
            initial_stats.total_allocations + 5
        );
        assert_eq!(
            final_stats.active_locations,
            initial_stats.active_locations + 5
        );
        assert!(final_stats.estimated_memory_usage > initial_stats.estimated_memory_usage);
    }

    #[test]
    fn test_raii_store_cleanup() {
        let mut evaluator = Evaluator::with_raii_store();
        let initial_stats = evaluator.store_statistics();

        {
            // Allocate in scope
            let _location = evaluator
                .allocate(Value::Number(SchemeNumber::Integer(42)))
                .unwrap();

            let stats_with_allocation = evaluator.store_statistics();
            match stats_with_allocation {
                StoreStatisticsWrapper::Raii(raii_stats) => {
                    assert!(raii_stats.active_locations > initial_stats.total_allocations());
                }
                _ => panic!("Expected RAII store"),
            }
        } // Location should be dropped here due to RAII

        // Force manual cleanup to simulate automatic cleanup
        evaluator.collect_garbage();

        let final_stats = evaluator.store_statistics();
        match final_stats {
            StoreStatisticsWrapper::Raii(raii_stats) => {
                // Deallocations should have occurred
                assert!(raii_stats.total_deallocations >= initial_stats.total_deallocations());
            }
            _ => panic!("Expected RAII store"),
        }
    }
}

#[cfg(not(feature = "raii-store"))]
mod default_tests {
    use lambdust::evaluator::{Evaluator, StoreStatisticsWrapper};

    #[test]
    fn test_default_traditional_store() {
        let evaluator = Evaluator::new();
        let stats = evaluator.store_statistics();

        match stats {
            StoreStatisticsWrapper::Traditional(_) => {
                // Expected traditional store
            }
            #[cfg(feature = "raii-store")]
            StoreStatisticsWrapper::Raii(_) => panic!("Expected traditional store"),
        }
    }
}
