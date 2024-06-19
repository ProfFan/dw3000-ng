use crate::hl::receiving::CarrierRecoveryIntegrator;
use defmt::Format;

/// A struct representing the carrier frequency offset of the received message.
#[cfg_attr(feature = "defmt", derive(Format))]
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct CarrierFreqOffset {
    /// Contains the carrier frequency offset in Hertz, as per
    /// DW3000 user manual 8.2.7.6.
    pub f_offset_hz: f64,

    /// Contains the carrier frequency offset in PPM, giving similar
    /// results as the `dwt_readclockoffset` function of the DWM3000 SDK.
    /// NOTE: Positive value means that the local receiver’s clock is running
    /// faster than that of the remote transmitter.
    pub f_offset_ppm: f64,
}

impl CarrierFreqOffset {
    /// This constructor calculates the carrier frequency offset from a CarrierRecoveryIntegrator and the center frequency of the channel
    pub fn from_drx_car_int(
        carrier_integrator: CarrierRecoveryIntegrator,
        f_c: u64,
    ) -> CarrierFreqOffset {
        // Convert it to a f64
        let drx_car_int: f64 = carrier_integrator.value() as f64;

        // F_S / 2 / N_samples / 2^17   (N_samples = 1024 for DW3000)
        const DW_FREQ_OFFSET_MULTIPLIER: f64 = 998.4e6 / 2.0 / 1024.0 / 131072.0;

        // Compute f_offset in hertz
        let f_offset_hz = drx_car_int * DW_FREQ_OFFSET_MULTIPLIER;

        CarrierFreqOffset::from_f_offset_hz(f_offset_hz, f_c)
    }

    /// This constructor calculates the carrier frequency offset in ppm from a carrier_frequency_offset in Hertz and the center frequency of the channel
    /// NOTE: The output has the same sign as the `dwt_readclockoffset` function of the DWM3000 SDK, and not as the computation in DW3000 user manual 8.2.7.6.
    /// The computation follows from DW3000 user manual 8.2.7.6, without the negative sign.
    pub fn from_f_offset_hz(f_offset_hz: f64, f_c: u64) -> CarrierFreqOffset {
        // Compute f_offset in ppm w.r.t to channel frequency
        let f_offset_ppm = 1e6 * f_offset_hz / (f_c as f64);

        CarrierFreqOffset {
            f_offset_hz,
            f_offset_ppm,
        }
    }
}
