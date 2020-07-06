#include <adc.h>
#include <timer.h>

int main(void) {

  uint8_t channel = 0;
  uint32_t freq = 125000;
  uint32_t length = 100;
  uint16_t buf[length];
  for (uint32_t i = 0; i < length; i++) {
    buf[i] = 0;
  }

  while(1) {
    int err = adc_sample_buffer_sync(channel, freq, buf, length);

    if (err < 0) {
        printf("Error sampling ADC: %d\n", err);
    }
    else {
        printf("Sample taken\n");
        //printf("\t[");
        //for (uint32_t i = 0; i < length; i++) {
        //    printf("%u ", buf[i]);
        //}
        //printf("]\n ");
    }

    // This delay uses an underlying timer in the kernel.
    delay_ms(500);
  }
}
