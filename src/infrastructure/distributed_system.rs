use tokio::sync::RwLock;
use std::sync::Arc;
use crate::cache::{MemoryCache, DiskCache, HybridCache};
use crate::load_balancer::LoadBalancer;
use crate::metrics::MetricsCollector;

pub struct DistributedSystem {
    hybrid_cache: Arc<HybridCache>,
    load_balancer: Arc<LoadBalancer>,
    metrics_collector: Arc<MetricsCollector>,
    resource_manager: Arc<ResourceManager>,
    failure_detector: Arc<FailureDetector>,
    replication_manager: Arc<ReplicationManager>,
}

impl DistributedSystem {
    pub fn new(config: &SystemConfig) -> Self {
        Self {
            hybrid_cache: Arc::new(HybridCache::new(config.cache_config())),
            load_balancer: Arc::new(LoadBalancer::new(config.balancer_config())),
            metrics_collector: Arc::new(MetricsCollector::new()),
            resource_manager: Arc::new(ResourceManager::new(config.resource_config())),
            failure_detector: Arc::new(FailureDetector::new(config.detector_config())),
            replication_manager: Arc::new(ReplicationManager::new(config.replication_config())),
        }
    }

    pub async fn process_request<T>(&self, request: Request<T>) -> Result<Response<T>> 
    where T: ProcessableData + Clone + Send + Sync + 'static 
    {
        // מדידת ביצועים
        let _metrics = self.metrics_collector.start_request();

        // בדיקת מטמון
        if let Some(cached) = self.hybrid_cache.get(&request).await? {
            return Ok(cached);
        }

        // בחירת צומת מתאים
        let node = self.load_balancer.select_node(&request).await?;
        
        // עיבוד מבוזר
        let result = self.process_distributed(request.clone(), node).await?;
        
        // שמירה במטמון
        self.hybrid_cache.store(&request, &result).await?;
        
        Ok(result)
    }

    async fn process_distributed<T>(
        &self,
        request: Request<T>,
        node: Node,
    ) -> Result<Response<T>> 
    where T: ProcessableData + Clone + Send + Sync + 'static 
    {
        // ניטור משאבים
        let _resources = self.resource_manager.monitor_resources(node.clone());
        
        // זיהוי כשלים
        let failure_guard = self.failure_detector.monitor_node(node.clone());
        
        // שכפול נתונים
        let replication = self.replication_manager.replicate_data(&request, &node).await?;

        // עיבוד הבקשה
        let result = node.process_request(request).await?;
        
        // עדכון שכפולים
        replication.update_replicas(&result).await?;

        Ok(result)
    }
}

pub struct HybridCache {
    memory_cache: Arc<RwLock<MemoryCache>>,
    disk_cache: Arc<RwLock<DiskCache>>,
    eviction_policy: Arc<EvictionPolicy>,
    metrics: Arc<CacheMetrics>,
}

impl HybridCache {
    pub fn new(config: CacheConfig) -> Self {
        Self {
            memory_cache: Arc::new(RwLock::new(MemoryCache::new(config.memory_config()))),
            disk_cache: Arc::new(RwLock::new(DiskCache::new(config.disk_config()))),
            eviction_policy: Arc::new(EvictionPolicy::new(config.eviction_config())),
            metrics: Arc::new(CacheMetrics::new()),
        }
    }

    pub async fn get<K, V>(&self, key: &K) -> Result<Option<V>> 
    where 
        K: CacheKey + Clone,
        V: CacheValue + Clone,
    {
        // בדיקה במטמון זיכרון
        if let Some(value) = self.memory_cache.read().await.get(key)? {
            self.metrics.record_memory_hit();
            return Ok(Some(value));
        }

        // בדיקה במטמון דיסק
        if let Some(value) = self.disk_cache.read().await.get(key)? {
            self.metrics.record_disk_hit();
            
            // העברה למטמון זיכרון
            self.promote_to_memory(key, &value).await?;
            
            return Ok(Some(value));
        }

        self.metrics.record_miss();
        Ok(None)
    }

    pub async fn store<K, V>(&self, key: K, value: V) -> Result<()> 
    where 
        K: CacheKey + Clone,
        V: CacheValue + Clone,
    {
        // שמירה במטמון זיכרון
        let mut memory_cache = self.memory_cache.write().await;
        
        // בדיקת מדיניות פינוי
        if memory_cache.needs_eviction() {
            let evicted = self.eviction_policy.select_for_eviction(&memory_cache)?;
            
            // העברה למטמון דיסק
            for (k, v) in evicted {
                self.disk_cache.write().await.store(k, v)?;
                memory_cache.remove(&k)?;
            }
        }

        memory_cache.store(key.clone(), value.clone())?;
        
        // שמירה במטמון דיסק
        self.disk_cache.write().await.store(key, value)?;

        Ok(())
    }

    async fn promote_to_memory<K, V>(&self, key: &K, value: &V) -> Result<()> 
    where 
        K: CacheKey + Clone,
        V: CacheValue + Clone,
    {
        let mut memory_cache = self.memory_cache.write().await;
        
        // פינוי אם נדרש
        if memory_cache.needs_eviction() {
            let evicted = self.eviction_policy.select_for_eviction(&memory_cache)?;
            
            for (k, v) in evicted {
                self.disk_cache.write().await.store(k, v)?;
                memory_cache.remove(&k)?;
            }
        }

        memory_cache.store(key.clone(), value.clone())?;
        Ok(())
    }
}

pub struct LoadBalancer {
    nodes: Arc<RwLock<Vec<Node>>>,
    strategy: Arc<BalancingStrategy>,
    health_checker: Arc<HealthChecker>,
    metrics: Arc<LoadMetrics>,
}

impl LoadBalancer {
    pub fn new(config: BalancerConfig) -> Self {
        Self {
            nodes: Arc::new(RwLock::new(Vec::new())),
            strategy: Arc::new(BalancingStrategy::new(config.strategy_config())),
            health_checker: Arc::new(HealthChecker::new(config.health_config())),
            metrics: Arc::new(LoadMetrics::new()),
        }
    }

    pub async fn select_node<T>(&self, request: &Request<T>) -> Result<Node> 
    where T: ProcessableData 
    {
        // בדיקת בריאות צמתים
        let healthy_nodes = self.health_checker.get_healthy_nodes().await?;
        
        // בחירת צומת לפי אסטרטגיה
        let selected = self.strategy.select_node(
            &healthy_nodes,
            request,
            &self.metrics
        ).await?;

        // עדכון מטריקות
        self.metrics.record_selection(&selected);

        Ok(selected)
    }
} 