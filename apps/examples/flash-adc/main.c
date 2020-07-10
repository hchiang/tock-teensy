#include <timer.h>
#include <adc.h>
#include <internal/nonvolatile_storage.h>
#include "ninedof.h"
#include "fft.h"

bool wdone = false;
static void write_done(__attribute__ ((unused)) int length,
                       __attribute__ ((unused)) int arg1,
                       __attribute__ ((unused)) int arg2,
                       __attribute__ ((unused)) void* ud) {
    wdone = true;
}

float movingAvg(float prev_avg, int num_samples, int new_val);
float movingAvg(float prev_avg, int num_samples, int new_val)
{
  return prev_avg + ((float)(new_val) - prev_avg) / num_samples;
}

int main(void) {

  int adc_length = 500;
  uint16_t adc_buffer[adc_length];
  uint8_t channel = 0;
  uint32_t freq = 125000;

  int fft_buf[16];
  int fft_mag[8];
  float avg_fft_mag[8]; //For each frequency bin, keep moving average of magnitude

  int flash_len = sizeof(float)*8;
  uint8_t writebuf[flash_len];
  int ret = nonvolatile_storage_internal_write_buffer(writebuf, flash_len);
  if (ret != 0) printf("Write buffer error\n");
  ret = nonvolatile_storage_internal_write_done_subscribe(write_done, NULL);
  if (ret != 0) printf("ERROR setting write done callback\n");

  //printf("Begin\n");
  while(true) {
    int i, j, k;
    //Stack issues if we increase adc buffer size so instead sample multiple times
    for(i=0; i< 4; i++) {
        int err = adc_sample_buffer_sync(channel, freq, adc_buffer, adc_length);
        if (err < 0) {
            printf("Error sampling ADC: %d\n", err);
        }
    }

    for(j=0; j<4; j++) {
        for (k=0; k<adc_length/16; k++) {
          for (i=k*16; i<(k+1)*16; i++) {
            fft_buf[i % 16] = adc_buffer[i]; // Copy needed bc fft alg I found is int not uint16
          }
          fft(fft_buf, fft_mag);
          int l;
          // For each returned fft magnitude, update the moving average for that magnitude bin
          for (l=3; l<8; l++) {
            avg_fft_mag[l] = movingAvg(avg_fft_mag[l], k, fft_mag[l]);
          }
        }
    }

    memcpy(writebuf, (void*)&avg_fft_mag[0], flash_len);

    wdone = false;
    ret  = nonvolatile_storage_internal_write(0, 2000);
    if (ret != 0) {
      printf("\tERROR calling write\n");
      return ret;
    }
    yield_for(&wdone);

   //printf("Done\n");

    delay_ms(500);
  }
}
