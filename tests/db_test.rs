mod common;

#[test]
fn test_in_memory_connection() {
    let pool = common::setup_test_pool();
    let conn = pool.get();
    assert!(conn.is_ok());
}
