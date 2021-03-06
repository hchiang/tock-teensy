#include <stdbool.h>
#include <stdio.h>

#include <led.h>
#include <spi.h>
#include <multispi.h>
#include <timer.h>

#define BUF_SIZE 200
char rbuf[BUF_SIZE];
char wbuf[BUF_SIZE];
bool toggle = true;

static void write_cb(__attribute__ ((unused)) int arg0,
                     __attribute__ ((unused)) int arg2,
                     __attribute__ ((unused)) int arg3,
                     __attribute__ ((unused)) void* userdata) {
  int i;
  for (i = 0; i < BUF_SIZE; i++) {
    if (rbuf[i] != wbuf[i]) {
        printf("Receive failed at character %i\r\n", i);
        led_on(0);
        while (1) {}
    }
  }

  for (i = 0; i < BUF_SIZE; i++) {
    wbuf[i]++;
  }

  delay_ms(500);

  spi_read_write(wbuf, rbuf, BUF_SIZE, write_cb, NULL);
}

// This function can operate in one of two modes. Either
// a periodic timer triggers an SPI operation, or SPI
// operations are performed back-to-back (callback issues
// the next one.) The periodic one writes 6 byte messages,
// the back-to-back writes a 10 byte message, followed by
// 6 byte ones.
//
// In both cases, the calls alternate on which of two
// buffers is used as the write buffer. The first call
// uses the buffer initialized to 0..199. The
// 2n calls use the buffer initialized to 0.
//
// If you use back-to-back operations, the calls
// both read and write. Periodic operations only
// write. Therefore, if you set SPI to loopback
// and use back-to-back // loopback, then the read buffer
// on the first call will read in the data written.  As a
// result, you can check if reads work properly: all writes
// will be 0..n rather than all 0s.

int main(void) {
  int i;
  for (i = 0; i < BUF_SIZE; i++) {
    wbuf[i] = i;
  }

  select_spi_bus(0);
  spi_set_chip_select(0);
  spi_set_rate(20000000);
  spi_set_polarity(false);
  spi_set_phase(false);
  spi_read_write(wbuf, rbuf, BUF_SIZE, write_cb, NULL);
}
