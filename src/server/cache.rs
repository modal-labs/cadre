//! Implementation of data caching for slow resolvers.

use std::borrow::Borrow;
use std::collections::HashMap;
use std::hash::Hash;

use parking_lot::Mutex;
use tokio::time::{Duration, Instant};

/// A timed cache, with random jitter on the time-to-live.
#[derive(Debug)]
pub struct TimedCache<K, V> {
    inner: Mutex<HashMap<K, (Instant, V)>>,
    min_ttl: f64,
    max_ttl: f64,
}

impl<K: Eq + Hash, V: Clone> TimedCache<K, V> {
    /// Create a new timed cache with min and max TTLs.
    pub fn new(min_ttl: f64, max_ttl: f64) -> Self {
        Self {
            inner: Default::default(),
            min_ttl,
            max_ttl,
        }
    }

    /// Get an entry from the cache, checking if it has expired.
    pub fn get<Q>(&self, k: &Q) -> Option<V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        let mut inner = self.inner.lock();
        let (expire_time, value) = inner.get(k)?;
        if *expire_time < Instant::now() {
            Some(value.clone())
        } else {
            inner.remove(k);
            None
        }
    }

    /// Set an entry in the cache, with a random expiration time.
    pub fn insert(&self, k: K, v: V) {
        let ttl = self.min_ttl + fastrand::f64() * (self.max_ttl - self.min_ttl);
        let expire_time = Instant::now() + Duration::from_secs_f64(ttl);
        self.inner.lock().insert(k, (expire_time, v));
    }
}
