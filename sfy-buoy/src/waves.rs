//! Measure waves using an IMU, feed it through a Kalman filter and collect
//! time-series or statistics.

use ahrs_fusion::NxpFusion;
use core::fmt::Debug;
use embedded_hal::blocking::{
    delay::DelayMs,
    i2c::{Write, WriteRead},
};
use ism330dhcx::{ctrl1xl, ctrl2g, fifo, fifoctrl, Ism330Dhcx};

/// The installed IMU.
pub type IMU = Ism330Dhcx;

pub struct Waves<I2C: WriteRead + Write> {
    pub i2c: I2C,
    pub imu: IMU,
    #[allow(unused)]
    filter: NxpFusion,
}

impl<E: Debug, I2C: WriteRead<Error = E> + Write<Error = E>> Waves<I2C> {
    pub fn new(mut i2c: I2C) -> Result<Waves<I2C>, E> {
        defmt::debug!("pinging imu..");
        i2c.write(0x6a, &[])?;

        defmt::debug!("setting up imu driver..");
        let imu = Ism330Dhcx::new_with_address(&mut i2c, 0x6a)?;

        let mut w = Waves {
            i2c,
            imu,
            filter: NxpFusion::new(5.),
        };

        defmt::debug!("booting imu..");
        w.boot_imu()?;
        w.disable_fifo()?;

        // TODO: Turn off magnetometer.

        Ok(w)
    }

    /// Temperature in Celsius.
    pub fn get_temperature(&mut self) -> Result<f32, E> {
        self.imu.get_temperature(&mut self.i2c)
    }

    /// Booting the sensor accoring to Adafruit's driver
    fn boot_imu(&mut self) -> Result<(), E> {
        let sensor = &mut self.imu;
        let i2c = &mut self.i2c;

        // =======================================
        // CTRL3_C

        sensor.ctrl3c.set_boot(i2c, true)?;
        sensor.ctrl3c.set_bdu(i2c, true)?;
        sensor.ctrl3c.set_if_inc(i2c, true)?;

        // =======================================
        // CTRL9_XL

        sensor.ctrl9xl.set_den_x(i2c, true)?;
        sensor.ctrl9xl.set_den_y(i2c, true)?;
        sensor.ctrl9xl.set_den_z(i2c, true)?;
        sensor.ctrl9xl.set_device_conf(i2c, true)?;

        // =======================================
        // CTRL1_XL

        // Use this for manually sampling.
        // sensor
        //     .ctrl1xl
        //     .set_accelerometer_data_rate(i2c, ctrl1xl::Odr_Xl::Hz833)?;

        sensor
            .ctrl1xl
            .set_chain_full_scale(i2c, ctrl1xl::Fs_Xl::G4)?;
        sensor.ctrl1xl.set_lpf2_xl_en(i2c, true)?;

        // =======================================
        // CTRL2_G

        // Use this for manually sampling.
        // sensor
        //     .ctrl2g
        //     .set_gyroscope_data_rate(i2c, ctrl2g::Odr::Hz833)?;

        sensor
            .ctrl2g
            .set_chain_full_scale(i2c, ctrl2g::Fs::Dps500)?;

        // =======================================
        // CTRL7_G

        sensor.ctrl7g.set_g_hm_mode(i2c, true)?;

        Ok(())
    }

    pub fn enable_fifo(&mut self, delay: &mut impl DelayMs<u16>) -> Result<(), E> {
        defmt::debug!("enabling FIFO mode");

        let i2c = &mut self.i2c;

        // Reset FIFO
        self.imu.fifoctrl.mode(i2c, fifoctrl::FifoMode::Bypass)?;
        self.imu
            .fifoctrl
            .set_accelerometer_batch_data_rate(i2c, fifoctrl::BdrXl::Hz417)?;
        self.imu
            .fifoctrl
            .set_gyroscope_batch_data_rate(i2c, fifoctrl::BdrGy::Hz417)?;

        // Wait for FIFO to be cleared.
        delay.delay_ms(10);

        // Start FIFO. The FIFO will fill up and stop if it is not emptied fast enough.
        self.imu.fifoctrl.mode(i2c, fifoctrl::FifoMode::FifoMode)?;

        Ok(())
    }

    /// Disable FIFO mode (this also resets the FIFO).
    pub fn disable_fifo(&mut self) -> Result<(), E> {
        self.imu
            .fifoctrl
            .mode(&mut self.i2c, fifoctrl::FifoMode::Bypass)
    }

    /// Returns iterator with all the currently available samples in the FIFO.
    pub fn consume_fifo(&mut self) -> Result<impl Iterator<Item = Result<fifo::Value, E>> + '_, E> {
        let n = self.imu.fifostatus.diff_fifo(&mut self.i2c)?;
        defmt::debug!("consuming {} samples from FIFO..", n);
        Ok((0..n).map(|_| self.imu.fifo_pop(&mut self.i2c)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use embedded_hal_mock::i2c::{Mock, Transaction};

    #[test]
    fn update_filter() {
        let expectations = [
            Transaction::write(0xaa, vec![1, 2]),
            Transaction::read(0xbb, vec![3, 4]),
        ];
        let mut i2c = Mock::new(&expectations);

        let mut w = Waves::new(i2c);
        w.filter.update(0.1, 0.2, 0.3, 0.3, 4., 0.5, 0., 0., 0.);
    }
}
