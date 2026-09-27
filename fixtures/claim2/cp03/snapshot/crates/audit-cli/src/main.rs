use query_router::RoutePlan;
use recovery_journal::RecoveryMarker;
use wire_schema::RecordEnvelope;

fn main() {
    let plan = RoutePlan::for_protocol(4);
    let marker = RecoveryMarker::new(17, "release-snapshot");
    let record = RecordEnvelope::new("release", "ready");
    println!("{}:{}:{}", plan.accepts(record.wire_revision), marker.offset, record.payload);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_path_accepts_the_current_protocol_envelope() {
        let plan = RoutePlan::for_protocol(4);
        let marker = RecoveryMarker::new(5, "snapshot");
        let record = RecordEnvelope::new("build", "verified");
        assert!(plan.accepts(record.wire_revision));
        assert_eq!(marker.segment, "snapshot");
        assert_eq!(record.payload, "verified");
    }
}
