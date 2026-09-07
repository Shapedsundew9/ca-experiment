//! Node micro-core dynamics: membrane potential accumulator, leak, refractory lockout,
//! history ring buffer, and local homeostatic adaptation.

use super::homeostasis::AdaptationMode;

#[derive(Debug, Clone)]
pub struct MicroNode {
    pub id: usize,
    /// Membrane accumulator potential V_i(t)
    pub v: f64,
    /// Refractory cooldown counter R_i(t)
    pub refractory_counter: usize,
    /// 8-bit historical spike register H_i(t)
    pub history: u8,
    /// Circular buffer tracking spikes across sliding window W_h
    pub window_buffer: Vec<u8>,
    /// Running sum of spikes in sliding window
    pub window_spike_sum: usize,
    /// Write pointer in circular window buffer
    pub window_ptr: usize,
    /// Current window capacity W_h
    pub window_size: usize,
    /// Current rolling firing rate \bar{r}_i(t)
    pub rolling_rate: f64,
    /// Dynamic firing threshold V_{thresh, i}(t)
    pub v_thresh: f64,
    /// Firing output emitted at current tick s_i(t)
    pub spike: u8,
    /// Total spikes fired by this node over the entire run
    pub total_spikes: usize,
}

impl MicroNode {
    pub fn new(id: usize, v_init: f64, window_size: usize) -> Self {
        Self {
            id,
            v: 0.0,
            refractory_counter: 0,
            history: 0,
            window_buffer: vec![0; window_size],
            window_spike_sum: 0,
            window_ptr: 0,
            window_size,
            rolling_rate: 0.0,
            v_thresh: v_init,
            spike: 0,
            total_spikes: 0,
        }
    }

    /// Reset node state to initial conditions
    pub fn reset(&mut self, v_init: f64) {
        self.v = 0.0;
        self.refractory_counter = 0;
        self.history = 0;
        self.window_buffer.fill(0);
        self.window_spike_sum = 0;
        self.window_ptr = 0;
        self.rolling_rate = 0.0;
        self.v_thresh = v_init;
        self.spike = 0;
        self.total_spikes = 0;
    }

    /// Advance node state by one discrete clock tick.
    ///
    /// Evaluates:
    /// 1. Refractory lockout check
    /// 2. Charge integration with leak factor \lambda
    /// 3. Spike emission and refractory initialization
    /// 4. History register shift
    /// 5. Rolling firing rate estimation over window W_h
    /// 6. Homeostatic threshold adaptation with projection to [v_min, v_max]
    #[inline]
    #[allow(clippy::too_many_arguments)]
    pub fn step(
        &mut self,
        tick: usize,
        incoming_charge: f64,
        leak: f64,
        n_ref: usize,
        mode: AdaptationMode,
        target_rate: f64,
        eta: f64,
        v_min: f64,
        v_max: f64,
        coin_flip: f64,
    ) {
        let spike: u8;

        if self.refractory_counter > 0 {
            // Refractory lockout active
            spike = 0;
            self.v = 0.0;
            self.refractory_counter -= 1;
        } else {
            // Receptive: charge integration with leak
            let v_pre = (1.0 - leak) * self.v + incoming_charge;
            if v_pre >= self.v_thresh {
                // Threshold reached: emit spike and enter refractory lockout
                spike = 1;
                self.v = 0.0;
                self.refractory_counter = n_ref;
                self.total_spikes += 1;
            } else {
                // Subthreshold: store integrated potential
                spike = 0;
                self.v = v_pre;
                self.refractory_counter = 0;
            }
        }

        self.spike = spike;

        // Advance 8-bit history register: prepend newest firing decision
        self.history = (self.history << 1) | spike;

        // Update rolling window in O(1) time
        let old_spike = self.window_buffer[self.window_ptr];
        self.window_spike_sum = self.window_spike_sum - (old_spike as usize) + (spike as usize);
        self.window_buffer[self.window_ptr] = spike;
        self.window_ptr = (self.window_ptr + 1) % self.window_size;

        let effective_w = if tick + 1 < self.window_size {
            tick + 1
        } else {
            self.window_size
        };
        self.rolling_rate = (self.window_spike_sum as f64) / (effective_w as f64);

        // Homeostatic threshold adaptation
        let delta_v = mode.compute_delta(self.rolling_rate, target_rate, eta, coin_flip);
        let unclipped_v = self.v_thresh + delta_v;
        self.v_thresh = unclipped_v.clamp(v_min, v_max);
    }
}
