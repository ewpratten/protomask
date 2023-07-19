use prometheus_client::{
    metrics::{counter::Counter, family::Family},
    registry::Registry,
};
use tokio::sync::mpsc;

use super::labels::PacketsMetric;

/// A metric event (modification to a metric)
pub enum MetricEvent {
    CounterAdd(Metric),
}

/// A metric to be modified
pub enum Metric {
    Packets(PacketsMetric),
}

/// Sender that sends `MetricEvent`s
pub type MetricEventSender = mpsc::Sender<MetricEvent>;

/// Automated metric handler
pub struct MetricRegistry {
    /// Sender for things that produce metrics to inform the registry of changes
    sender: MetricEventSender,

    /// Receiver used to read incoming events
    receiver: mpsc::Receiver<MetricEvent>,

    // Handles on various metrics
    metric_packets: Family<PacketsMetric, Counter>,
}

impl MetricRegistry {
    /// Construct a new metric registry
    pub fn new() -> Self {
        // Build rx and tx
        let (sender, receiver) = mpsc::channel(100);

        // Create the internal registry
        let mut internal_registry = <Registry>::default();

        // Create and register each metric
        let metric_packets = Family::<PacketsMetric, Counter>::default();
        internal_registry.register(
            "packets",
            "Number of packets accepted, sent, or dropped",
            metric_packets.clone(),
        );

        Self {
            sender,
            receiver,
            metric_packets,
        }
    }

    /// Get a copy of the sender to allow a task to send metrics
    pub fn get_sender(&self) -> MetricEventSender {
        self.sender.clone()
    }

    /// Run the metric registry
    pub async fn run(&mut self) {
        // Process events in a loop
        loop {
            // Try to read an event
            match self.receiver.recv().await {
                Some(event) => match event {
                    MetricEvent::CounterAdd(metric) => match metric {
                        Metric::Packets(metric) => {
                            self.metric_packets.get_or_create(&metric).inc();
                        }
                    },
                },
                None => {
                    // The sender has been dropped, so we should exit
                    break;
                }
            }
        }
    }
}
