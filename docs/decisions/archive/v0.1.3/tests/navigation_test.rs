use std::sync::Arc;
use uuid::Uuid;
use scripture_os::repository::postgres::PostgresRepository;
use scripture_os::engines::TraversalEngine;
use scripture_os::engines::traversal::CoreTraversalEngine;
use scripture_os::test_utils;

#[tokio::test]
async fn test_structural_discovery_and_adjacency() {
    let pool = test_utils::setup_db().await;
    test_utils::seed_universal_data(&pool).await;

    let repo = Arc::new(PostgresRepository::new(pool));
    let engine = CoreTraversalEngine::new(repo);

    // Target: Hafs Sura 1:1 (Basmala)
    let hafs_1_1_id = Uuid::parse_str("00000000-0000-0000-0000-000000000A06").unwrap();

    // Architectural Design Decision: Contextual Continuity
    // Use get_adjacent_nodes to find siblings in the hierarchy
    let adjacency = engine.get_adjacent_nodes(hafs_1_1_id).await.unwrap();

    assert!(adjacency.previous.is_none());
    assert_eq!(adjacency.next.unwrap().path, "hafs.sura.1.2");
}