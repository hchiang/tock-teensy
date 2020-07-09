use kernel;
use capsules::clock_pm;
use components::Component;

pub struct ClockManagerComponent {
    chip_configs: &'static kernel::hil::clock_pm::ClockConfigs, 
}

impl ClockManagerComponent {
    pub fn new(chip_configs: &'static kernel::hil::clock_pm::ClockConfigs) -> Self {
        ClockManagerComponent {
            chip_configs: chip_configs,
        }
    }
}

impl Component for ClockManagerComponent {
    type Output = &'static clock_pm::ClockManagement<'static>;

    unsafe fn finalize(&mut self) -> Option<Self::Output> {

        let all_clocks = self.chip_configs.get_all_clocks();
        let max_freq = self.chip_configs.get_max_freq();

        let clients = [
            clock_pm::ClockData::new(&clock_pm::CLIENT_INDEX0, all_clocks, max_freq),
            clock_pm::ClockData::new(&clock_pm::CLIENT_INDEX1, all_clocks, max_freq),
            clock_pm::ClockData::new(&clock_pm::CLIENT_INDEX2, all_clocks, max_freq),
            clock_pm::ClockData::new(&clock_pm::CLIENT_INDEX3, all_clocks, max_freq),
            clock_pm::ClockData::new(&clock_pm::CLIENT_INDEX4, all_clocks, max_freq),
            clock_pm::ClockData::new(&clock_pm::CLIENT_INDEX5, all_clocks, max_freq),
            clock_pm::ClockData::new(&clock_pm::CLIENT_INDEX6, all_clocks, max_freq),
            clock_pm::ClockData::new(&clock_pm::CLIENT_INDEX7, all_clocks, max_freq),
            clock_pm::ClockData::new(&clock_pm::CLIENT_INDEX8, all_clocks, max_freq),
            clock_pm::ClockData::new(&clock_pm::CLIENT_INDEX9, all_clocks, max_freq),
        ];

        let clock_manager = static_init!(
            clock_pm::ClockManagement<'static>,
            clock_pm::ClockManagement::new(
                self.chip_configs,
                clients,
                self.chip_configs.get_default(),
            )
        );

        Some(clock_manager)
    }
}
