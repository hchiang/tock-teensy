use mk66;
use capsules::adc;
use components::Component;
use kernel::hil::adc::Adc;

pub struct AdcComponent;

impl AdcComponent {
    pub fn new() -> Self {
        AdcComponent {}
    }
}

impl Component for AdcComponent {
    type Output = &'static adc::Adc<'static, mk66::adc::Adc>;

    unsafe fn finalize(&mut self) -> Option<Self::Output> {

    mk66::adc::ADC0.initialize();
    mk66::adc::ADC1.initialize();

    let adc_channels = static_init!(
        [&'static mk66::adc::AdcChannel; 6],
        [
            &mk66::adc::CHANNEL0_A0,
            &mk66::adc::CHANNEL0_A1,
            &mk66::adc::CHANNEL0_A2,
            &mk66::adc::CHANNEL0_A3,
            &mk66::adc::CHANNEL0_A4,
            &mk66::adc::CHANNEL0_A5,
        ]
    );
    let adc = static_init!(
        adc::Adc<'static, mk66::adc::Adc>,
        adc::Adc::new(
            &mut mk66::adc::ADC0,
            adc_channels,
            &mut adc::ADC_BUFFER1,
            &mut adc::ADC_BUFFER2,
            &mut adc::ADC_BUFFER3
        )
    );
    mk66::adc::ADC0.set_client(adc);
        Some(adc)
    }
}
