use std::sync::Arc;
use tokio::sync::RwLock;
use prometheus::{Registry, Counter, Gauge, Histogram};
use crate::analytics::AnalyticsEngine;

pub struct MetricsCollector {
    registry: Arc<Registry>,
    analytics: Arc<AnalyticsEngine>,
    performance_metrics: Arc<PerformanceMetrics>,
    system_metrics: Arc<SystemMetrics>,
    business_metrics: Arc<BusinessMetrics>,
    anomaly_detector: Arc<AnomalyDetector>,
}

impl MetricsCollector {
    pub fn new(config: &MetricsConfig) -> Self {
        let registry = Arc::new(Registry::new());
        
        Self {
            registry: registry.clone(),
            analytics: Arc::new(AnalyticsEngine::new(config.analytics_config())),
            performance_metrics: Arc::new(PerformanceMetrics::new(&registry)),
            system_metrics: Arc::new(SystemMetrics::new(&registry)),
            business_metrics: Arc::new(BusinessMetrics::new(&registry)),
            anomaly_detector: Arc::new(AnomalyDetector::new(config.anomaly_config())),
        }
    }

    pub fn start_request(&self) -> RequestMetrics {
        RequestMetrics::new(
            self.performance_metrics.clone(),
            self.system_metrics.clone(),
            self.business_metrics.clone(),
        )
    }

    pub async fn collect_metrics(&self) -> Result<MetricsSnapshot> {
        // איסוף מטריקות מקבילי
        let (performance, system, business, anomalies) = tokio::join!(
            self.performance_metrics.collect(),
            self.system_metrics.collect(),
            self.business_metrics.collect(),
            self.anomaly_detector.detect_anomalies(),
        );

        // ניתוח אנליטי
        let analytics = self.analytics.analyze(
            &performance?,
            &system?,
            &business?,
            &anomalies?
        ).await?;

        Ok(MetricsSnapshot {
            performance: performance?,
            system: system?,
            business: business?,
            anomalies: anomalies?,
            analytics,
            timestamp: chrono::Utc::now(),
        })
    }

    pub async fn export_metrics(&self, format: ExportFormat) -> Result<String> {
        let snapshot = self.collect_metrics().await?;
        
        match format {
            ExportFormat::Prometheus => self.export_prometheus(&snapshot),
            ExportFormat::JSON => self.export_json(&snapshot),
            ExportFormat::Dashboard => self.export_dashboard(&snapshot),
        }
    }
}

pub struct PerformanceMetrics {
    request_duration: Histogram,
    response_time: Histogram,
    throughput: Counter,
    error_rate: Counter,
    cache_hits: Counter,
    cache_misses: Counter,
}

impl PerformanceMetrics {
    pub fn new(registry: &Registry) -> Self {
        let request_duration = Histogram::new_with_opts(
            prometheus::HistogramOpts::new(
                "request_duration_seconds",
                "Request duration in seconds",
            )
            .buckets(vec![0.1, 0.5, 1.0, 2.0, 5.0]),
        ).unwrap();

        let response_time = Histogram::new_with_opts(
            prometheus::HistogramOpts::new(
                "response_time_seconds",
                "Response time in seconds",
            )
            .buckets(vec![0.05, 0.1, 0.25, 0.5, 1.0]),
        ).unwrap();

        let throughput = Counter::new(
            "requests_total",
            "Total number of requests",
        ).unwrap();

        let error_rate = Counter::new(
            "errors_total",
            "Total number of errors",
        ).unwrap();

        let cache_hits = Counter::new(
            "cache_hits_total",
            "Total number of cache hits",
        ).unwrap();

        let cache_misses = Counter::new(
            "cache_misses_total",
            "Total number of cache misses",
        ).unwrap();

        registry.register(Box::new(request_duration.clone())).unwrap();
        registry.register(Box::new(response_time.clone())).unwrap();
        registry.register(Box::new(throughput.clone())).unwrap();
        registry.register(Box::new(error_rate.clone())).unwrap();
        registry.register(Box::new(cache_hits.clone())).unwrap();
        registry.register(Box::new(cache_misses.clone())).unwrap();

        Self {
            request_duration,
            response_time,
            throughput,
            error_rate,
            cache_hits,
            cache_misses,
        }
    }

    pub async fn collect(&self) -> Result<PerformanceSnapshot> {
        Ok(PerformanceSnapshot {
            avg_request_duration: self.request_duration.get_sample_sum() / self.request_duration.get_sample_count(),
            avg_response_time: self.response_time.get_sample_sum() / self.response_time.get_sample_count(),
            requests_per_second: self.calculate_throughput(),
            error_percentage: self.calculate_error_rate(),
            cache_hit_ratio: self.calculate_cache_hit_ratio(),
        })
    }
}

pub struct SystemMetrics {
    cpu_usage: Gauge,
    memory_usage: Gauge,
    disk_usage: Gauge,
    network_traffic: Counter,
    active_connections: Gauge,
}

impl SystemMetrics {
    pub fn new(registry: &Registry) -> Self {
        let cpu_usage = Gauge::new(
            "cpu_usage_percent",
            "CPU usage percentage",
        ).unwrap();

        let memory_usage = Gauge::new(
            "memory_usage_bytes",
            "Memory usage in bytes",
        ).unwrap();

        let disk_usage = Gauge::new(
            "disk_usage_bytes",
            "Disk usage in bytes",
        ).unwrap();

        let network_traffic = Counter::new(
            "network_traffic_bytes",
            "Network traffic in bytes",
        ).unwrap();

        let active_connections = Gauge::new(
            "active_connections",
            "Number of active connections",
        ).unwrap();

        registry.register(Box::new(cpu_usage.clone())).unwrap();
        registry.register(Box::new(memory_usage.clone())).unwrap();
        registry.register(Box::new(disk_usage.clone())).unwrap();
        registry.register(Box::new(network_traffic.clone())).unwrap();
        registry.register(Box::new(active_connections.clone())).unwrap();

        Self {
            cpu_usage,
            memory_usage,
            disk_usage,
            network_traffic,
            active_connections,
        }
    }

    pub async fn collect(&self) -> Result<SystemSnapshot> {
        Ok(SystemSnapshot {
            cpu_usage: self.cpu_usage.get(),
            memory_usage: self.memory_usage.get(),
            disk_usage: self.disk_usage.get(),
            network_traffic: self.network_traffic.get(),
            active_connections: self.active_connections.get() as u64,
        })
    }
}

pub struct BusinessMetrics {
    translation_accuracy: Histogram,
    user_satisfaction: Histogram,
    technical_terms_coverage: Counter,
    domain_specific_accuracy: Counter,
}

impl BusinessMetrics {
    pub fn new(registry: &Registry) -> Self {
        let translation_accuracy = Histogram::new_with_opts(
            prometheus::HistogramOpts::new(
                "translation_accuracy_percent",
                "Translation accuracy percentage",
            )
            .buckets(vec![50.0, 75.0, 90.0, 95.0, 99.0]),
        ).unwrap();

        let user_satisfaction = Histogram::new_with_opts(
            prometheus::HistogramOpts::new(
                "user_satisfaction_score",
                "User satisfaction score",
            )
            .buckets(vec![1.0, 2.0, 3.0, 4.0, 5.0]),
        ).unwrap();

        let technical_terms_coverage = Counter::new(
            "technical_terms_total",
            "Total number of technical terms covered",
        ).unwrap();

        let domain_specific_accuracy = Counter::new(
            "domain_specific_accuracy_total",
            "Total number of accurate domain-specific translations",
        ).unwrap();

        registry.register(Box::new(translation_accuracy.clone())).unwrap();
        registry.register(Box::new(user_satisfaction.clone())).unwrap();
        registry.register(Box::new(technical_terms_coverage.clone())).unwrap();
        registry.register(Box::new(domain_specific_accuracy.clone())).unwrap();

        Self {
            translation_accuracy,
            user_satisfaction,
            technical_terms_coverage,
            domain_specific_accuracy,
        }
    }

    pub async fn collect(&self) -> Result<BusinessSnapshot> {
        Ok(BusinessSnapshot {
            avg_translation_accuracy: self.translation_accuracy.get_sample_sum() / self.translation_accuracy.get_sample_count(),
            avg_user_satisfaction: self.user_satisfaction.get_sample_sum() / self.user_satisfaction.get_sample_count(),
            technical_terms_coverage: self.technical_terms_coverage.get(),
            domain_specific_accuracy: self.domain_specific_accuracy.get(),
        })
    }
} 