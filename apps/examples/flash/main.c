#include <timer.h>

#include <internal/nonvolatile_storage.h>
#include "ninedof.h"

bool rdone = false;
bool wdone = false;
static void write_done(int length,
                       __attribute__ ((unused)) int arg1,
                       __attribute__ ((unused)) int arg2,
                       __attribute__ ((unused)) void* ud) {
    wdone = true;
}
static void read_done(int length,
                       __attribute__ ((unused)) int arg1,
                       __attribute__ ((unused)) int arg2,
                       __attribute__ ((unused)) void* ud) {
    rdone = true;
}

int main(void) {
  int len = 500;
  uint8_t readbuf[len];
  uint8_t writebuf[len];
  int ret = nonvolatile_storage_internal_read_buffer(readbuf, len);
  if (ret != 0) printf("Read buffer error\n");
  ret = nonvolatile_storage_internal_read_done_subscribe(read_done, NULL);
  if (ret != 0) printf("ERROR setting read done callback\n");
  ret = nonvolatile_storage_internal_write_buffer(writebuf, len);
  if (ret != 0) printf("Write buffer error\n");
  ret = nonvolatile_storage_internal_write_done_subscribe(write_done, NULL);
  if (ret != 0) printf("ERROR setting write done callback\n");

  printf("Begin\n");
  int num_runs = 0;
  while(num_runs < 5) {
    for(int i=0; i<len; i++){
      writebuf[i] = 4+num_runs;
    }

    wdone = false;
    ret  = nonvolatile_storage_internal_write(0, len);
    if (ret != 0) {
      printf("\tERROR calling write\n");
      return ret;
    }
    yield_for(&wdone);

    rdone = false;
    ret  = nonvolatile_storage_internal_read(0, len);
    if (ret != 0) {
      printf("\tERROR calling write\n");
      return ret;
    }
    yield_for(&rdone);

    printf("Readbuf:");
    for(int i=0; i< len; i++){
        printf("%d ", readbuf[len]);
    }
    printf("\n");

    num_runs += 1;
    delay_ms(300);
  }
  printf("Done\n");
}
