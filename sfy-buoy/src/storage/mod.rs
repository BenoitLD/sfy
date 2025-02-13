//! Stores data-packages to the SD-card.
//!
//! Every data-package is stored to the SD-card and queued for the Notecard. It should also be
//! possible to request a range of old packages.
//!
//! The maximum number of files in a FAT32 directory is 65536. If a data package has ID
//! `1234567` it is put in the directory: `123` and named `4567.axl`. The directory is the full
//! ID stripped of the last 4 digits, and the file name is the last 4 digits. At 52 Hz and 1024
//! length data-package, this should amount to 4389 files per day. Each directory will last a bit
//! longer than two days.

use embedded_sdmmc::{
    Controller, Error as GenericSdMmcError, Mode, SdMmcError, SdMmcSpi, VolumeIdx,
};

use ambiq_hal::gpio::pin::{Mode as SpiMode, P35 as CS};
use ambiq_hal::spi::Spi0 as Spi;

use crate::axl::AxlPacket;

mod clock;
mod handles;

use clock::CountClock;
use handles::*;

pub enum StorageErr {
    SdMmcErr(SdMmcError),
    GenericSdMmmcErr(GenericSdMmcError<SdMmcError>),
    ParseIDFailure,
}

impl From<SdMmcError> for StorageErr {
    fn from(e: SdMmcError) -> Self {
        StorageErr::SdMmcErr(e)
    }
}

impl From<embedded_sdmmc::Error<SdMmcError>> for StorageErr {
    fn from(e: embedded_sdmmc::Error<SdMmcError>) -> Self {
        StorageErr::GenericSdMmmcErr(e)
    }
}

const ID_FILE: &'static str = "sfy.id";

pub struct Storage {
    sd: SdMmcSpi<Spi, CS<{ SpiMode::Output }>>,
    /// Last written ID.
    current_id: u32,
}

impl Storage {
    pub fn open(spi: Spi, cs: CS<{ SpiMode::Output }>) -> Result<Storage, StorageErr> {
        // Get last id (or create file with 0, verify it's free, or scan)
        defmt::info!("Opening SD card..");

        let mut sd = SdMmcSpi::new(spi, cs);
        // TODO: Re-clock SPI

        defmt::info!("Initialize SD-card..");
        let current_id = {
            let block = sd.acquire()?;
            let sz = block.card_size_bytes()? / 1024_u64.pow(2);
            defmt::info!("SD card size: {} mb", sz);

            let mut c = Controller::new(block, CountClock);
            let mut v = c.get_volume(VolumeIdx(0))?;

            let mut root = DirHandle::open_root(&mut c, &mut v)?;
            let idf = root.open_file(ID_FILE, Mode::ReadOnly);

            let id = match idf {
                Ok(mut idf) => {
                    let mut buf = [0u8; 128];
                    idf.read(&mut buf)?;

                    // TODO: Handle corrupted ID file.
                    let buf = core::str::from_utf8(&buf).map_err(|_| StorageErr::ParseIDFailure)?;
                    let id =
                        u32::from_str_radix(&buf, 10).map_err(|_| StorageErr::ParseIDFailure)?;

                    Ok(id)
                }
                Err(GenericSdMmcError::FileNotFound) => Ok(0),
                Err(e) => Err(e),
            }?;

            id
        };

        Ok(Storage { sd, current_id })
    }

    /// Writes the current ID to SD-card.
    pub fn write_id(&mut self) -> Result<(), StorageErr> {
        Ok(())
    }

    /// Takes IMU queue and stores items.
    pub fn drain_queue(&mut self) -> Result<(), ()> {
        todo!()
    }

    pub fn current_id(&self) -> u32 {
        self.current_id
    }

    // Deserialize and return AxlPacket (without modifying sent status).
    pub fn get(&self, id: u32) -> Result<AxlPacket, StorageErr> {
        unimplemented!()
    }

    // Mark package as sent
    pub fn mark_sent(&mut self, id: u32) -> Result<(), StorageErr> {
        unimplemented!()
    }

    // Store a new package and mark it as unsent.
    pub fn store(&mut self, pck: AxlPacket) -> Result<u32, StorageErr> {
        // Store to id
        // Store unsent-status
        // Update current ID on disk
        // Update current ID in self
        unimplemented!()
    }
}

pub fn id_to_parts(id: u32) -> Result<(heapless::String<10>, heapless::String<8>), ()> {
    let dir = id / 10000;
    let file = id % 10000;

    let dir = heapless::String::from(dir);
    let mut file = heapless::String::from(file);
    file.push_str(".axl")?;

    Ok((dir, file))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_id_to_parts() {
        let (dir, file) = id_to_parts(0).unwrap();
        assert_eq!(dir, "0");
        assert_eq!(file, "0.axl");

        let (dir, file) = id_to_parts(1234567).unwrap();
        assert_eq!(dir, "123");
        assert_eq!(file, "4567.axl");
    }
}
