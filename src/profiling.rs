use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TimerScope {
    Ipv4ToIpv6,
    Ipv6ToIpv4,
    IcmpToIcmpv6,
    Icmpv6ToIcmp,
    Udp,
    Tcp,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum TimerState {
    Started(Instant, u8),
    Ended(Duration, u8),
}

pub struct PacketTimer {
    protocol: u8,
    creation_time: Instant,
    states: HashMap<TimerScope, TimerState>,
}

impl PacketTimer {
    /// Construct a new `PacketTimer`
    pub fn new(protocol: u8) -> Self {
        Self {
            protocol,
            creation_time: Instant::now(),
            states: HashMap::new(),
        }
    }

    pub fn start(&mut self, scope: TimerScope) {
        match self.states.get(&scope) {
            // If we have already started a timer for this scope, just increment the counter for "number of times started"
            Some(TimerState::Started(instant, start_count)) => {
                self.states
                    .insert(scope, TimerState::Started(*instant, start_count + 1));
            }
            // This should never happen
            Some(TimerState::Ended(_, _)) => {
                unreachable!(
                    "Timer already ended for scope {:?}. Not starting again.",
                    scope
                );
            }
            // If the timer hasn't been started yet, start it
            None => {
                self.states
                    .insert(scope, TimerState::Started(Instant::now(), 0));
            }
        }
    }

    pub fn end(&mut self, scope: TimerScope) {
        match self.states.get(&scope) {
            // If the timer is currently "started", either decrement the counter or end the timer
            Some(TimerState::Started(instant, start_count)) => {
                if *start_count == 0 {
                    self.states
                        .insert(scope, TimerState::Ended(instant.elapsed(), *start_count));
                } else {
                    self.states
                        .insert(scope, TimerState::Started(*instant, start_count - 1));
                }
            }
            // Ending an already ended timer does nothing
            Some(TimerState::Ended(_, _)) => {
                log::error!(
                    "Timer already ended for scope {:?}. Not ending again.",
                    scope
                );
            }
            // Ending a timer that never started should be impossible
            None => {
                unreachable!("Timer not started for scope {:?}. Not ending.", scope);
            }
        }
    }

    pub fn log(&self) {
        log::trace!(
            "[Timer report] proto: {}, total: {:05}µs, ipv4_to_ipv6: {:05}µs, ipv6_to_ipv4: {:05}µs, icmp_to_icmpv6: {:05}µs, icmpv6_to_icmp: {:05}µs, udp: {:05}µs, tcp: {:05}µs",
            self.protocol,
            self.creation_time.elapsed().as_micros(),
            self.states.get(&TimerScope::Ipv4ToIpv6).map(|val| match val {
                TimerState::Ended(duration, _) => duration.as_micros(),
                _=>0
            }).unwrap_or(0),
            self.states.get(&TimerScope::Ipv6ToIpv4).map(|val| match val {
                TimerState::Ended(duration, _) => duration.as_micros(),
                _=>0
            }).unwrap_or(0),
            self.states.get(&TimerScope::IcmpToIcmpv6).map(|val| match val {
                TimerState::Ended(duration, _) => duration.as_micros(),
                _=>0
            }).unwrap_or(0),
            self.states.get(&TimerScope::Icmpv6ToIcmp).map(|val| match val {
                TimerState::Ended(duration, _) => duration.as_micros(),
                _=>0
            }).unwrap_or(0),
            self.states.get(&TimerScope::Udp).map(|val| match val {
                TimerState::Ended(duration, _) => duration.as_micros(),
                _=>0
            }).unwrap_or(0),
            self.states.get(&TimerScope::Tcp).map(|val| match val {
                TimerState::Ended(duration, _) => duration.as_micros(),
                _=>0
            }).unwrap_or(0),
        );
    }
}
