use quarry_ledger::{Library, LookupQuery};

#[test]
fn validates_a_lookup_query_before_reading_storage() {
    assert!(LookupQuery::new("customer-17").is_ok());
    assert!(LookupQuery::new("").is_err());
    assert!(LookupQuery::new("customer\t17").is_err());
}

#[test]
fn library_entry_point_opens_a_local_segment_store() {
    let library = Library::open("fixture/index.meta").unwrap();
    let query = LookupQuery::new("customer-17").unwrap();
    // The fixture directory is deliberately not part of this source test.
    // The IO path is observable through find_record's typed result.
    assert!(library.find_record(&query).is_err());
}
