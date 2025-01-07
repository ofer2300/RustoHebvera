use std::sync::Arc;
use tokio::sync::RwLock;
use crate::ml::{ModelManager, PredictionEngine};
use crate::metrics::{PerformanceSnapshot, SystemSnapshot, BusinessSnapshot};

pub struct AnomalyDetector {
    model_manager: Arc<ModelManager>,
    prediction_engine: Arc<PredictionEngine>,
    performance_detector: Arc<PerformanceAnomalyDetector>,
    system_detector: Arc<SystemAnomalyDetector>,
    business_detector: Arc<BusinessAnomalyDetector>,
    pattern_detector: Arc<PatternAnomalyDetector>,
}

impl AnomalyDetector {
    pub fn new(config: &AnomalyConfig) -> Self {
        Self {
            model_manager: Arc::new(ModelManager::new(config.model_config())),
            prediction_engine: Arc::new(PredictionEngine::new(config.prediction_config())),
            performance_detector: Arc::new(PerformanceAnomalyDetector::new()),
            system_detector: Arc::new(SystemAnomalyDetector::new()),
            business_detector: Arc::new(BusinessAnomalyDetector::new()),
            pattern_detector: Arc::new(PatternAnomalyDetector::new()),
        }
    }

    pub async fn detect_anomalies(&self) -> Result<Vec<AnomalyReport>> {
        // איסוף נתונים היסטוריים
        let historical_data = self.model_manager.get_historical_data().await?;

        // חיזוי ערכים צפויים
        let predictions = self.prediction_engine.predict_next_values(&historical_data).await?;

        // זיהוי אנומליות במקביל
        let (performance_anomalies, system_anomalies, business_anomalies, pattern_anomalies) = tokio::join!(
            self.detect_performance_anomalies(&historical_data, &predictions),
            self.detect_system_anomalies(&historical_data, &predictions),
            self.detect_business_anomalies(&historical_data, &predictions),
            self.detect_pattern_anomalies(&historical_data, &predictions),
        );

        // מיזוג כל האנומליות שזוהו
        let mut anomalies = Vec::new();
        anomalies.extend(performance_anomalies?);
        anomalies.extend(system_anomalies?);
        anomalies.extend(business_anomalies?);
        anomalies.extend(pattern_anomalies?);

        // מיון לפי חומרה
        anomalies.sort_by(|a, b| b.severity.cmp(&a.severity));

        Ok(anomalies)
    }

    async fn detect_performance_anomalies(
        &self,
        historical_data: &HistoricalData,
        predictions: &Predictions,
    ) -> Result<Vec<AnomalyReport>> {
        self.performance_detector.detect(historical_data, predictions).await
    }

    async fn detect_system_anomalies(
        &self,
        historical_data: &HistoricalData,
        predictions: &Predictions,
    ) -> Result<Vec<AnomalyReport>> {
        self.system_detector.detect(historical_data, predictions).await
    }

    async fn detect_business_anomalies(
        &self,
        historical_data: &HistoricalData,
        predictions: &Predictions,
    ) -> Result<Vec<AnomalyReport>> {
        self.business_detector.detect(historical_data, predictions).await
    }

    async fn detect_pattern_anomalies(
        &self,
        historical_data: &HistoricalData,
        predictions: &Predictions,
    ) -> Result<Vec<AnomalyReport>> {
        self.pattern_detector.detect(historical_data, predictions).await
    }
}

pub struct PerformanceAnomalyDetector {
    latency_detector: LatencyAnomalyDetector,
    throughput_detector: ThroughputAnomalyDetector,
    error_detector: ErrorAnomalyDetector,
    cache_detector: CacheAnomalyDetector,
}

impl PerformanceAnomalyDetector {
    pub fn new() -> Self {
        Self {
            latency_detector: LatencyAnomalyDetector::new(),
            throughput_detector: ThroughputAnomalyDetector::new(),
            error_detector: ErrorAnomalyDetector::new(),
            cache_detector: CacheAnomalyDetector::new(),
        }
    }

    pub async fn detect(
        &self,
        historical_data: &HistoricalData,
        predictions: &Predictions,
    ) -> Result<Vec<AnomalyReport>> {
        let (latency, throughput, errors, cache) = tokio::join!(
            self.detect_latency_anomalies(historical_data, predictions),
            self.detect_throughput_anomalies(historical_data, predictions),
            self.detect_error_anomalies(historical_data, predictions),
            self.detect_cache_anomalies(historical_data, predictions),
        );

        let mut anomalies = Vec::new();
        anomalies.extend(latency?);
        anomalies.extend(throughput?);
        anomalies.extend(errors?);
        anomalies.extend(cache?);

        Ok(anomalies)
    }
}

pub struct SystemAnomalyDetector {
    resource_detector: ResourceAnomalyDetector,
    network_detector: NetworkAnomalyDetector,
    capacity_detector: CapacityAnomalyDetector,
}

impl SystemAnomalyDetector {
    pub fn new() -> Self {
        Self {
            resource_detector: ResourceAnomalyDetector::new(),
            network_detector: NetworkAnomalyDetector::new(),
            capacity_detector: CapacityAnomalyDetector::new(),
        }
    }

    pub async fn detect(
        &self,
        historical_data: &HistoricalData,
        predictions: &Predictions,
    ) -> Result<Vec<AnomalyReport>> {
        let (resources, network, capacity) = tokio::join!(
            self.detect_resource_anomalies(historical_data, predictions),
            self.detect_network_anomalies(historical_data, predictions),
            self.detect_capacity_anomalies(historical_data, predictions),
        );

        let mut anomalies = Vec::new();
        anomalies.extend(resources?);
        anomalies.extend(network?);
        anomalies.extend(capacity?);

        Ok(anomalies)
    }
}

pub struct BusinessAnomalyDetector {
    accuracy_detector: AccuracyAnomalyDetector,
    satisfaction_detector: SatisfactionAnomalyDetector,
    coverage_detector: CoverageAnomalyDetector,
    domain_detector: DomainAnomalyDetector,
}

impl BusinessAnomalyDetector {
    pub fn new() -> Self {
        Self {
            accuracy_detector: AccuracyAnomalyDetector::new(),
            satisfaction_detector: SatisfactionAnomalyDetector::new(),
            coverage_detector: CoverageAnomalyDetector::new(),
            domain_detector: DomainAnomalyDetector::new(),
        }
    }

    pub async fn detect(
        &self,
        historical_data: &HistoricalData,
        predictions: &Predictions,
    ) -> Result<Vec<AnomalyReport>> {
        let (accuracy, satisfaction, coverage, domain) = tokio::join!(
            self.detect_accuracy_anomalies(historical_data, predictions),
            self.detect_satisfaction_anomalies(historical_data, predictions),
            self.detect_coverage_anomalies(historical_data, predictions),
            self.detect_domain_anomalies(historical_data, predictions),
        );

        let mut anomalies = Vec::new();
        anomalies.extend(accuracy?);
        anomalies.extend(satisfaction?);
        anomalies.extend(coverage?);
        anomalies.extend(domain?);

        Ok(anomalies)
    }
}

pub struct PatternAnomalyDetector {
    time_series_detector: TimeSeriesAnomalyDetector,
    correlation_detector: CorrelationAnomalyDetector,
    seasonality_detector: SeasonalityAnomalyDetector,
    trend_detector: TrendAnomalyDetector,
}

impl PatternAnomalyDetector {
    pub fn new() -> Self {
        Self {
            time_series_detector: TimeSeriesAnomalyDetector::new(),
            correlation_detector: CorrelationAnomalyDetector::new(),
            seasonality_detector: SeasonalityAnomalyDetector::new(),
            trend_detector: TrendAnomalyDetector::new(),
        }
    }

    pub async fn detect(
        &self,
        historical_data: &HistoricalData,
        predictions: &Predictions,
    ) -> Result<Vec<AnomalyReport>> {
        let (time_series, correlations, seasonality, trends) = tokio::join!(
            self.detect_time_series_anomalies(historical_data, predictions),
            self.detect_correlation_anomalies(historical_data, predictions),
            self.detect_seasonality_anomalies(historical_data, predictions),
            self.detect_trend_anomalies(historical_data, predictions),
        );

        let mut anomalies = Vec::new();
        anomalies.extend(time_series?);
        anomalies.extend(correlations?);
        anomalies.extend(seasonality?);
        anomalies.extend(trends?);

        Ok(anomalies)
    }
} 