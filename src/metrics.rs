use lazy_static::lazy_static;
use prometheus::{Encoder, Histogram, HistogramOpts, Registry, TextEncoder};
use std::collections::HashMap;

lazy_static! {
    static ref REGISTRY: Registry = Registry::new();
    static ref LATENCY_HISTOGRAMS: HashMap<&'static str, Histogram> = {
        let mut map = HashMap::new();
        
        // Create histogram for each endpoint
        let endpoints = vec!["get_block", "submit_transaction", "get_dag_tips", "subscribe_utxo"];
        
        for endpoint in endpoints {
            let opts = HistogramOpts::new(
                format!("kaspa_rpc_{}_latency_ms", endpoint),
                format!("Latency for {} endpoint in milliseconds", endpoint),
            )
            .buckets(vec![1.0, 5.0, 10.0, 25.0, 50.0, 100.0, 250.0, 500.0, 1000.0]);
            
            let histogram = Histogram::with_opts(opts).unwrap();
            REGISTRY.register(Box::new(histogram.clone())).unwrap();
            map.insert(endpoint, histogram);
        }
        
        map
    };
}

/// Record latency for an endpoint
pub fn record_latency(endpoint: &str, latency_ms: f64) {
    if let Some(histogram) = LATENCY_HISTOGRAMS.get(endpoint) {
        histogram.observe(latency_ms);
    }
    
    // Log warning if latency exceeds target
    if latency_ms > 50.0 {
        tracing::warn!(
            endpoint = endpoint,
            latency_ms = latency_ms,
            "Latency exceeded 50ms target"
        );
    }
}

/// Export metrics in Prometheus format
pub fn export_metrics() -> String {
    let encoder = TextEncoder::new();
    let metric_families = REGISTRY.gather();
    let mut buffer = vec![];
    encoder.encode(&metric_families, &mut buffer).unwrap();
    String::from_utf8(buffer).unwrap()
}
