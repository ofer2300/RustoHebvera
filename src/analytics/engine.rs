use std::sync::Arc;
use tokio::sync::RwLock;
use crate::metrics::{PerformanceSnapshot, SystemSnapshot, BusinessSnapshot, AnomalyReport};
use crate::ml::{PredictionEngine, ModelManager};

pub struct AnalyticsEngine {
    prediction_engine: Arc<PredictionEngine>,
    model_manager: Arc<ModelManager>,
    performance_analyzer: Arc<PerformanceAnalyzer>,
    system_analyzer: Arc<SystemAnalyzer>,
    business_analyzer: Arc<BusinessAnalyzer>,
    trend_analyzer: Arc<TrendAnalyzer>,
}

impl AnalyticsEngine {
    pub fn new(config: &AnalyticsConfig) -> Self {
        Self {
            prediction_engine: Arc::new(PredictionEngine::new(config.prediction_config())),
            model_manager: Arc::new(ModelManager::new(config.model_config())),
            performance_analyzer: Arc::new(PerformanceAnalyzer::new()),
            system_analyzer: Arc::new(SystemAnalyzer::new()),
            business_analyzer: Arc::new(BusinessAnalyzer::new()),
            trend_analyzer: Arc::new(TrendAnalyzer::new()),
        }
    }

    pub async fn analyze(
        &self,
        performance: &PerformanceSnapshot,
        system: &SystemSnapshot,
        business: &BusinessSnapshot,
        anomalies: &[AnomalyReport],
    ) -> Result<AnalyticsReport> {
        // ניתוח מקבילי של כל המדדים
        let (perf_analysis, sys_analysis, bus_analysis, trends) = tokio::join!(
            self.analyze_performance(performance),
            self.analyze_system(system),
            self.analyze_business(business),
            self.analyze_trends(performance, system, business),
        );

        // חיזוי מגמות עתידיות
        let predictions = self.prediction_engine.predict(
            &perf_analysis?,
            &sys_analysis?,
            &bus_analysis?,
            &trends?,
        ).await?;

        // עדכון מודלים
        self.model_manager.update_models(
            performance,
            system,
            business,
            &predictions,
        ).await?;

        Ok(AnalyticsReport {
            performance_analysis: perf_analysis?,
            system_analysis: sys_analysis?,
            business_analysis: bus_analysis?,
            trend_analysis: trends?,
            predictions,
            anomalies: anomalies.to_vec(),
            timestamp: chrono::Utc::now(),
        })
    }

    async fn analyze_performance(&self, snapshot: &PerformanceSnapshot) -> Result<PerformanceAnalysis> {
        self.performance_analyzer.analyze(snapshot).await
    }

    async fn analyze_system(&self, snapshot: &SystemSnapshot) -> Result<SystemAnalysis> {
        self.system_analyzer.analyze(snapshot).await
    }

    async fn analyze_business(&self, snapshot: &BusinessSnapshot) -> Result<BusinessAnalysis> {
        self.business_analyzer.analyze(snapshot).await
    }

    async fn analyze_trends(
        &self,
        performance: &PerformanceSnapshot,
        system: &SystemSnapshot,
        business: &BusinessSnapshot,
    ) -> Result<TrendAnalysis> {
        self.trend_analyzer.analyze(performance, system, business).await
    }
}

pub struct PerformanceAnalyzer {
    latency_analyzer: LatencyAnalyzer,
    throughput_analyzer: ThroughputAnalyzer,
    error_analyzer: ErrorAnalyzer,
    cache_analyzer: CacheAnalyzer,
}

impl PerformanceAnalyzer {
    pub fn new() -> Self {
        Self {
            latency_analyzer: LatencyAnalyzer::new(),
            throughput_analyzer: ThroughputAnalyzer::new(),
            error_analyzer: ErrorAnalyzer::new(),
            cache_analyzer: CacheAnalyzer::new(),
        }
    }

    pub async fn analyze(&self, snapshot: &PerformanceSnapshot) -> Result<PerformanceAnalysis> {
        let (latency, throughput, errors, cache) = tokio::join!(
            self.analyze_latency(snapshot),
            self.analyze_throughput(snapshot),
            self.analyze_errors(snapshot),
            self.analyze_cache(snapshot),
        );

        Ok(PerformanceAnalysis {
            latency_analysis: latency?,
            throughput_analysis: throughput?,
            error_analysis: errors?,
            cache_analysis: cache?,
        })
    }
}

pub struct SystemAnalyzer {
    resource_analyzer: ResourceAnalyzer,
    network_analyzer: NetworkAnalyzer,
    capacity_analyzer: CapacityAnalyzer,
}

impl SystemAnalyzer {
    pub fn new() -> Self {
        Self {
            resource_analyzer: ResourceAnalyzer::new(),
            network_analyzer: NetworkAnalyzer::new(),
            capacity_analyzer: CapacityAnalyzer::new(),
        }
    }

    pub async fn analyze(&self, snapshot: &SystemSnapshot) -> Result<SystemAnalysis> {
        let (resources, network, capacity) = tokio::join!(
            self.analyze_resources(snapshot),
            self.analyze_network(snapshot),
            self.analyze_capacity(snapshot),
        );

        Ok(SystemAnalysis {
            resource_analysis: resources?,
            network_analysis: network?,
            capacity_analysis: capacity?,
        })
    }
}

pub struct BusinessAnalyzer {
    accuracy_analyzer: AccuracyAnalyzer,
    satisfaction_analyzer: SatisfactionAnalyzer,
    coverage_analyzer: CoverageAnalyzer,
    domain_analyzer: DomainAnalyzer,
}

impl BusinessAnalyzer {
    pub fn new() -> Self {
        Self {
            accuracy_analyzer: AccuracyAnalyzer::new(),
            satisfaction_analyzer: SatisfactionAnalyzer::new(),
            coverage_analyzer: CoverageAnalyzer::new(),
            domain_analyzer: DomainAnalyzer::new(),
        }
    }

    pub async fn analyze(&self, snapshot: &BusinessSnapshot) -> Result<BusinessAnalysis> {
        let (accuracy, satisfaction, coverage, domain) = tokio::join!(
            self.analyze_accuracy(snapshot),
            self.analyze_satisfaction(snapshot),
            self.analyze_coverage(snapshot),
            self.analyze_domain(snapshot),
        );

        Ok(BusinessAnalysis {
            accuracy_analysis: accuracy?,
            satisfaction_analysis: satisfaction?,
            coverage_analysis: coverage?,
            domain_analysis: domain?,
        })
    }
}

pub struct TrendAnalyzer {
    time_series_analyzer: TimeSeriesAnalyzer,
    pattern_detector: PatternDetector,
    correlation_analyzer: CorrelationAnalyzer,
    seasonality_detector: SeasonalityDetector,
}

impl TrendAnalyzer {
    pub fn new() -> Self {
        Self {
            time_series_analyzer: TimeSeriesAnalyzer::new(),
            pattern_detector: PatternDetector::new(),
            correlation_analyzer: CorrelationAnalyzer::new(),
            seasonality_detector: SeasonalityDetector::new(),
        }
    }

    pub async fn analyze(
        &self,
        performance: &PerformanceSnapshot,
        system: &SystemSnapshot,
        business: &BusinessSnapshot,
    ) -> Result<TrendAnalysis> {
        let (time_series, patterns, correlations, seasonality) = tokio::join!(
            self.analyze_time_series(performance, system, business),
            self.detect_patterns(performance, system, business),
            self.analyze_correlations(performance, system, business),
            self.detect_seasonality(performance, system, business),
        );

        Ok(TrendAnalysis {
            time_series_analysis: time_series?,
            pattern_analysis: patterns?,
            correlation_analysis: correlations?,
            seasonality_analysis: seasonality?,
        })
    }
} 