// Taken from https://www.projectrhea.org/rhea/index.php/Embedded_Fixed_Point_FFT#C_Code
#include "fft.h"

/* FFT CONSTANTS*/
#define SIN2PI 49//sin(2*pi/16) * 2^7
#define SIN4PI 90
#define SIN6PI 118
#define COS2PI_P_SIN2PI 167//(cos(2*pi/16) + sin(2*pi/16)) * 2^7
#define COS2PI_M_SIN2PI 56//cos(2*pi/16) - sin(2*pi/16)) * 2^7
#define COS6PI_P_SIN6PI 167
#define COS6PI_M_SIN6PI -56
#define OneTwentyEight 128
void fft(int inarr[16],int mags[8] ) {
  int atemp, temp, temp0, temp1, temp2, temp3, temp4, temp5, temp6, temp7, temp8;
  int temp9,temp10,temp11,temp12,temp13,temp14,temp15;
  int outarr[16]; temp0=inarr[0]+inarr[8];
  temp1=inarr[1]+inarr[9];
  temp2=inarr[2]+inarr[10];
  temp3=inarr[3]+inarr[11];
  temp4=inarr[4]+inarr[12];
  temp5=inarr[5]+inarr[13];
  temp6=inarr[6]+inarr[14];
  temp7=inarr[7]+inarr[15];
  temp8=inarr[0]-inarr[8];
  temp9=inarr[1]-inarr[9];
  temp10=inarr[2]-inarr[10];
  temp11=inarr[3]-inarr[11];
  temp12=inarr[12]-inarr[4];
  temp13=inarr[13]-inarr[5];
  temp14=inarr[14]-inarr[6];
  temp15=inarr[15]-inarr[7]; temp =(temp13-temp9)*(SIN2PI)/OneTwentyEight;
  temp9=temp9*(COS2PI_P_SIN2PI)/OneTwentyEight+temp;
  temp13=temp13*(COS2PI_M_SIN2PI)/OneTwentyEight+temp; temp10 = temp10 *   (SIN4PI)/OneTwentyEight;
   temp14 = temp14 * (SIN4PI)/OneTwentyEight;
  atemp = temp14;
  temp14=temp14-temp10;
  temp10+=atemp; temp = (temp15-temp11)*(SIN6PI)/OneTwentyEight;
  temp15=temp15*(COS6PI_P_SIN6PI)/OneTwentyEight+temp;
  temp11=temp11*(COS6PI_M_SIN6PI)/OneTwentyEight+temp;

  atemp = temp8;
  temp8+=temp10;
  temp10=atemp-temp10;
  atemp = temp9;
  temp9+=temp11;
  temp11=atemp-temp11;
  atemp = temp12;
  temp12+=temp14;
  temp14=atemp-temp14;
  atemp = temp13;
  temp13+=temp15;
  temp15=atemp-temp15; outarr[1]=temp8+temp9;
  outarr[3]=temp10-temp15;
  outarr[5]=temp10+temp15;
  outarr[7]=temp8-temp9;
  outarr[9]=temp12+temp13;
  outarr[11]=-temp14-temp11;
  outarr[13]=temp14-temp11;
  outarr[15]=temp13-temp12;
  atemp = temp0;
  temp0=temp0+temp4;
  temp4=atemp-temp4;
  atemp = temp1;
  temp1=temp1+temp5;
  temp5=atemp-temp5;
  atemp = temp2;
  temp2+=temp6;
  temp6=atemp-temp6;
  atemp = temp3;
  temp3+=temp7;
  temp7=atemp-temp7; outarr[0]=temp0+temp2;
  outarr[4]=temp0-temp2;
  temp1+=temp3;
  outarr[12]=(temp3<<1)-temp1;
  outarr[0]+=temp1;
  outarr[8]=outarr[0]-temp1-temp1; atemp = temp5 * (SIN4PI)/ OneTwentyEight;
  temp7 = temp7 * (SIN4PI)/ OneTwentyEight;
  temp5=atemp-temp7;
  temp7+=atemp; outarr[14]=temp6-temp7;
  outarr[2]=temp5+temp4;
  outarr[6]=temp4-temp5;
  outarr[10]=-temp7-temp6; //Magnitude calculations
  if(outarr[8] > 0) {mags[0] = outarr[8];}
  else {mags[0] = -outarr[8];}
  mags[1] = findRoot4(outarr[7], outarr[15]);
  mags[2] = findRoot4(outarr[6], outarr[14]);
  mags[3] = findRoot4(outarr[5], outarr[13]);
  mags[4] = findRoot4(outarr[4], outarr[12]);
  mags[5] = findRoot4(outarr[3], outarr[11]);
  mags[6] = findRoot4(outarr[2], outarr[10]);
  mags[7] = findRoot4(outarr[1], outarr[9]);
  }
int findRoot4(int x, int y)
{
  if(x < 0)x = -x;
  if(y < 0)y = -y;
  if (x > y) {
    if (x > 2180) return ((15 * (long)x) / 16) + ((15 * (long)y) / 32); //2180 is about 2^15 / 15
    return ((15 * x) / 16) + ((15 * y) / 32);
  } else{
     if (y > 2180) return (int)(((15 * (long)y) / 16) + ((15 * (long)x) / 32));
     return ((15 * y) / 16) + ((15 * x) / 32);
  }
}
