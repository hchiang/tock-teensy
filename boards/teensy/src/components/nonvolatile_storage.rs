use mk66;
use capsules::nonvolatile_storage_driver::{NonvolatileStorage, BUFFER};
use capsules::nonvolatile_to_pages::NonvolatileToPages;
use components::Component;
use kernel::hil;
use kernel;

pub struct NonvolatileStorageComponent;

impl NonvolatileStorageComponent {
    pub fn new() -> Self {
        NonvolatileStorageComponent {}
    }
}

impl Component for NonvolatileStorageComponent {
    type Output = &'static NonvolatileStorage<'static>;

    unsafe fn finalize(&mut self) -> Option<Self::Output> {
        mk66::flash::FLASH_CONTROLLER.configure();
        pub static mut FLASH_PAGEBUFFER: mk66::flash::K66Sector =
            mk66::flash::K66Sector::new();
        let nv_to_page = static_init!(
            NonvolatileToPages<'static, mk66::flash::FTFE>,
            NonvolatileToPages::new(
                &mut mk66::flash::FLASH_CONTROLLER,
                &mut FLASH_PAGEBUFFER
            )
        );
        hil::flash::HasClient::set_client(&mk66::flash::FLASH_CONTROLLER, nv_to_page);

        extern "C" {
            /// Beginning on the ROM region containing app images.
            static _sstorage: u8;
            static _estorage: u8;
        }

        // Flash locations from Datasheet 4.2
        let nonvolatile_storage = static_init!(
            NonvolatileStorage<'static>,
            NonvolatileStorage::new(
                nv_to_page,
                kernel::Grant::create(),
                0x10000000, // Start address for userspace accessible region
                0x40000,    // Length of userspace accessible region
                0x0,        // Start address of kernel region
                0x100000,   // Length of kernel region
                &mut BUFFER
            )
        );
        hil::nonvolatile_storage::NonvolatileStorage::set_client(nv_to_page, nonvolatile_storage);
        Some(nonvolatile_storage)
    }
}
